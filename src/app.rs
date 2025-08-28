use std::{error::Error, io, time::Duration};

use crate::{audio::play_lesson_audio, lesson, morse, scores::ScoreData, ui::draw_ui};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    style::{Color, Style},
    text::Span,
};
use strsim::levenshtein;

pub enum Mode {
    PickingLesson,
    TypingLesson,
    LetterPractice,
}

pub struct App {
    pub mode: Mode,
    pub selected: usize, // 0-based index into lessons
    pub scroll_offset: usize,
    pub user_input: String,
    pub scores: ScoreData,
    pub wpm: u32,
    pub effective_wpm: u32,
    pub freq: f32,
    pub sink: Option<rodio::Sink>,
    pub stream: Option<rodio::OutputStream>,
    pub current_practice: String,
    pub visible_rows: usize,
    pub highlighted_results: Option<Vec<Span<'static>>>,
    pub letter_practice: String,
}

impl App {
    pub fn new(wpm: u32, effective_wpm: u32, freq: f32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            mode: Mode::PickingLesson,
            selected: 0,
            scroll_offset: 0,
            user_input: String::new(),
            scores: confy::load("moser", None)?,
            wpm,
            effective_wpm,
            freq,
            sink: None,
            stream: None,
            current_practice: String::new(),
            visible_rows: 0,
            highlighted_results: None,
            letter_practice: String::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let total_lessons = morse::KOCH_SEQUENCE.len() - 1;

        loop {
            terminal.draw(|f| draw_ui(f, self, total_lessons))?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if self.handle_key(key.code, total_lessons)? {
                            break;
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

    fn handle_key(&mut self, code: KeyCode, total_lessons: usize) -> Result<bool, Box<dyn Error>> {
        match self.mode {
            Mode::PickingLesson => match code {
                KeyCode::Char('q') => return Ok(true), // quit
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected = (self.selected + 1) % total_lessons;
                    if self.selected >= self.scroll_offset + self.visible_rows {
                        self.scroll_offset += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected = (self.selected + total_lessons - 1) % total_lessons;
                    if self.selected < self.scroll_offset && self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
                KeyCode::Enter => {
                    self.user_input.clear();
                    self.current_practice.clear();
                    self.highlighted_results = None;
                    self.current_practice = lesson::lesson_text(self.selected + 1);
                    let (stream, sink) = play_lesson_audio(
                        &self.current_practice,
                        self.wpm,
                        self.freq,
                        self.effective_wpm,
                    )?;
                    self.stream = Some(stream);
                    self.sink = Some(sink);
                    self.mode = Mode::TypingLesson;
                }
                KeyCode::Char('p') => {
                    self.mode = Mode::LetterPractice;
                    self.letter_practice = lesson::practice_text(self.selected + 1);
                    let (stream, sink) = play_lesson_audio(
                        &self.letter_practice,
                        self.wpm,
                        self.freq,
                        self.effective_wpm,
                    )?;
                    self.stream = Some(stream);
                    self.sink = Some(sink);
                }
                _ => {}
            },
            Mode::TypingLesson => match code {
                KeyCode::Char(c) => self.user_input.push(c),
                KeyCode::Backspace => {
                    self.user_input.pop();
                }
                KeyCode::Enter => {
                    self.finish_typing()?;
                    self.mode = Mode::PickingLesson;
                }
                KeyCode::Esc => {
                    self.mode = Mode::PickingLesson;
                    if let Some(sink) = self.sink.take() {
                        sink.stop();
                    }
                    self.stream.take();
                }
                _ => {}
            },
            Mode::LetterPractice => match code {
                KeyCode::Esc => {
                    self.mode = Mode::PickingLesson;
                    if let Some(sink) = self.sink.take() {
                        sink.stop();
                    }
                    self.stream.take();
                }
                _ => {}
            },
        }
        Ok(false)
    }

    fn finish_typing(&mut self) -> Result<(), Box<dyn Error>> {
        let lesson_num = self.selected + 1;
        let typed = self.user_input.trim().to_uppercase();
        let practice = self.current_practice.to_uppercase();

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
        self.highlighted_results = Some(spans);

        let distance = levenshtein(&typed, &self.current_practice);
        let max_len = self.current_practice.len().max(typed.len());
        let accuracy = if max_len == 0 {
            0
        } else {
            ((max_len - distance) * 100 / max_len) as u32
        };
        self.scores
            .lessons
            .entry(lesson_num.to_string())
            .or_default()
            .push(accuracy);
        confy::store("moser", None, &self.scores)?;

        Ok(())
    }
}
