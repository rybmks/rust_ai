use std::path::Path;

use futures::StreamExt;
use futures::stream;
use ollama::Result;
use ollama::consts::*;
use ollama::generate::gen_stream_print;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::chat::MessageRole;
use ollama_rs::generation::chat::request;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::embeddings;
use ollama_rs::generation::embeddings::request::EmbeddingsInput;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use simple_fs::ensure_dir;
use simple_fs::ensure_file_dir;
use simple_fs::read_to_string;
use simple_fs::save_be_f64;
use simple_fs::save_json;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const MOCK_DIR: &str = "_mock_data";
const C04_DIR: &str = ".co4-data";

#[tokio::main]
async fn main() -> Result<()> {
    //default localhost:11434
    let ollama = Ollama::default();
    ensure_dir(C04_DIR)?;

    let txt = read_to_string(Path::new(MOCK_DIR).join("for-embeddings.txt"))?;
    let splits = simpe_text_splitter(&txt, 500)?;

    println!("splits count: {}", splits.len());
    let input = EmbeddingsInput::Multiple(splits);
    let request = GenerateEmbeddingsRequest::new(MODEL.to_string(), input);
    let res = ollama.generate_embeddings(request).await?;
    let embeddings = res.embeddings;
    println!("embeddings size: {}", embeddings.len());

    for (i, seg) in embeddings.iter().enumerate() {
        println!();
        let byte_vec: Vec<u8> = seg.iter().flat_map(|f| f.to_le_bytes()).collect();

        let file_name = format!("c04-embeddings-{:0>2}.txt", i);
        let path = Path::new(C04_DIR).join(file_name);
        std::fs::write(path, &byte_vec)?;

        println!("text_length: {}", txt.len());
        println!("embeddings size: {}", &embeddings.len());

        let file_name = format!("c04-embeddings-{:0>2}.json", i);
        save_json(Path::new(C04_DIR).join(file_name), &embeddings)?;

        let file_name = format!("c04-embeddings-{:0>2}.be-f64.bin", i);
        let file_path = Path::new(C04_DIR).join(file_name);
        save_be_f64(
            &file_path,
            &seg.iter().map(|v| *v as f64).collect::<Vec<f64>>(),
        )?;
    }

    Ok(())
}

fn simpe_text_splitter(txt: &str, num: u32) -> Result<Vec<String>> {
    let mut res = Vec::new();
    let mut last = 0;
    let mut count = 0;

    for (idx, _) in txt.char_indices() {
        count += 1;

        if count == num {
            res.push(&txt[last..idx + 1]);
            last = idx + 1;
            count = 0;
        }
    }

    if last < txt.len() {
        res.push(&txt[last..]);
    }
    Ok(res.into_iter().map(String::from).collect())
}
