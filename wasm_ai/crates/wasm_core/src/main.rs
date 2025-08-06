//! TODO: add support of converstion history

use std::{
    io::{Write, stdout},
    process::ExitCode,
};
use wasmedge_wasi_nn::{ExecutionTarget, GraphBuilder, GraphEncoding};

pub mod model;
use model::GgmlMetadata;

type Error = Box<dyn std::error::Error>;

const SYSTEM_PROMPT: &str =
    "You are a helpful assistant. Answer in a concise and informative way.\n\n";

fn create_config() -> GgmlMetadata {
    GgmlMetadata {
        ctx_size: 32000,
        batch_size: 1024,
        ubatch_size: 16,
        ..Default::default()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<ExitCode, Error> {
    let config = create_config();

    let config_json = serde_json::to_string(&config)?;
    let graph = GraphBuilder::new(GraphEncoding::Ggml, ExecutionTarget::CPU)
        .config(config_json)
        .build_from_cache(&config.model_alias)?;

    let mut execution_context = graph.init_execution_context()?;

    let mut conversation_history: Vec<String> = vec![];
    conversation_history.push(format!("SYSTEM:{SYSTEM_PROMPT}"));

    let res: Result<(), Error> = loop {
        print!("Write your prompt(or /q for exit): ");
        stdout().flush()?;

        let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt)?;
        let prompt = prompt.trim();

        if prompt.eq("/q") {
            break Ok(());
        }

        if conversation_history.len() > 100 {
            conversation_history.clear();
            conversation_history.push(format!("SYSTEM:{SYSTEM_PROMPT}"));
        }

        conversation_history.push(format!("User: {prompt}\nAssistant:"));

        let output =
            model::run_model(conversation_history.join("\n"), &mut execution_context).await;

        match output {
            Ok(out) => {
                let assistant_response = String::from_utf8(out).expect("Failed cast to string");
                println!("Response: {assistant_response}");

                if !assistant_response.trim().is_empty() {
                    conversation_history.push(format!("{assistant_response}\n"));
                }
            }
            Err(err) => {
                eprintln!("Error while getting model response: {err}")
            }
        }
    };

    res?;

    Ok(ExitCode::SUCCESS)
}
