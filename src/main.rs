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
    text::{Span, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph, Row, Table},
};

#[derive(Parser, Debug)]
struct Args {
    /// character speed
    #[arg(short, long, default_value_t = 20)]
    wpm: u32,

    /// effective overall wpm
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
    scroll_offset: usize,
    user_input: String,
    scores: ScoreData,
    wpm: u32,
    effective_wpm: u32,
    freq: f32,
    sink: Option<rodio::Sink>,
    stream: Option<rodio::OutputStream>,
    current_practice: String,
    visible_rows: usize,
    highlighted_results: Option<Vec<Span<'static>>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut app = App {
        mode: Mode::PickingLesson,
        selected: 0,
        scroll_offset: 0,
        user_input: String::new(),
        scores: confy::load("moser", None)?,
        wpm: args.wpm,
        effective_wpm: args.effective_wpm,
        freq: args.tone_freq,
        sink: None,
        stream: None,
        current_practice: String::new(),
        visible_rows: 0,
        highlighted_results: None,
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let total_lessons = morse::KOCH_SEQUENCE.len() - 1;
    loop {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(size);

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[0]);

            let all_rows: Vec<Row> = (1..=total_lessons)
                .map(|i| {
                    let lesson_str = format!("{}", i);
                    let new_chars = format!("{:?}", lesson::new_letters_for_lesson(i));
                    let mut row = Row::new(vec![lesson_str, new_chars]);
                    if i - 1 == app.selected {
                        row = row.style(Style::default().fg(Color::Yellow));
                    }
                    row
                })
                .collect();

            let height = top_chunks[0].height.saturating_sub(3) as usize;
            app.visible_rows = height;
            let start = app.scroll_offset.min(all_rows.len().saturating_sub(height));
            let end = (start + height).min(all_rows.len());
            let visible = &all_rows[start..end];
            let columns = vec![Constraint::Length(8), Constraint::Length(20)];

            let table = Table::new(visible.to_vec(), columns)
                .header(
                    Row::new(vec!["Lesson", "New character(s)"])
                        .style(Style::default().fg(Color::Cyan)),
                )
                .block(
                    Block::default()
                        .title("Lessons (↑/↓, Enter)")
                        .borders(Borders::ALL)
                        .border_style(if matches!(app.mode, Mode::PickingLesson) {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default()
                        }),
                )
                .widths(&[Constraint::Length(8), Constraint::Length(20)]);
            f.render_widget(table, top_chunks[0]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(top_chunks[1]);

            let lesson_num = app.selected + 1;

            let details_text = format!(
                "Lesson {}\n\nChar WPM: {}\nEffective WPM: {}\n\nPress <q> to quit",
                lesson_num, app.wpm, app.effective_wpm
            );

            let details = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .title("Lesson Details")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Center);
            f.render_widget(details, right_chunks[0]);

            let scores_vec: Vec<u32> = app
                .scores
                .lessons
                .get(&lesson_num.to_string())
                .unwrap_or(&Vec::new())
                .iter()
                .take(10)
                .cloned()
                .collect();
            let data: Vec<(f64, f64)> = scores_vec
                .iter()
                .enumerate()
                .map(|(i, s)| (i as f64, *s as f64))
                .collect();
            let data2: Vec<(f64, f64)> = (0..scores_vec.len()).map(|i| (i as f64, 90.0)).collect();

            let datasets = vec![
                Dataset::default()
                    .name("Accuracy")
                    .marker(symbols::Marker::Dot)
                    .graph_type(ratatui::widgets::GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&data),
                Dataset::default()
                    .name("90%")
                    .marker(symbols::Marker::Dot)
                    .graph_type(ratatui::widgets::GraphType::Line)
                    .style(Style::default().fg(Color::Red))
                    .data(&data2),
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

            let cursor = if matches!(app.mode, Mode::TypingLesson) {
                "_"
            } else {
                ""
            };
            let display_input = format!("{}{}", app.user_input, cursor);

            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(display_input));

            if let Some(spans) = &app.highlighted_results {
                lines.push(Line::from(spans.clone()));
            } else {
                lines.push(Line::from(format!("{}", app.current_practice)));
            }

            let input_box = Paragraph::new(Text::from(lines))
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

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        Mode::PickingLesson => match key.code {
                            KeyCode::Char('q') => break, // quit
                            KeyCode::Down => {
                                app.selected = (app.selected + 1) % total_lessons;
                                if app.selected >= app.scroll_offset + app.visible_rows {
                                    app.scroll_offset += 1;
                                }
                            }
                            KeyCode::Up => {
                                app.selected = (app.selected + total_lessons - 1) % total_lessons;
                                if app.selected < app.scroll_offset && app.scroll_offset > 0 {
                                    app.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Enter => {
                                app.user_input.clear();
                                app.current_practice.clear();
                                app.current_practice = lesson::lesson_text(app.selected + 1);
                                if let Ok((stream, sink)) = play_lesson_audio(
                                    &app.current_practice,
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
                                let lesson_num = app.selected + 1;
                                let typed = app.user_input.trim().to_uppercase();
                                let practice = app.current_practice.to_uppercase();

                                let mut spans = Vec::new();
                                for (uc, pc) in typed.chars().zip(practice.chars()) {
                                    if uc == pc {
                                        spans.push(Span::styled(
                                            uc.to_string(),
                                            Style::default().fg(Color::Green),
                                        ))
                                    } else {
                                        spans.push(Span::styled(
                                            uc.to_string(),
                                            Style::default().fg(Color::Red),
                                        ))
                                    }
                                }
                                if typed.len() < practice.len() {
                                    for pc in practice.chars().skip(typed.len()) {
                                        spans.push(Span::styled(
                                            pc.to_string(),
                                            Style::default().fg(Color::Red),
                                        ))
                                    }
                                }
                                app.highlighted_results = Some(spans);

                                let distance = levenshtein(&typed, &app.current_practice);
                                let max_len = app.current_practice.len().max(typed.len());
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
                                if let Some(sink) = app.sink.take() {
                                    sink.stop();
                                }
                                app.stream.take();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn play_lesson_audio(
    lesson_text: &str,
    wpm: u32,
    freq: f32,
    effective_wpm: u32,
) -> Result<(rodio::OutputStream, rodio::Sink), Box<dyn Error>> {
    let sample_rate = 44_100;
    let map = morse::morse_map();

    let audio = audio::MorseAudio::new(wpm, effective_wpm, freq, sample_rate);

    let mut samples: Vec<f32> = Vec::new();
    for ch in lesson_text.chars() {
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
