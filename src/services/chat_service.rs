use crate::{config, services::event_service::Event};

use async_openai::{config::AzureConfig, Client};
use tokio::sync::mpsc;

pub enum Action {
    AiRequest(String),
}

pub struct ChatService {
    client: Client<AzureConfig>,
}

impl ChatService {
    pub fn new() -> Self {
        let client = Client::with_config(config::get_config());
        Self { client }
    }

    pub async fn start(
        &self,
        _event_sender: mpsc::UnboundedSender<Event>,
        _action_receiver: &mut mpsc::UnboundedReceiver<Action>,
    ) {
        // Spawn a tokio task that listens for actions from the action_receiver.
        // Communicate back to the UI by sending events to the event_sender.
        // Inspiration: https://github.com/dustinblackman/oatmeal/blob/a6148b2474778698f7b261aa549dcbda439e2060/src/domain/services/actions.rs#L239
        todo!()
    }
}
