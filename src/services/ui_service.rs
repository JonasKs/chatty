use crate::app_state::{self, AppState, Message, MessageSender, Mode};

use super::{
    chat_service::Action,
    event_service::{Event, EventService},
};
use bytes::Bytes;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    terminal::{self as crossterm_terminal, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    text::Line,
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
use tracing::info;
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

pub struct UiService {
    action_sender: UnboundedSender<Action>,
    app_state: AppState,
    terminal_sender: Sender<Bytes>,
    gpt_role: String,
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

        let terminal_style = match self.app_state.current_mode {
            Mode::Terminal => Style::default().cyan(),
            Mode::Chat => Style::default(),
        };

        let chat_box_style = match self.app_state.current_mode {
            Mode::Terminal => Style::default(),
            Mode::Chat => Style::default().cyan(),
        };

        let chat_input_style = match self.app_state.current_mode {
            Mode::Terminal => Style::default(),
            Mode::Chat => match self.app_state.disable_chat {
                true => Style::default().gray(),
                false => Style::default().cyan(),
            },
        };

        // Terminal code. We don't need to do much here, everything is handled by the widget pretty much
        let pseudo_terminal = PseudoTerminal::new(screen);
        frame.render_widget(
            pseudo_terminal.block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(terminal_style)
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

        let chat_block = Block::default()
            .title("GPT")
            .borders(Borders::ALL)
            .border_style(chat_box_style);

        let styled_messages: Vec<Line> = self
            .app_state
            .chat_history
            .iter()
            .flat_map(|message| {
                message
                    .style(chat_layout[0].width.into(), self.gpt_role.clone())
                    .into_iter()
            })
            .collect();

        frame.render_widget(
            Paragraph::new(styled_messages)
                .wrap(Wrap { trim: true })
                .block(chat_block)
                .scroll((self.app_state.scroll, 0)),
            chat_layout[0],
        );

        // Chat box where we type shit
        let chat_box_style = match self.app_state.disable_chat {
            true => Style::default().gray(),
            false => Style::default(),
        };

        let default_throbber = throbber_widgets_tui::Throbber::default()
            .label("Loading...")
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

        let chatbox_widget = match self.app_state.disable_chat {
            true => Paragraph::new(self.app_state.user_chat_to_send_to_gpt.clone())
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_style(chat_input_style)
                        .title(default_throbber)
                        .style(chat_box_style),
                )
                .alignment(if self.app_state.disable_chat {
                    Alignment::Center
                } else {
                    Alignment::Left
                }),
            false => Paragraph::new(self.app_state.user_chat_to_send_to_gpt.clone())
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_style(chat_input_style)
                        .title("GPT")
                        .style(chat_box_style),
                )
                .alignment(if self.app_state.disable_chat {
                    Alignment::Center
                } else {
                    Alignment::Left
                }),
        };

        frame.render_widget(chatbox_widget, chat_layout[1]);
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
                Event::AIStreamResponse(stream) => {
                    if let Some(last_message) = self.app_state.chat_history.last_mut() {
                        match last_message.sender {
                            MessageSender::User => {
                                self.app_state.chat_history.push(Message {
                                    sender: app_state::MessageSender::Assistant,
                                    message: stream,
                                });
                            }
                            MessageSender::Assistant => {
                                last_message.message.push_str(&stream);
                            }
                        }
                    } else {
                        // No last message, so it has to be assistant role change
                        self.app_state.chat_history.push(Message {
                            sender: app_state::MessageSender::Assistant,
                            message: stream,
                        });
                    }
                }
                Event::AIReasoning(is_finished_reasoning) => {
                    match is_finished_reasoning {
                        true => {
                            self.app_state.disable_chat = false;
                        }
                        false => {
                            self.app_state.disable_chat = true;
                        }
                    };
                }
                Event::Tick => self.app_state.tick(),
                Event::Quit => self.app_state.quit(),
                Event::ChangeMode => self.app_state.change_mode(),
                Event::Resize(columns, rows) => {
                    let adjusted_width = (columns as f32 * 0.57).floor() as u16;
                    parser.write().await.set_size(rows - 5, adjusted_width)
                }
                Event::ScrollUp => self.app_state.scroll = self.app_state.scroll.saturating_add(1),
                Event::ScrollDown => {
                    self.app_state.scroll = self.app_state.scroll.saturating_sub(1)
                }
                Event::ScrollUpTerminal => {
                    self.app_state.terminal_scroll = self.app_state.terminal_scroll.saturating_add(1)
                }
                Event::ScrollDownTerminal => {
                    self.app_state.terminal_scroll = self.app_state.terminal_scroll.saturating_sub(1)
                }

                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        match self.app_state.current_mode {
                            Mode::Terminal => {
                                self.terminal_sender
                                    .send(Bytes::from(vec![3]))
                                    .await
                                    .unwrap();
                            }
                            Mode::Chat => {
                                // handle
                            }
                        }
                    }
                    KeyCode::Char(char) => match self.app_state.current_mode {
                        Mode::Chat => {
                            if (!self.app_state.disable_chat) {
                                self.app_state
                                    .user_chat_to_send_to_gpt
                                    .push_str(&char.to_string())
                            }
                        }
                        Mode::Terminal => self
                            .terminal_sender
                            .send(Bytes::from(char.to_string().into_bytes()))
                            .await
                            .unwrap(),
                    },
                    KeyCode::Enter => match self.app_state.current_mode {
                        Mode::Terminal => {
                            self.terminal_sender
                                .send(Bytes::from(vec![13u8]))
                                .await
                                .unwrap();
                        }
                        Mode::Chat => {
                            if self.app_state.user_chat_to_send_to_gpt == "/clear" {
                                self.app_state.user_chat_to_send_to_gpt.clear();
                                self.app_state.chat_history.clear();
                                self.app_state.scroll = 0;
                                self.action_sender.send(Action::Clear).unwrap();
                                self.app_state.terminal_context.lock().await.clear();
                                continue;
                            } else if self.app_state.user_chat_to_send_to_gpt == "/network" {
                                self.app_state.user_chat_to_send_to_gpt.clear();
                                self.app_state.chat_history.clear();
                                self.app_state.scroll = 0;
                                self.app_state.terminal_context.lock().await.clear();
                                self.action_sender.send(Action::NetworkEngineer).unwrap();
                                self.gpt_role = "Network Engineer".to_string();
                                continue;
                            } else if self.app_state.user_chat_to_send_to_gpt == "/linux" {
                                self.app_state.user_chat_to_send_to_gpt.clear();
                                self.app_state.chat_history.clear();
                                self.app_state.scroll = 0;
                                self.app_state.terminal_context.lock().await.clear();
                                self.gpt_role = "Linux Engineer".to_string();
                                self.action_sender.send(Action::LinuxEngineer).unwrap();
                                continue;
                            }
                            match self.app_state.terminal_has_been_active {
                                true => {
                                    tracing::warn!(
                                        "{}",
                                        self.app_state.terminal_context.lock().await.clone()
                                    );
                                    self.action_sender
                                        .send(Action::AiRequest(format!(
                                            "This is my terminal output: \n\n ```\n{}\n```\n\n{}",
                                            self.app_state.terminal_context.lock().await.clone(),
                                            self.app_state.user_chat_to_send_to_gpt,
                                        )))
                                        .unwrap();
                                }
                                false => {
                                    self.action_sender
                                        .send(Action::AiRequest(
                                            self.app_state.user_chat_to_send_to_gpt.clone(),
                                        ))
                                        .unwrap();
                                }
                            }

                            // save chat to history
                            self.app_state.chat_history.push(Message {
                                sender: app_state::MessageSender::User,
                                message: self.app_state.user_chat_to_send_to_gpt.clone(),
                            });
                            self.app_state.terminal_has_been_active = false;
                            self.app_state.user_chat_to_send_to_gpt.clear();
                            self.app_state.disable_chat = true;
                        }
                    },
                    KeyCode::Backspace => match self.app_state.current_mode {
                        Mode::Chat => {
                            self.app_state.user_chat_to_send_to_gpt.pop();
                        }
                        Mode::Terminal => {
                            self.terminal_sender
                                .send(Bytes::from(vec![8u8]))
                                .await
                                .unwrap();
                        }
                    },
                    KeyCode::Up => match self.app_state.current_mode {
                        Mode::Terminal => {
                            // Handle up arrow key in Terminal mode (e.g., navigate through command history)
                            self.terminal_sender
                                .send(Bytes::from(vec![27, 91, 65]))
                                .await
                                .unwrap(); // ASCII for ESC[A (Up Arrow)
                        }
                        Mode::Chat => {
                            // Handle up arrow key in Chat mode (e.g., navigate through chat history)
                        }
                    },
                    KeyCode::Down => match self.app_state.current_mode {
                        Mode::Terminal => {
                            // Handle down arrow key in Terminal mode
                            self.terminal_sender
                                .send(Bytes::from(vec![27, 91, 66]))
                                .await
                                .unwrap(); // ASCII for ESC[B (Down Arrow)
                        }
                        Mode::Chat => {
                            // Handle down arrow key in Chat mode
                        }
                    },
                    KeyCode::Left => match self.app_state.current_mode {
                        Mode::Terminal => {
                            // Handle left arrow key in Terminal mode
                            self.terminal_sender
                                .send(Bytes::from(vec![27, 91, 68]))
                                .await
                                .unwrap(); // ASCII for ESC[D (Left Arrow)
                        }
                        Mode::Chat => {
                            // Handle left arrow key in Chat mode
                        }
                    },
                    KeyCode::Right => match self.app_state.current_mode {
                        Mode::Terminal => {
                            // Handle right arrow key in Terminal mode
                            self.terminal_sender
                                .send(Bytes::from(vec![27, 91, 67]))
                                .await
                                .unwrap(); // ASCII for ESC[C (Right Arrow)
                        }
                        Mode::Chat => {
                            // Handle right arrow key in Chat mode
                        }
                    },
                    KeyCode::Delete => match self.app_state.current_mode {
                        Mode::Terminal => {
                            // Handle delete key in Terminal mode
                            self.terminal_sender
                                .send(Bytes::from(vec![27, 91, 51, 126]))
                                .await
                                .unwrap(); // ASCII for ESC[3~ (Delete)
                        }
                        Mode::Chat => {
                            // Handle delete key in Chat mode
                            // You might want to implement similar to backspace or clear a specific part of the text
                        }
                    },
                    KeyCode::Esc => {
                        // Handle escape key globally, maybe switch mode or clear input
                        match self.app_state.current_mode {
                            Mode::Chat => {
                                self.app_state.user_chat_to_send_to_gpt.clear();
                            }
                            Mode::Terminal => {
                                self.terminal_sender
                                    .send(Bytes::from(vec![27u8]))
                                    .await
                                    .unwrap(); // ASCII for ESC
                            }
                        }
                    }
                    KeyCode::Tab => {
                        // Handle Tab key, perhaps for auto-completion or cycling through options
                        match self.app_state.current_mode {
                            Mode::Chat => {
                                // Handle tab in chat, if applicable
                            }
                            Mode::Terminal => {
                                self.terminal_sender
                                    .send(Bytes::from(vec![9u8]))
                                    .await
                                    .unwrap(); // ASCII for Tab
                            }
                        }
                    }
                    _ => {}
                },
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
            gpt_role: "".into(),
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
