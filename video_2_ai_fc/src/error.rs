use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display, From)]
pub enum Error {
    #[from]
    Custom(String),

    #[from]
    OpenAi(async_openai::error::OpenAIError),

    #[from]
    Json(serde_json::Error),

    #[from]
    RpcCall(Box<rpc_router::CallError>),
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Custom(err.to_string())
    }
}
