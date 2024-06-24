use crate::app_state::{self, AppState, Mode};

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
    style::{Modifier, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io::{self, Stdout},
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
        // Root layout which has a footer spanning the entire screen
        let root_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Max(1)])
            .split(frame.size());
        // Outer layout, which is inside the root_layout, on top of the footer. This is essentially the area we use
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(60), // Terminal
                Constraint::Percentage(40), // Chat
            ])
            .split(root_box[0]);
        let footer_text = "<CTRL>q to exit | <CTRL>b to change mode".to_string();
        let footer = Paragraph::new(footer_text)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
            .alignment(Alignment::Center);
        frame.render_widget(footer, root_box[1]);

        // Terminal code. We don't need to do much here, everything is handled by the widget pretty much
        let terminal_border_style = match self.app_state.current_mode {
            Mode::Terminal => Style::default().cyan(),
            Mode::Chat => Style::default(),
        };
        let pseudo_terminal = PseudoTerminal::new(screen)
            .scroll((self.app_state.terminal_scroll, 0)); // Added scroll functionality based on AppState::terminal_scroll
        frame.render_widget(
            pseudo_terminal.block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(terminal_border_style)
                    .title("Terminal"),
            ),
            outer_layout[0],
        );

        // Chat code - here we need to create our own layout, with two boxes inside
        let chat_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(30), // Chat history
                Constraint::Min(3),   // Chat input
            ])
            .split(outer_layout[1]);
        let chat_border_style = match self.app_state.current_mode {
            Mode::Terminal => Style::default(),
            Mode::Chat => Style::default().cyan(),
        };
        let chat = Paragraph::new(self.app_state.ai_response.to_string())
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(chat_border_style)
                    .title("GPT"),
            )
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(chat, chat_layout[0]);
        frame.render_widget(
            Paragraph::new(self.app_state.user_chat_to_send_to_gpt.to_string()).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(chat_border_style)
                    .title("Chat"),
            ),
            chat_layout[1],
        );
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
                Event::ChangeMode => self.app_state.change_mode(),
                Event::Key(key) => match key.code {
                    KeyCode::Char(char) => match self.app_state.current_mode {
                        Mode::Chat => self
                            .app_state
                            .user_chat_to_send_to_gpt
                            .push_str(&char.to_string()),
                        Mode::Terminal => self
                            .terminal_sender
                            .send(Bytes::from(char.to_string().into_bytes()))
                            .await
                            .unwrap(),
                    },
                    KeyCode::Enter => self
                        .action_sender
                        .send(Action::AiRequest(
                            "tell me a two paragraph story".to_string(),
                        ))
                        .unwrap(),
                    KeyCode::Up => {
                        if self.app_state.terminal_scroll > 0 {
                            self.app_state.terminal_scroll -= 1; // Handle scroll up
                        }
                    }
                    KeyCode::Down => {
                        self.app_state.terminal_scroll += 1; // Handle scroll down
                    }
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
