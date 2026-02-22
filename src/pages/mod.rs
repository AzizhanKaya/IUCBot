use anyhow::Result;
use crossterm::event::Event;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::layout::Rect;

use crate::router::PageAction;

pub mod dashboard;
pub mod login;

pub trait Page<B: Backend> {
    fn render(&mut self, terminal: &mut Frame<B>);
    fn handle_event(&mut self, event: Event) -> PageAction;
    fn tick(&mut self) -> PageAction {
        PageAction::None
    }
}

trait Contains {
    fn contains(&self, x: u16, y: u16) -> bool;
}

impl Contains for Rect {
    fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub static STORE: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);
