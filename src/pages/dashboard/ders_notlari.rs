use std::sync::{Arc, Mutex};

use crate::client::obs::{Course, ObsClient};
use crate::pages::PageAction;
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

pub struct DersNotlari {
    pub courses: Vec<Course>,
    state: TableState,
    selected_course_idx: usize,
    client: Arc<ObsClient>,
    year: i32,
    term: i32,
    fetch_result: Arc<Mutex<Option<Vec<Course>>>>,
}

impl DersNotlari {
    pub fn new(client: Arc<ObsClient>) -> Self {
        Self {
            state: TableState::default(),
            selected_course_idx: 0,
            courses: Vec::new(),
            year: 2025,
            term: 1,
            client,
            fetch_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn fetch_courses(&mut self) {
        let y_str = self.year.to_string();
        let t_str = self.term.to_string();
        let client = self.client.clone();
        let result_slot = self.fetch_result.clone();

        tokio::spawn(async move {
            let res = if let Ok(courses) = client.get_exam_results(&y_str, &t_str).await {
                courses
            } else {
                Vec::new()
            };
            *result_slot.lock().unwrap() = Some(res);
        });
    }

    pub fn update(&mut self) -> PageAction {
        let res = self.fetch_result.lock().unwrap().take();
        if let Some(res) = res {
            self.courses = res;
            self.selected_course_idx = 0;
            self.update_state(false);
        }
        PageAction::None
    }

    fn next_term(&mut self) -> PageAction {
        let (y, t) = if self.term == 2 {
            (self.year + 1, 1)
        } else {
            (self.year, self.term + 1)
        };
        self.year = y;
        self.term = t;
        self.fetch_courses();
        PageAction::None
    }

    fn prev_term(&mut self) -> PageAction {
        let (y, t) = if self.term == 1 {
            (self.year - 1, 2)
        } else {
            (self.year, self.term - 1)
        };
        self.year = y;
        self.term = t;
        self.fetch_courses();
        PageAction::None
    }

    fn update_state(&mut self, align_bottom: bool) {
        if self.courses.is_empty() {
            return;
        }
        let mut target_row = 0;
        for i in 0..self.selected_course_idx {
            target_row += 1;
            target_row += self.courses[i].exams.len();
            target_row += 1;
        }

        let offset = if align_bottom {
            self.courses[self.selected_course_idx].exams.len()
        } else {
            0
        };

        self.state.select(Some(target_row + offset));
    }

    pub fn next(&mut self) {
        if !self.courses.is_empty() {
            self.selected_course_idx = (self.selected_course_idx + 1) % self.courses.len();
            let align_bottom = self.selected_course_idx != 0;
            self.update_state(align_bottom);
        }
    }

    pub fn previous(&mut self) {
        if !self.courses.is_empty() {
            let align_bottom = if self.selected_course_idx == 0 {
                self.selected_course_idx = self.courses.len() - 1;
                true
            } else {
                self.selected_course_idx -= 1;
                false
            };
            self.update_state(align_bottom);
        }
    }

    fn get_grade_color(grade: &str) -> Color {
        if grade.contains("AA") || grade.contains("BA") || grade.contains("BB") {
            return Color::Green;
        }
        if grade.contains("FF") || grade.contains("DC") || grade.contains("DD") {
            return Color::Red;
        }
        Color::White
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(area);

        let mut rows = Vec::new();

        for (idx, course) in self.courses.iter().enumerate() {
            let is_selected = idx == self.selected_course_idx;

            let course_style = if is_selected {
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            };

            rows.push(
                Row::new(vec![
                    Cell::from(course.name.clone()),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .style(course_style),
            );

            for exam in &course.exams {
                let is_harf = exam.name.contains("HARF NOTU");
                let grade_color = if is_harf {
                    Self::get_grade_color(&exam.grade)
                } else {
                    Color::White
                };

                rows.push(Row::new(vec![
                    Cell::from(format!("  • {}", exam.name)).style(
                        Style::default().fg(if is_harf { Color::Magenta } else { Color::Gray }),
                    ),
                    Cell::from(exam.weight.clone()).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(exam.date.clone()).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(exam.grade.clone()).style(Style::default().fg(grade_color)),
                ]));
            }

            rows.push(Row::new(vec![Cell::from("")]));
        }

        let table = Table::new(rows)
            .header(
                Row::new(vec!["Ders", "Etki", "Tarih", "Not"])
                    .style(Style::default().fg(Color::Cyan)),
            )
            .block(Block::default().borders(Borders::ALL).title(format!(
                " Ders Notları ({} - {}. Dönem) ",
                self.year, self.term
            )))
            .highlight_style(Style::default())
            .widths(&[
                Constraint::Percentage(45),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]);

        frame.render_stateful_widget(table, chunks[0], &mut self.state);

        if let Some(course) = self.courses.get(self.selected_course_idx) {
            let scale_text = course.grade_scale.as_deref().unwrap_or("Harf notu yok");
            let scale_widget = Paragraph::new(scale_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Harf Notları "),
                )
                .style(Style::default().fg(Color::White));

            frame.render_widget(scale_widget, chunks[1]);
        }
    }

    pub fn handle_event(&mut self, event: Event) -> PageAction {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.next(),
                KeyCode::Up | KeyCode::Char('k') => self.previous(),
                KeyCode::Right | KeyCode::Char('l') => return self.next_term(),
                KeyCode::Left | KeyCode::Char('h') => return self.prev_term(),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => self.next(),
                MouseEventKind::ScrollUp => self.previous(),
                _ => {}
            },
            _ => {}
        }
        PageAction::None
    }
}
