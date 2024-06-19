use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::{Arc, Mutex};
use terminal_ai_ops::app::{App, AppResult};
use terminal_ai_ops::event::{Event, EventHandler};
use terminal_ai_ops::handler::handle_key_events;
use terminal_ai_ops::terminal_utils;
use terminal_ai_ops::tui::Tui;
use tokio::sync::mpsc::{channel, Sender};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;

    let terminal_context = Arc::new(Mutex::new(String::new()));
    let chat_messages = Arc::new(Mutex::new(Vec::new()));
    let (chat_sender, mut chat_receiver) = channel::<String>(32);

    let (parser, sender_to_terminal) = terminal_utils::new(&terminal, terminal_context.clone());
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Create an application.
    let mut app = App::new(
        sender_to_terminal,
        terminal_context,
        chat_messages,
        chat_sender,
        chat_receiver,
    );

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

// async fn setup_chat_channel(chat_messages: Arc<Mutex<Vec<String>>>) -> Sender<String> {
// let (chat_sender, mut chat_receiver) = channel::<String>(32);

//     tokio::spawn(async move {
//         chat_receiver.try_recv().ok();
//         while let Some(message) = chat_receiver.recv().await {
//             chat_messages[l]
//             println!("Received message: {}", message);
//         }
//     });

//     chat_sender
// }
