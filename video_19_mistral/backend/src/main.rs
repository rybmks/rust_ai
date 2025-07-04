use backend::{
    models::{Model, mistral7b::Mistral7B},
    *,
};
use candle_core::{Device, MetalDevice, backend::BackendDevice};
use hf_hub::{
    Repo, RepoType,
    api::tokio::{Api, ApiBuilder},
};
use serde::{Deserialize, Serialize};

const MODEL_DIR: &str = "./model";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let token = "TOKEN";

    let api = ApiBuilder::new()
        .with_token(Some(token.to_string()))
        .with_cache_dir(std::path::PathBuf::from("./.hf_cache"))
        .build()?;
    let device = Device::Metal(MetalDevice::new(0)?);

    let mut model = Mistral7B::init(api, device).await?;
    model.run("Make a short essay about beer", 400)?;
    Ok(())
}
