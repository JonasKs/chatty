use std::error;

use bytes::Bytes;
use tokio::sync::mpsc::Sender;

use crate::widgets::terminal::TerminalWidget;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub terminal_widget: TerminalWidget,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(terminal_widget: TerminalWidget) -> Self {
        Self {
            running: true,
            terminal_widget,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
