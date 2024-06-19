use std::error;

use bytes::Bytes;
use tokio::sync::mpsc::Sender;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub sender_to_terminal: Sender<Bytes>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(sender_to_terminal: Sender<Bytes>) -> Self {
        Self {
            running: true,
            sender_to_terminal,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
