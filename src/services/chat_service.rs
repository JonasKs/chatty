use crate::{config, services::event_service::Event};

use async_openai::{
    config::AzureConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::info;

pub enum Action {
    AiRequest(String),
    Clear,
}

pub struct ChatService {
    client: Client<AzureConfig>,
    previous_messages: Vec<ChatCompletionRequestMessage>,
}

impl ChatService {
    pub fn new() -> Self {
        let client = Client::with_config(config::get_config());
        let system_prompt = ChatCompletionRequestSystemMessageArgs::default()
                // .content("You are a network administrator. The user will send you questions about his terminal output, always give LONG answers(a paragraph + section of the example config if)! Be consice!!.")
                .content("You are to repeat this exact sentence 3 times, with two line breaks, prefix it with the number: Hello per anders you are very beautiful today but not as beautiful as your dear friend Jonas who is extreamly beautiful today dont you think or what??????")
                .build()
                .unwrap();
        Self {
            client,
            previous_messages: vec![system_prompt.into()],
        }
    }

    pub async fn start(
        &mut self,
        event_sender: mpsc::UnboundedSender<Event>,
        action_receiver: &mut mpsc::UnboundedReceiver<Action>,
    ) {
        // Spawn a tokio task that listens for actions from the action_receiver.
        // Communicate back to the UI by sending events to the event_sender.
        // Inspiration: https://github.com/dustinblackman/oatmeal/blob/a6148b2474778698f7b261aa549dcbda439e2060/src/domain/services/actions.rs#L239
        while let Some(action) = action_receiver.recv().await {
            match action {
                Action::Clear => {
                    // Clear all messages except first, which is the system message
                    self.previous_messages.drain(1..);
                }
                Action::AiRequest(message) => {
                    // Process the AI request...
                    let new_message = ChatCompletionRequestUserMessageArgs::default()
                        .content(message)
                        .build()
                        .unwrap();

                    // Push the message into the history
                    self.previous_messages.push(new_message.into());

                    let request = CreateChatCompletionRequestArgs::default()
                        .model("gpt-4o")
                        .max_tokens(512u16)
                        .messages(self.previous_messages.clone())
                        .build()
                        .unwrap();

                    let mut stream = self.client.chat().create_stream(request).await.unwrap();

                    let mut assistant_response = String::new();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                for chat_choice in response.choices.iter() {
                                    if let Some(ref content) = chat_choice.delta.content {
                                        assistant_response.push_str(content);
                                        info!("{}", content);
                                        event_sender
                                            .send(Event::AIStreamResponse(content.into()))
                                            .unwrap();
                                    }

                                    // send event to the UI to indicate if the AI is reasoning or not
                                    event_sender
                                        .send(Event::AIReasoning(
                                            chat_choice.finish_reason.is_some(),
                                        ))
                                        .unwrap();
                                }
                                tracing::info!("{:?}", response)
                            }
                            Err(err) => {
                                println!("{}", err)
                            }
                        }
                    }
                    if !assistant_response.is_empty() {
                        tracing::info!(assistant_response);
                        let ai_response = ChatCompletionRequestAssistantMessageArgs::default()
                            .content(assistant_response)
                            .build()
                            .unwrap();
                        self.previous_messages.push(ai_response.into());
                    }
                }
            }
        }
    }
}
