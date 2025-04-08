use ollama::Result;
use ollama::consts::*;
use ollama::generate::gen_stream_print;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use simple_fs::ensure_file_dir;
use simple_fs::save_json;

#[tokio::main]
async fn main() -> Result<()> {
    //default localhost:11434
    let ollama = Ollama::default();
    let prompts = &[
        "why sky is red? (be concise)",
        "what was my previous question?",
    ];

    let mut last_ctx = None;
    for prompt in prompts {
        println!("prompt {prompt}");
        let mut gen_req = GenerationRequest::new(MODEL.to_string(), prompt.to_string());

        if let Some(last_ctx) = last_ctx.take() {
            gen_req = gen_req.context(last_ctx);
        }

        let ctx = gen_stream_print(&ollama, gen_req).await?;

        if let Some(ctx) = ctx {
            last_ctx = Some(ctx);
            let ctx_file_path = ".c02-data/ctx.json";
            ensure_file_dir(ctx_file_path)?;
            save_json(ctx_file_path, &last_ctx)?;
        }
    }

    Ok(())
}
