//! Unsued: Replaced by run.sh

use hf_hub::{
    Repo, RepoType,
    api::tokio::{Api, ApiBuilder},
};

const _MISTRAL_ID: &str = "mistralai/Mistral-7B-v0.1";
const _LLAMA_ID: &str = "TinyLlama/TinyLlama-1.1B-Chat-v1.0";
const _LLAMA_GGUF_ID: &str = "second-state/Llama-2-7B-Chat-GGUF";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = "TOKEN";
    let api = ApiBuilder::new()
        .with_token(Some(token.to_string()))
        .with_cache_dir(std::path::PathBuf::from("./.hf_cache"))
        .build()?;

    _download_tiny_llama(api).await?;
    Ok(())
}

pub async fn _download_llama_gguf(api: Api) -> Result<(), Box<dyn std::error::Error>> {
    let repo = api.repo(Repo::new(_LLAMA_GGUF_ID.to_string(), RepoType::Model));
    let filename = "llama-2-7b-chat.Q4_K_M.gguf";

    let _model = repo.get(filename).await?;
    Ok(())
}
pub async fn _download_tiny_llama(api: Api) -> Result<(), Box<dyn std::error::Error>> {
    let repo = api.repo(Repo::new(_LLAMA_ID.to_string(), RepoType::Model));

    let _tokenizer = repo.get("tokenizer.model").await?;
    let _config = repo.get("config.json").await?;
    let _ = repo.get("generation_config.json").await.ok();

    let _model = repo.get("model.safetensors").await?;

    Ok(())
}

async fn _download_mistral(api: Api) -> Result<(), Box<dyn std::error::Error>> {
    let repo = api.repo(Repo::new(_MISTRAL_ID.to_string(), RepoType::Model));

    //download model files
    let _tokenizer_path = repo.get("tokenizer.json").await?;
    let _config_path = repo.get("config.json").await?;
    let _tensors_index_path = repo.get("model.safetensors.index.json").await?;

    let _ = repo.get("tokenizer_config.json").await?;
    let _tensors1 = repo.get("model-00001-of-00002.safetensors").await?;
    let _tensors2 = repo.get("model-00002-of-00002.safetensors").await?;

    Ok(())
}
