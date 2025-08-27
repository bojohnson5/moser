mod audio;
mod lesson;
mod morse;
mod scores;

use clap::Parser;
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
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, List, ListItem, Paragraph},
};

#[derive(Parser, Debug)]
struct Args {
    /// character speed (WPM for dots/dashes)
    #[arg(short, long, default_value_t = 20)]
    wpm: u32,

    /// effective overall WPM (Farnsworth spacing)
    #[arg(long, default_value_t = 15)]
    effective_wpm: u32,

    /// tone frequency (Hz)
    #[arg(short, long, default_value_t = 600.0)]
    tone_freq: f32,
}

enum Mode {
    PickingLesson,
    TypingLesson,
}

struct App {
    mode: Mode,
    selected: usize, // 0-based index into lessons
    user_input: String,
    scores: ScoreData,
    wpm: u32,
    effective_wpm: u32,
    freq: f32,
    sink: Option<rodio::Sink>,
    stream: Option<rodio::OutputStream>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut app = App {
        mode: Mode::PickingLesson,
        selected: 0,
        user_input: String::new(),
        scores: confy::load("moser", None)?,
        wpm: args.wpm,
        effective_wpm: args.effective_wpm,
        freq: args.tone_freq,
        sink: None,
        stream: None,
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
                    .borders(Borders::ALL)
                    .border_style(if matches!(app.mode, Mode::PickingLesson) {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            );
            f.render_widget(list, top_chunks[0]);

            // Lesson details split: info + chart
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(top_chunks[1]);

            let lesson_num = app.selected + 1;
            let new_chars = lesson::new_letters_for_lesson(lesson_num);

            let details_text = format!(
                "Lesson {}\nNew character(s): {:?}\n\nPress <q> to quit",
                lesson_num, new_chars
            );

            let details = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .title("Lesson Details")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Center);
            f.render_widget(details, right_chunks[0]);

            // Chart for last scores
            let scores_vec: Vec<u32> = app
                .scores
                .lessons
                .get(&lesson_num.to_string())
                .unwrap_or(&Vec::new())
                .iter()
                .rev()
                .take(10)
                .cloned()
                .collect();
            let data: Vec<(f64, f64)> = scores_vec
                .iter()
                .enumerate()
                .map(|(i, s)| (i as f64, *s as f64))
                .collect();

            let datasets = vec![
                Dataset::default()
                    .name("Accuracy")
                    .marker(symbols::Marker::Dot)
                    .graph_type(ratatui::widgets::GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&data),
            ];

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title("Last Scores")
                        .borders(Borders::ALL)
                        .border_style(if matches!(app.mode, Mode::PickingLesson) {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default()
                        }),
                )
                .x_axis(
                    Axis::default()
                        .bounds([0.0, data.len().max(1) as f64])
                        .labels(["0".into(), format!("{}", data.len())]),
                )
                .y_axis(
                    Axis::default()
                        .bounds([0.0, 100.0])
                        .labels(["0%", "50%", "100%"]),
                );
            f.render_widget(chart, right_chunks[1]);

            // Input pane
            let input_box = Paragraph::new(app.user_input.clone())
                .block(
                    Block::default()
                        .title("Your Input")
                        .borders(Borders::ALL)
                        .border_style(if matches!(app.mode, Mode::TypingLesson) {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default()
                        }),
                )
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
                                app.selected = (app.selected + 1) % 36;
                            }
                            KeyCode::Up => {
                                app.selected = (app.selected + 35) % 36;
                            }
                            KeyCode::Enter => {
                                app.user_input.clear();
                                if let Ok((stream, sink)) = play_lesson_audio(
                                    app.selected + 1,
                                    app.wpm,
                                    app.freq,
                                    app.effective_wpm,
                                ) {
                                    app.stream = Some(stream);
                                    app.sink = Some(sink);
                                }
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
                            KeyCode::Esc => {
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

/// Play audio for a lesson with Farnsworth spacing
fn play_lesson_audio(
    lesson_num: usize,
    wpm: u32,
    freq: f32,
    effective_wpm: u32,
) -> Result<(rodio::OutputStream, rodio::Sink), Box<dyn Error>> {
    let sample_rate = 44_100;
    let morse_text = lesson::lesson_text(lesson_num);
    let map = morse::morse_map();

    let audio = audio::MorseAudio::new(wpm, effective_wpm, freq, sample_rate);

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
    Ok((stream, sink))
}
