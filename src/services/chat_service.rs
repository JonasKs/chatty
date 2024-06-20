use crate::{config, services::event_service::Event};

use async_openai::{
    config::AzureConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
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
        event_sender: mpsc::UnboundedSender<Event>,
        action_receiver: &mut mpsc::UnboundedReceiver<Action>,
    ) {
        // Spawn a tokio task that listens for actions from the action_receiver.
        // Communicate back to the UI by sending events to the event_sender.
        // Inspiration: https://github.com/dustinblackman/oatmeal/blob/a6148b2474778698f7b261aa549dcbda439e2060/src/domain/services/actions.rs#L239
        while let Some(action) = action_receiver.recv().await {
            match action {
                Action::AiRequest(message) => {
                    // Process the AI request...
                    let new_message = ChatCompletionRequestUserMessageArgs::default()
                        .content(message)
                        .build()
                        .unwrap();

                    let request = CreateChatCompletionRequestArgs::default()
                        .model("gpt-4o")
                        .max_tokens(512u16)
                        .messages(vec![new_message.into()])
                        .build()
                        .unwrap();

                    let mut stream = self.client.chat().create_stream(request).await.unwrap();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                for chat_choice in response.choices.iter() {
                                    if let Some(ref content) = chat_choice.delta.content {
                                        event_sender
                                            .send(Event::AIStreamResponse(content.into()))
                                            .unwrap();
                                        // chat_messages.push(content.into())
                                    }
                                }
                            }
                            Err(err) => {
                                println!("{}", err)
                            }
                        }
                    }
                }
            }
        }
    }
}
