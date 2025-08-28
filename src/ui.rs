use crate::{
    app::{App, Mode},
    lesson,
};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Style},
    symbols,
    text::Text,
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, Paragraph, Row, Table},
};

pub fn draw_ui(f: &mut Frame, app: &mut App, total_lessons: usize) {
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
            Row::new(vec!["Lesson", "New character(s)"]).style(Style::default().fg(Color::Cyan)),
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
        "Lesson {}\n\nChar WPM: {}\nEffective WPM: {}\n\nPress <q> to quit\nPress <p> to hear letters",
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
    } else if !matches!(app.mode, Mode::LetterPractice) {
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

    match app.mode {
        Mode::LetterPractice => {
            let area = centered_rect(60, 20, f.area());
            f.render_widget(Clear, area);
            let block = Block::default()
                .title("Letter Practice (Esc to close)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black));

            let text = Paragraph::new(app.letter_practice.clone())
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(text, area);
        }
        _ => {}
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
