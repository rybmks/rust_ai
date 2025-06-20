use genai::{
    Client,
    chat::{ChatMessage, ChatRequest, printer::print_chat_stream},
};

#[tokio::main]
async fn main() {
    let client = Client::default();
    let chat_message = ChatMessage::user("Make a very short essay about beer");
    let chat_req = ChatRequest::new(vec![chat_message]);

    let model = "gpt-3.5-turbo";
    let chat_res = client
        .exec_chat_stream(model, chat_req.clone(), None)
        .await
        .unwrap();
    let response = print_chat_stream(chat_res, None).await.unwrap();
    chat_req.append_message(ChatMessage::assistant(response));
}
