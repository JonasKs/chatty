use crate::app_state::AppState;

use super::{
    chat_service::Action,
    event_service::{Event, EventService},
};
use bytes::Bytes;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    terminal::{self as crossterm_terminal, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io::{self, Stderr, Stdout},
    panic,
    sync::Arc,
};
use tokio::sync::{
    mpsc::{Sender, UnboundedSender},
    RwLock,
};
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

pub struct UiService {
    action_sender: UnboundedSender<Action>,
    app_state: AppState,
    terminal_sender: Sender<Bytes>,
}

impl UiService {
    /// Renders the user interface widgets.
    pub fn render(&self, frame: &mut Frame, screen: &Screen) {
        let root_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Max(1)])
            .split(frame.size());
        let explanation = "Press <CTRL>q to exit".to_string();
        let explanation = Paragraph::new(explanation)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
            .alignment(Alignment::Center);
        frame.render_widget(explanation, root_box[1]);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(root_box[0]);

        let pseudo_terminal = PseudoTerminal::new(screen);
        frame.render_widget(
            pseudo_terminal.block(Block::default().borders(Borders::RIGHT)),
            outer_layout[0],
        );

        let chat = Paragraph::new(self.app_state.ai_response.to_string())
            .wrap(Wrap { trim: false })
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(chat, outer_layout[1]);
    }

    pub async fn start(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        event_service: &mut EventService,
        parser: Arc<RwLock<vt100::Parser>>,
    ) {
        while self.app_state.running {
            let screen = parser.read().await.screen().clone();
            terminal.draw(|frame| self.render(frame, &screen)).unwrap();
            // Handle events
            match event_service.next().await.unwrap() {
                Event::AIStreamResponse(stream) => self.app_state.ai_response.push_str(&stream),
                Event::Tick => self.app_state.tick(),
                Event::Quit => self.app_state.quit(),
                Event::Key(key) => match key.code {
                    KeyCode::Char(char) => self
                        .terminal_sender
                        .send(Bytes::from(char.to_string().into_bytes()))
                        .await
                        .unwrap(),
                    KeyCode::Enter => self
                        .action_sender
                        .send(Action::AiRequest(
                            "tell me a two paragraph story".to_string(),
                        ))
                        .unwrap(),
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

impl UiService {
    pub fn new(
        action_sender: UnboundedSender<Action>,
        app_state: AppState,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        terminal_sender: Sender<Bytes>,
    ) -> Self {
        crossterm_terminal::enable_raw_mode().unwrap();
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture).unwrap();

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset();
            panic_hook(panic);
        }));

        terminal.hide_cursor().unwrap();
        terminal.clear().unwrap();

        Self {
            action_sender,
            app_state,
            terminal_sender,
        }
    }

    pub fn reset() {
        crossterm_terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    }

    pub fn exit(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
        Self::reset();
        terminal.show_cursor().unwrap();
    }
}
