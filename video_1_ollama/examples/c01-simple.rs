use ollama::Result;
use ollama::consts::*;
use ollama::generate::gen_stream_print;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

#[tokio::main]
async fn main() -> Result<()> {
    //default localhost:11434
    let ollama = Ollama::default();
    let model = MODEL.to_string();
    let prompt = "Say hi, but translate it to Ukranian".to_string();

    let gen_req = GenerationRequest::new(model, prompt).system(DEFAULT_SYSTEM_MOCK);
    gen_stream_print(&ollama, gen_req).await?;

    Ok(())
}
