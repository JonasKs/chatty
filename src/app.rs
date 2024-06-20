use std::{error, sync::Arc};

use async_openai::{config::AzureConfig, Client};
use bytes::Bytes;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub sender_to_terminal: Sender<Bytes>,
    pub terminal_context: Arc<Mutex<String>>,
    pub chat_messages: Arc<Mutex<Vec<String>>>,
    pub chat_sender: Sender<String>,
    pub chat_receiver: Receiver<String>,
    pub client: Arc<Mutex<Client<AzureConfig>>>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        sender_to_terminal: Sender<Bytes>,
        terminal_context: Arc<Mutex<String>>,
        chat_messages: Arc<Mutex<Vec<String>>>,
        chat_sender: Sender<String>,
        chat_receiver: Receiver<String>,
        client: Arc<Mutex<Client<AzureConfig>>>,
    ) -> Self {
        Self {
            running: true,
            sender_to_terminal,
            terminal_context,
            chat_messages,
            chat_sender,
            chat_receiver,
            client,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
