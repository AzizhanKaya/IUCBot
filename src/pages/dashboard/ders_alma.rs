use crate::client::obs::{AlinabilecekDers, ObsClient};
use crate::router::PageAction;
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::rc::Rc;

pub enum State {
    Loading,
    Loaded,
    Saved,
    OutOfSchedule,
    Error(String),
}

pub struct DersAlma {
    course_list_state: ListState,
    pub sections: Vec<(String, Vec<AlinabilecekDers>)>,
    state: State,
    client: Option<Rc<ObsClient>>,
    error: Option<String>,
    needs_fetch: bool,
    needs_save: bool,
    action_ready: bool,
}

impl Default for DersAlma {
    fn default() -> Self {
        Self {
            course_list_state: ListState::default(),
            sections: Vec::new(),
            state: State::Loading,
            client: None,
            error: None,
            needs_fetch: false,
            needs_save: false,
            action_ready: false,
        }
    }
}

impl DersAlma {
    pub fn new(client: Rc<ObsClient>) -> Self {
        let mut inst = Self {
            course_list_state: ListState::default(),
            client: Some(client),
            sections: Vec::new(),
            state: State::Loading,
            error: None,
            needs_fetch: false,
            needs_save: false,
            action_ready: false,
        };
        inst.fetch_courses();
        inst
    }

    fn get_flat_len(&self) -> usize {
        self.sections.iter().map(|s| 1 + s.1.len()).sum()
    }

    fn is_header(&self, index: usize) -> bool {
        let mut current_idx = 0;
        for section in &self.sections {
            if index == current_idx {
                return true;
            }
            current_idx += 1 + section.1.len();
        }
        false
    }

    fn get_course_location(&self, index: usize) -> Option<(usize, usize)> {
        let mut current_idx = 0;
        for (s_idx, section) in self.sections.iter().enumerate() {
            if index == current_idx {
                return None;
            }
            current_idx += 1;
            if index < current_idx + section.1.len() {
                return Some((s_idx, index - current_idx));
            }
            current_idx += section.1.len();
        }
        None
    }

    fn fetch_courses(&mut self) {
        if let Some(client) = &self.client {
            self.state = State::Loading;
            match client.get_alinabilecek_dersler() {
                Ok(mut courses) => {
                    courses.iter_mut().for_each(|c| {
                        if c.Tipi == "Zorunlu" {
                            c.is_selected = true;
                        }
                    });

                    let mut zorunlular = Vec::new();
                    let mut secmeliler = std::collections::HashMap::new();

                    for c in courses {
                        if c.Tipi == "Zorunlu" {
                            zorunlular.push(c);
                        } else {
                            let grup = c
                                .Grup
                                .clone()
                                .unwrap_or_else(|| "Diğer Seçmeliler".to_string());
                            secmeliler.entry(grup).or_insert_with(Vec::new).push(c);
                        }
                    }

                    let mut sections = Vec::new();
                    let mut basliklar: Vec<String> = secmeliler.keys().cloned().collect();
                    basliklar.sort();
                    for b in basliklar {
                        let title = if b == "Diğer Seçmeliler" {
                            b.clone()
                        } else {
                            format!("{} Grubu Seçmeli", b)
                        };
                        sections.push((title, secmeliler.remove(&b).unwrap()));
                    }
                    if !zorunlular.is_empty() {
                        sections.push(("Zorunlu Dersler".to_string(), zorunlular));
                    }

                    self.sections = sections;
                    if !self.sections.is_empty() {
                        let mut i = 0;
                        while i < self.get_flat_len() && self.is_header(i) {
                            i += 1;
                        }
                        self.course_list_state.select(Some(i));
                    }
                    self.state = State::Loaded;
                    self.error = None;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("Ders Alma Takvimi Dışındasınız") {
                        self.state = State::OutOfSchedule;
                    } else {
                        self.error = Some(err_msg.clone());
                        self.state = State::Error(err_msg);
                    }
                }
            }
        }
    }

    fn save_courses(&mut self) {
        if let Some(client) = &self.client {
            self.state = State::Loading;

            let mut selected_courses = Vec::new();
            for (_, section_courses) in &self.sections {
                selected_courses.extend(section_courses.iter().filter(|c| c.is_selected).cloned());
            }

            match client.kaydet_dersler(&selected_courses) {
                Ok(_) => {
                    self.state = State::Saved;
                    self.error = None;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("Ders Alma Takvimi Dışındasınız") {
                        self.state = State::OutOfSchedule;
                    } else {
                        self.error = Some(err_msg.clone());
                        self.state = State::Error(err_msg);
                    }
                }
            }
        }
    }

