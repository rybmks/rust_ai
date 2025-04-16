mod ai_tools;
mod currency;
mod spec;

pub use ai_tools::*;
pub use spec::*;

use crate::Result;
use rpc_router::RouterBuilder;

pub fn new_ai_tools() -> Result<AiTools> {
    let router = RouterBuilder::default()
        .extend(currency::router_builder())
        .build();

    let mut chat_tools = Vec::new();
    chat_tools.extend(currency::chat_tools()?);

    Ok(AiTools::new(router, chat_tools))
}
