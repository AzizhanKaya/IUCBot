use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use std::{io, time::Duration};

pub mod client;
pub mod pages;
pub mod router;

use pages::PageAction;
use router::{Route, Router};

struct App<B: Backend> {
    router: Router<B>,
}

#[tokio::main]
async fn main() -> Result<()> {
    simplelog::WriteLogger::init(
        log::LevelFilter::Info,
        simplelog::Config::default(),
        std::fs::File::create("iucbot.log")?,
    )?;
    log::info!("Starting IUCBot...");

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let router: Router<CrosstermBackend<std::io::Stdout>> = Router::new(Route::Login).await;
    let mut app = App { router };

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

        let mut page_action = PageAction::None;

        if event::poll(Duration::from_millis(16))? {
            let e = event::read()?;
            if let Event::Key(key) = e {
                if key.kind == event::KeyEventKind::Press {
                    page_action = app.router.current_page.handle_event(Event::Key(key));
                }
            } else if let Event::Mouse(mouse) = e {
                page_action = app.router.current_page.handle_event(Event::Mouse(mouse));
            } else if let Event::Resize(x, y) = e {
                page_action = app.router.current_page.handle_event(Event::Resize(x, y));
            }
        }

        if matches!(page_action, PageAction::None) {
            page_action = app.router.current_page.update();
        }

        if let PageAction::Quit = page_action {
            break;
        } else if !matches!(page_action, PageAction::None) {
            app.router.handle_action(page_action).await;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
