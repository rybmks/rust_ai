use crate::Result;
use async_openai::{Client, config::OpenAIConfig};

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

pub mod asst;
pub mod msg;

pub type OaClient = Client<OpenAIConfig>;

pub fn new_oa_client() -> Result<OaClient> {
    if std::env::var(ENV_OPENAI_API_KEY).is_ok() {
        Ok(Client::new())
    } else {
        tracing::error!("No {ENV_OPENAI_API_KEY} env is provided");
        Err(format!("No {ENV_OPENAI_API_KEY} env is provided").into())
    }
}
