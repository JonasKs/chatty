use crate::{config, services::event_service::Event};

use async_openai::{
    config::AzureConfig,
    error::OpenAIError,
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
    NetworkEngineer,
    LinuxEngineer,
}

pub struct ChatService {
    client: Client<AzureConfig>,
    previous_messages: Vec<ChatCompletionRequestMessage>,
}

impl ChatService {
    pub fn new() -> Self {
        let client = Client::with_config(config::get_config());
        let system_prompt = ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a general purpose programmer. The user that will chat with you appreciates short answers when possible. If you want to share commands, there is not reason to explain what all the commands does in detail, a short sentence maximum. The user will most likely send you questions about his terminal output. When answering, be consice!!

                If you're asked what IP a demo device has, respond with 10.82.189.130.")
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
                Action::NetworkEngineer => {
                    // Clear all messages except first, which is the system message
                    self.previous_messages.drain(1..);
                    tracing::info!("Changing prompt to network engineer");
                    if let Some(first) = self.previous_messages.first_mut() {
                        *first = ChatCompletionRequestSystemMessageArgs::default()
                            .content("You are an advanced network assistant. Your role is to assist users in resolving their network challenges, improving their config, and understanding networking concepts. Follow these guidelines: - Analyze the config provided by the user for errors, inefficiencies, or areas that require explanation. - If in doubt of which operating system, ask (e.g. Cisco IOS XE or IOS XR) - Offer solutions, optimizations, or explanations that are directly relevant to the user's request. -Ensure that your responses are in the same operating system language that the user is using. - Provide clear, step-by-step guidance when explaining solutions or concepts. - Encourage best practices in networking, such as clean code principles, commenting, and efficient algorithms. - Structure your response and questions using markdown headers to organize different aspects of coding assistance. For example: ### <your text> <feedback> - Always respond in the same language as the user writes to you in (If the user writes in Norwegian, respond in Norwegian). Remember to adapt your guidance to the user's level of expertise, from beginner to advanced.")
                            .build()
                            .unwrap().into();
                    }
                    tracing::info!(
                        "First message: {:?}",
                        self.previous_messages.first().unwrap()
                    );
                    event_sender
                        .send(Event::AIStreamResponse("Hi! I'm your personal Network assistant. I'm can see your terminal, so feel free to ask questions!".into()))
                        .unwrap();
                }
                Action::LinuxEngineer => {
                    // Clear all messages except first, which is the system message
                    self.previous_messages.drain(1..);
                    tracing::info!("Changing prompt to linux engineer");
                    if let Some(first) = self.previous_messages.first_mut() {
                        *first = ChatCompletionRequestSystemMessageArgs::default()
                            .content("You are a Linux Security Expert. The user that will chat with you appreciates short answers when possible. If you want to share commands, there is not reason to explain what all the commands does in detail, a short sentence maximum. The user will most likely send you questions about his terminal output. When answering, be consice!!")
                            .build()
                            .unwrap().into();
                    }
                    tracing::info!("First message: {:?}", self.previous_messages);
                    event_sender
                        .send(Event::AIStreamResponse("Hi! I'm your personal Linux assistant. I'm can see your terminal, so feel free to ask questions!".into()))
                        .unwrap();
                }
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
                            Err(err) => match err {
                                OpenAIError::Reqwest(reqerror) => {
                                    tracing::warn!("reqwest error {:?}", reqerror);
                                }
                                OpenAIError::StreamError(reqerror) => {
                                    tracing::warn!("stream error {:?}", reqerror);
                                }
                                OpenAIError::JSONDeserialize(reqerror) => {
                                    tracing::warn!("JSON des error {:?}", reqerror);
                                }
                                OpenAIError::FileReadError(reqerror) => {
                                    tracing::warn!("File readerror {:?}", reqerror);
                                }
                                OpenAIError::ApiError(reqerror) => {
                                    tracing::warn!("API error readerror {:?}", reqerror);
                                }
                                OpenAIError::InvalidArgument(reqerror) => {
                                    tracing::warn!("Invalid arg error readerror {:?}", reqerror);
                                }
                                _ => {
                                    tracing::warn!("{:?}", err);
                                }
                            },
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
