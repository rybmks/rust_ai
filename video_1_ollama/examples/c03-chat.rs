use futures::StreamExt;
use futures::stream;
use ollama::Result;
use ollama::consts::*;
use ollama::generate::gen_stream_print;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::chat::MessageRole;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::completion::request::GenerationRequest;
use simple_fs::ensure_file_dir;
use simple_fs::save_json;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    //default localhost:11434
    let ollama = Ollama::default();
    let prompts = &[
        "What is the best language (be very concise)",
        "What is the second best language (be very concise)",
        "what was my last question",
    ];

    let mut system_msg = ChatMessage::new(MessageRole::System, DEFAULT_SYSTEM_MOCK.to_string());
    let mut thread_msgs: Vec<ChatMessage> = vec![system_msg];

    for prompt in prompts {
        println!("\nprompt {prompt}");

        let prompt_msg = ChatMessage::new(MessageRole::User, prompt.to_string());

        thread_msgs.push(prompt_msg);

        let chat_req = ChatMessageRequest::new(MODEL.to_string(), thread_msgs.clone());

        let msg_content = run_chat_req(&ollama, chat_req).await?;

        if let Some(content) = msg_content {
            let asst_msg = ChatMessage::new(MessageRole::Assistant, content);
            thread_msgs.push(asst_msg);
        }
    }

    Ok(())
}

pub async fn run_chat_req(ollama: &Ollama, chat_req: ChatMessageRequest) -> Result<Option<String>> {
    let mut stream = ollama.send_chat_messages_stream(chat_req).await?;
    let mut stdout = tokio::io::stdout();
    let mut current_asst_msg_elems: Vec<String> = Vec::new();
    let mut char_count = 0;
    let mut final_received = false;

    while let Some(res) = stream.next().await {
        let res = res.map_err(|_| "stream.next error")?;
        let msg_content = res.message.content.clone();

        char_count += msg_content.len();
        if char_count > 80 {
            stdout.write_all(b"\n").await?;
            char_count = 0;
        }

        stdout.write_all(msg_content.as_bytes()).await?;
        stdout.flush().await?;

        current_asst_msg_elems.push(msg_content);

        if res.done {
            final_received = true;
            break;
        }
    }

    stdout.write_all(b"\n").await?;
    stdout.flush().await?;

    if final_received {
        let asst_content = current_asst_msg_elems.join("");
        Ok(Some(asst_content))
    } else {
        Ok(None)
    }
}
