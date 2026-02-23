use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::sync::{Arc, Mutex};

use crate::client::obs::{ObsClient, OzlukBilgileriData};
use crate::pages::PageAction;
use crossterm::event::{Event, KeyCode};

pub enum State {
    Loading,
    Loaded(OzlukBilgileriData),
    Error(String),
}

pub struct OzlukBilgileri {
    client: Option<Arc<ObsClient>>,
    state: State,
    fetch_result: Arc<Mutex<Option<Result<OzlukBilgileriData, String>>>>,
}

impl Default for OzlukBilgileri {
    fn default() -> Self {
        Self {
            client: None,
            state: State::Loading,
            fetch_result: Arc::new(Mutex::new(None)),
        }
    }
}

impl OzlukBilgileri {
    pub fn new(client: Arc<ObsClient>) -> Self {
        Self {
            client: Some(client),
            state: State::Loading,
            fetch_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn fetch_data(&mut self) {
        if let Some(client) = &self.client {
            self.state = State::Loading;
            let client = client.clone();
            let result_slot = self.fetch_result.clone();
            tokio::spawn(async move {
                let mapped = match client.get_ozluk_bilgileri().await {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        log::error!("Özlük bilgileri fetch hatası: {}", e);
                        Err(e.to_string())
                    }
                };
                *result_slot.lock().unwrap() = Some(mapped);
            });
        }
    }

    pub fn update(&mut self) -> PageAction {
        if let Some(res) = self.fetch_result.lock().unwrap().take() {
            match res {
                Ok(data) => self.state = State::Loaded(data),
                Err(e) => {
                    log::error!("Özlük bilgileri update hatası: {}", e);
                    self.state = State::Error(e);
                }
            }
        }
        PageAction::None
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Min(10),
            ])
            .split(area);

        let info_block = Block::default()
            .borders(Borders::ALL)
            .title(Spans::from(vec![Span::styled(
                " Özlük Bilgileri ",
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]));

        frame.render_widget(info_block.clone(), chunks[0]);

        let info_inner = info_block.inner(chunks[0]);
        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(10)])
            .split(info_inner);

        let photo = Paragraph::new(vec![
            Spans::from("  .---.  "),
            Spans::from(" /     \\ "),
            Spans::from("| O  O  |"),
            Spans::from(" \\  ^  / "),
            Spans::from("  `---'  "),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(photo, info_layout[0]);

        let text_info = match &self.state {
            State::Loading => Paragraph::new("Bilgiler Yükleniyor...").alignment(Alignment::Left),
            State::Error(e) => Paragraph::new(format!("Hata: {}", e))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Left),
            State::Loaded(data) => Paragraph::new(vec![
                Spans::from(format!("Adı: {}", data.ad)),
                Spans::from(format!("Soyadı: {}", data.soyad)),
                Spans::from(format!("Kimlik Numarası: {}", data.tc_kimlik)),
            ])
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().fg(Color::White)),
            ),
        };

        let centered_text_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Min(1),
                Constraint::Percentage(25),
            ])
            .split(info_layout[1]);
        frame.render_widget(text_info, centered_text_area[1]);

        if let State::Loaded(data) = &self.state {
            let email_block = Block::default()
                .borders(Borders::ALL)
                .title(Spans::from(vec![Span::styled(
                    " İletişim Bilgileri ",
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )]));
            frame.render_widget(email_block.clone(), chunks[1]);

            let email_inner = email_block.inner(chunks[1]);
            let mut email_content = Vec::new();
            for item in &data.iletisim {
                let status = match item.TercihEdilenMi {
                    Some(true) => Span::styled(
                        " [Tercih Edilen]",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    _ => Span::raw(""),
                };
                let verified = match item.DogrulandimiStr.as_deref() {
                    Some("Evet") => {
                        Span::styled(" (Doğrulandı)", Style::default().fg(Color::Green))
                    }
                    _ => Span::raw(""),
                };
                email_content.push(Spans::from(vec![
                    Span::styled(
                        format!("{}: ", item.IletisimTipi),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(item.Telefon_Mail.clone(), Style::default().fg(Color::Cyan)),
                    status,
                    verified,
                ]));
            }
            frame.render_widget(
                Paragraph::new(email_content).wrap(Wrap { trim: true }),
                email_inner,
            );

            let address_block = Block::default()
                .borders(Borders::ALL)
                .title(Spans::from(vec![Span::styled(
                    " Adres Bilgileri ",
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )]));
            frame.render_widget(address_block.clone(), chunks[2]);

            let address_inner = address_block.inner(chunks[2]);
            let mut address_content = Vec::new();
            for item in &data.adresler {
                let location = match (&item.ilce, &item.il) {
                    (Some(ilce), Some(il)) => format!(" ({}/{})", ilce, il),
                    _ => String::new(),
                };
                address_content.push(Spans::from(vec![
                    Span::styled(
                        format!("{}: ", item.AdresTipi),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(item.Adres.clone()),
                    Span::styled(location, Style::default().fg(Color::DarkGray)),
                ]));
            }
            frame.render_widget(
                Paragraph::new(address_content).wrap(Wrap { trim: true }),
                address_inner,
            );
        }
    }

    pub fn handle_event(&mut self, event: Event) -> PageAction {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('r') {
                if matches!(self.state, State::Loaded(_) | State::Error(_)) {
                    self.fetch_data();
                }
            }
        }
        PageAction::None
    }
}
