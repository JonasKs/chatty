use std::sync::Arc;

use async_openai::{
    config::AzureConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
use tokio::sync::{mpsc::Sender, Mutex};

/// Function to call Azure AI
/// This function needs: Context of what exists in terminal and the chat input
/// Stream response and send the results to a channel
pub async fn call_ai(client: Arc<Mutex<Client<AzureConfig>>>, sender: Sender<String>) {
    // let terminal_context = app.terminal_context.lock().unwrap();
    // let mut chat_messages = app.chat_messages.lock().unwrap();
    let client = client.lock().await;

    let new_message = ChatCompletionRequestUserMessageArgs::default()
        .content("Give me an example response")
        .build()
        .unwrap();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .max_tokens(512u16)
        .messages(vec![new_message.into()])
        .build()
        .unwrap();

    let mut stream = client.chat().create_stream(request).await.unwrap();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                    if let Some(ref content) = chat_choice.delta.content {
                        sender.send(content.into()).await.unwrap();
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
