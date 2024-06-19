use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use terminal_ai_ops::app::{App, AppResult};
use terminal_ai_ops::event::{Event, EventHandler};
use terminal_ai_ops::handler::handle_key_events;
use terminal_ai_ops::terminal_utils;
use terminal_ai_ops::tui::Tui;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let (parser, sender_to_terminal) = terminal_utils::new(&terminal);
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Create an application.
    let mut app = App::new(sender_to_terminal);

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app, parser.clone())?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
