use crate::error::Result;
use std::sync::Arc;

use async_openai::{Client, config::OpenAIConfig};

pub type OaClient = Arc<Client<OpenAIConfig>>;

pub fn new_oa_client() -> Result<OaClient> {
    Ok(Client::new().into())
}