    fn next_course(&mut self) {
        if self.sections.is_empty() {
            return;
        }
        let len = self.get_flat_len();
        let mut i = self.course_list_state.selected().unwrap_or(0);
        loop {
            i += 1;
            if i >= len {
                i = 0;
            }
            if !self.is_header(i) {
                break;
            }
        }
        self.course_list_state.select(Some(i));
    }

    fn previous_course(&mut self) {
        if self.sections.is_empty() {
            return;
        }
        let len = self.get_flat_len();
        let mut i = self.course_list_state.selected().unwrap_or(0);
        loop {
            if i == 0 {
                i = len - 1;
            } else {
                i -= 1;
            }
            if !self.is_header(i) {
                break;
            }
        }
        self.course_list_state.select(Some(i));
    }

    fn toggle_course(&mut self) {
        if let Some(i) = self.course_list_state.selected() {
            if let Some((s_idx, c_idx)) = self.get_course_location(i) {
                self.sections[s_idx].1[c_idx].is_selected =
                    !self.sections[s_idx].1[c_idx].is_selected;
            }
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        if self.needs_fetch || self.needs_save {
            self.action_ready = true;
        }

        let mut items = Vec::new();
        if matches!(self.state, State::Loaded | State::Error(_)) {
            let mut idx = 0;
            for (title, section_courses) in &self.sections {
                items.push(ListItem::new(Spans::from(vec![Span::styled(
                    title.as_str(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )])));
                idx += 1;

                for course in section_courses {
                    let is_selected = self.course_list_state.selected() == Some(idx);
                    let check_mark = if course.is_selected { "[x]" } else { "[ ]" };
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    items.push(ListItem::new(Spans::from(vec![Span::styled(
                        format!(
                            "  {} {} - {} ({})",
                            check_mark, course.Kodu, course.DersAdi, course.Tipi
                        ),
                        style,
                    )])));
                    idx += 1;
                }
            }
        }

        let (title, content, show_error) = match &self.state {
            State::Loading => (
                "Yükleniyor...",
                vec![ListItem::new("Lütfen bekleyin...")],
                false,
            ),
            State::Saved => (
                "Dersler seçildi",
                vec![ListItem::new("Ders seçim işleminiz başarıyla tamamlandı.")],
                false,
            ),
            State::OutOfSchedule => (
                "Ders seçimi (r ile yenile)",
                vec![
                    ListItem::new(""),
                    ListItem::new(Spans::from(Span::styled(
                        "  Ders Alma Takvimi Dışındasınız.  ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))),
                    ListItem::new(""),
                ],
                false,
            ),
            State::Error(_) => ("Ders seçimi (boşluk ile seç, enter ile onayla", items, true),
            State::Loaded => (
                "Ders seçimi (boşluk ile seç, enter ile onayla",
                items,
                false,
            ),
        };

        if show_error || matches!(self.state, State::Loaded) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(if self.error.is_some() {
                    [Constraint::Min(0), Constraint::Length(3)]
                } else {
                    [Constraint::Min(0), Constraint::Length(0)]
                })
                .split(area);

            let list = List::new(content)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(Style::default());

            frame.render_stateful_widget(list, chunks[0], &mut self.course_list_state);

            if let Some(err) = &self.error {
                let error_p = Paragraph::new(err.as_str())
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title("Hata"));
                frame.render_widget(error_p, chunks[1]);
            }
        } else {
            let list = List::new(content)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(Style::default());

            if matches!(self.state, State::Loading | State::Saved) {
                frame.render_widget(list, area);
            } else {
                frame.render_stateful_widget(list, area, &mut self.course_list_state);
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) -> PageAction {
        if matches!(
            self.state,
            State::Loading | State::Saved | State::OutOfSchedule
        ) {
            if let Event::Key(key) = event {
                if key.code == KeyCode::Char('r')
                    && matches!(self.state, State::Saved | State::OutOfSchedule)
                {
                    self.state = State::Loading;
                    self.needs_fetch = true;
                    self.action_ready = false;
                }
            }
            return PageAction::None;
        }

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_course();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous_course();
                }
                KeyCode::Char(' ') => {
                    self.toggle_course();
                }
                KeyCode::Enter => {
                    self.state = State::Loading;
                    self.needs_save = true;
                    self.action_ready = false;
                }
                _ => {}
            }
        }
        PageAction::None
    }

    pub fn tick(&mut self) {
        if self.action_ready {
            if self.needs_fetch {
                self.needs_fetch = false;
                self.action_ready = false;
                self.fetch_courses();
            } else if self.needs_save {
                self.needs_save = false;
                self.action_ready = false;
                self.save_courses();
            }
        }
    }
}
