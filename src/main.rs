use anyhow::Result;
use crossterm::event;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::backend::Backend;
use ratatui::layout::Alignment;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};

mod client;
mod pages;
mod router;

use router::{Route, Router};

use crate::pages::Page;
use crate::router::RouterResult;

struct App<B: Backend> {
    router: Router<B>,
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let router = Router::new(Route::Login);

    let mut app: App<_> = App { router };

    loop {
        terminal.draw(|f| {
            let size = f.size();

            const MIN_WIDTH: u16 = 50;
            const MIN_HEIGHT: u16 = 11;

            if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
                let warning = Paragraph::new("Terminal çok küçük! Lütfen büyütün.")
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center);
                f.render_widget(warning, size);
                return;
            }

            app.router.current_page.render(f);
        })?;

        if event::poll(Duration::from_millis(200))? {
            let e = event::read()?;

            if let event::Event::Key(key) = e {
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
            }

            let action = app.router.current_page.handle_event(e);

            match app.router.handle_action(action) {
                RouterResult::Exit => break,
                _ => {}
            }
        }

        let tick_action = app.router.current_page.tick();
        match app.router.handle_action(tick_action) {
            RouterResult::Exit => break,
            _ => {}
        }
    }
    drop(terminal);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
