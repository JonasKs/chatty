use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

enum Mode {
    Terminal,
    Chat,
}

pub struct AppState {
    pub running: bool,
    pub current_mode: Mode,

    pub terminal_context: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(terminal_context: Arc<Mutex<String>>) -> Self {
        Self {
            running: true,
            current_mode: Mode::Terminal,
            terminal_context,
        }
    }

    pub fn change_mode(&mut self) {
        match self.current_mode {
            Mode::Chat => self.current_mode = Mode::Terminal,
            Mode::Terminal => self.current_mode = Mode::Chat,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
