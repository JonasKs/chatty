use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use terminal_ai_ops::app_state::AppState;
use terminal_ai_ops::services::chat_service::ChatService;
use terminal_ai_ops::services::event_service::EventService;
use terminal_ai_ops::services::ui_service::UiService;
use terminal_ai_ops::services::{chat_service::Action, event_service::Event};
use terminal_ai_ops::terminal_utils;
use tokio::sync::mpsc::{self};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let (action_sender, mut action_receiver) = mpsc::unbounded_channel::<Action>();
    let (event_sender, event_receiver) = mpsc::unbounded_channel::<Event>();

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();
    let terminal_context = Arc::new(Mutex::new(String::new()));
    let app_state = AppState::new(terminal_context.clone());
    let mut event_service = EventService::new(event_receiver);
    let (parser, terminal_sender) = terminal_utils::new(&terminal, terminal_context.clone());
    let mut ui_service = UiService::new(
        action_sender,
        app_state,
        &mut terminal,
        terminal_sender.clone(),
    );

    let chat_service = ChatService::new();
    tokio::spawn(async move { chat_service.start(event_sender, &mut action_receiver).await });
    ui_service
        .start(&mut terminal, &mut event_service, parser)
        .await;
    ui_service.exit(&mut terminal);
}
