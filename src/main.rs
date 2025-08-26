mod audio;
mod lesson;
mod morse;
mod scores;

use clap::Parser;
use morse::KOCH_SEQUENCE;
use rodio::Sink;
use scores::ScoreData;
use strsim::levenshtein;

use std::error::Error;
use std::io;
use std::time::Duration;

use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 15)]
    wpm: u32,
    #[arg(short, long, default_value_t = 600.0)]
    tone_freq: f32,
}

enum Mode {
    PickingLesson,
    TypingLesson,
}

struct App {
    mode: Mode,
    selected: usize,
    user_input: String,
    scores: ScoreData,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut app = App {
        mode: Mode::PickingLesson,
        selected: 0,
        user_input: String::new(),
        scores: confy::load("moser", None)?,
    };

    // TUI setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Split into top (lesson pick/details) + bottom (input)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);

            // Top split into lesson list + lesson details
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[0]);

            // Lesson picker list
            let lessons: Vec<ListItem> = (1..=36)
                .map(|i| {
                    let mut item = ListItem::new(format!("Lesson {}", i));
                    if i - 1 == app.selected {
                        item = item.style(Style::default().fg(Color::Yellow));
                    }
                    item
                })
                .collect();
            let list = List::new(lessons).block(
                Block::default()
                    .title("Lessons (↑/↓, Enter)")
                    .borders(Borders::ALL),
            );
            f.render_widget(list, top_chunks[0]);

            // Lesson details
            let lesson_num = app.selected + 1;
            let history = app
                .scores
                .lessons
                .get(&lesson_num.to_string())
                .unwrap_or(&Vec::new())
                .iter()
                .rev()
                .take(5)
                .map(|s| format!("{}%", s))
                .collect::<Vec<_>>()
                .join(", ");

            let details_text = format!(
                "Lesson {}\nLetters: {:?}\nLast scores: {}\n\nPress <q> to quit",
                lesson_num,
                &KOCH_SEQUENCE[..lesson_num],
                if history.is_empty() {
                    "None".into()
                } else {
                    history
                }
            );

            let details = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .title("Lesson Details")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Center);
            f.render_widget(details, top_chunks[1]);

            // Input pane
            let input_box = Paragraph::new(app.user_input.clone())
                .block(Block::default().title("Your Input").borders(Borders::ALL))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green));
            f.render_widget(input_box, chunks[1]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        Mode::PickingLesson => match key.code {
                            KeyCode::Char('q') => break, // quit
                            KeyCode::Down => {
                                if app.selected < 35 {
                                    app.selected += 1;
                                }
                            }
                            KeyCode::Up => {
                                if app.selected > 0 {
                                    app.selected -= 1;
                                }
                            }
                            KeyCode::Enter => {
                                // start audio + switch to typing mode
                                app.user_input.clear();
                                play_lesson_audio(app.selected + 1, args.wpm, args.tone_freq)?;
                                app.mode = Mode::TypingLesson;
                            }
                            _ => {}
                        },
                        Mode::TypingLesson => match key.code {
                            KeyCode::Char(c) => app.user_input.push(c),
                            KeyCode::Backspace => {
                                app.user_input.pop();
                            }
                            KeyCode::Enter => {
                                // finish typing, score
                                let lesson_num = app.selected + 1;
                                let expected = lesson::lesson_text(lesson_num).to_uppercase();
                                let typed = app.user_input.trim().to_uppercase();
                                let distance = levenshtein(&typed, &expected);
                                let max_len = expected.len().max(typed.len());
                                let accuracy = if max_len == 0 {
                                    0
                                } else {
                                    ((max_len - distance) * 100 / max_len) as u32
                                };
                                app.scores
                                    .lessons
                                    .entry(lesson_num.to_string())
                                    .or_default()
                                    .push(accuracy);
                                confy::store("moser", None, &app.scores)?;
                                app.mode = Mode::PickingLesson;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    // cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Helper: play audio for a lesson
fn play_lesson_audio(lesson_num: usize, wpm: u32, freq: f32) -> Result<(), Box<dyn Error>> {
    let sample_rate = 44_100;
    let morse_text = lesson::lesson_text(lesson_num);
    let map = morse::morse_map();
    let audio = audio::MorseAudio::new(wpm, freq, sample_rate);

    let mut samples: Vec<f32> = Vec::new();
    for ch in morse_text.chars() {
        if let Some(code) = map.get(&ch) {
            samples.extend(audio.morse_to_audio(code));
        }
    }

    let mut stream = rodio::stream::OutputStreamBuilder::open_default_stream()?;
    stream.log_on_drop(false);
    let sink = Sink::connect_new(&stream.mixer());
    let source = audio.to_source(samples);
    sink.append(source);
    Ok(())
}
