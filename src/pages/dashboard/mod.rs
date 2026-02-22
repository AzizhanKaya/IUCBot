use std::rc::Rc;

use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::client;
use crate::client::obs::ObsClient;
use crate::pages::Page;
use crate::router::PageAction;

pub mod ders_alma;
pub mod ders_notlari;
pub mod ozluk_bilgileri;

use ders_alma::DersAlma;
use ders_notlari::DersNotlari;
use ozluk_bilgileri::OzlukBilgileri;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Tab {
    #[default]
    OzlukBilgileri,
    DersAlma,
    DersNotlari,
}

impl Tab {
    fn next(&self) -> Self {
        match self {
            Self::OzlukBilgileri => Self::DersAlma,
            Self::DersAlma => Self::DersNotlari,
            Self::DersNotlari => Self::OzlukBilgileri,
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::OzlukBilgileri => "Özlük Bilgileri",
            Self::DersAlma => "Ders Alma",
            Self::DersNotlari => "Ders Notları",
        }
    }
}

pub struct Dashboard {
    client: Rc<ObsClient>,
    tab: Tab,
    ozluk_bilgileri: OzlukBilgileri,
    ders_alma: DersAlma,
    ders_notlari: DersNotlari,
}

impl Dashboard {
    pub fn new(client: ObsClient) -> Self {
        let client = Rc::new(client);
        Self {
            client: client.clone(),
            tab: Tab::default(),
            ozluk_bilgileri: OzlukBilgileri::new(client.clone()),
            ders_alma: DersAlma::new(client.clone()),
            ders_notlari: DersNotlari::new(client.clone()),
        }
    }
}

impl<B: Backend> Page<B> for Dashboard {
    fn render(&mut self, frame: &mut Frame<B>) {
        let size = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(size);

        let sidebar_area = chunks[0];
        let content_area = chunks[1];

        let items: Vec<ListItem> = [Tab::OzlukBilgileri, Tab::DersAlma, Tab::DersNotlari]
            .iter()
            .map(|t| {
                let style = if *t == self.tab {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if *t == self.tab { "> " } else { "  " };
                ListItem::new(Spans::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(t.title(), style),
                ]))
            })
            .collect();

        let sidebar = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Menü (Tab)"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(sidebar, sidebar_area);

        match self.tab {
            Tab::OzlukBilgileri => self.ozluk_bilgileri.render(frame, content_area),
            Tab::DersAlma => self.ders_alma.render(frame, content_area),
            Tab::DersNotlari => self.ders_notlari.render(frame, content_area),
        }
    }

    fn handle_event(&mut self, event: Event) -> PageAction {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Tab => {
                    self.tab = self.tab.next();
                    return PageAction::None;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    return PageAction::Exit;
                }
                _ => {}
            }
        }

        match self.tab {
            Tab::OzlukBilgileri => self.ozluk_bilgileri.handle_event(event),
            Tab::DersAlma => self.ders_alma.handle_event(event),
            Tab::DersNotlari => self.ders_notlari.handle_event(event),
        }
    }

    fn tick(&mut self) -> PageAction {
        if self.tab == Tab::DersAlma {
            self.ders_alma.tick();
        }
        PageAction::None
    }
}
