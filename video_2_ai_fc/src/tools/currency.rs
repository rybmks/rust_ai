use async_openai::types::ChatCompletionTool;
use derive_more::Display;
use rpc_router::{RouterBuilder, RpcParams, router_builder};
use serde::{Deserialize, Serialize};

use crate::chat;

pub(super) fn router_builder() -> RouterBuilder {
    router_builder![get_currency_rate]
}

pub(super) fn chat_tools() -> crate::Result<Vec<ChatCompletionTool>> {
    let tool_currency = chat::tool_fn_from_type::<ConvertCurrencyParams>()?;
    Ok(vec![tool_currency])
}

#[derive(Debug, Deserialize, RpcParams, schemars::JsonSchema)]
#[schemars(
    title = "get_currency_rate",
    description = "Converts one currency into another"
)]
pub struct ConvertCurrencyParams {
    amount: f64,
    from: Currency,
    to: Currency,
}

#[derive(Serialize)]
pub struct CurrencyRate {
    to: Currency,
    converted: f64,
}

#[non_exhaustive]
#[derive(Debug, Display, Serialize, Deserialize, RpcParams, schemars::JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Uah,
    Usd,
    Eur,
}

impl Currency {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Uah => "UAH",
            Self::Usd => "USD",
            Self::Eur => "EUR",
        }
    }
}

async fn get_currency_rate(params: ConvertCurrencyParams) -> Result<CurrencyRate, String> {
    let rate = match (params.from.as_str(), params.to.as_str()) {
        ("EUR", "USD") => 1.1,
        ("EUR", "UAH") => 42.0,
        ("USD", "EUR") => 0.91,
        ("USD", "UAH") => 38.0,
        ("UAH", "EUR") => 0.0238,
        ("UAH", "USD") => 0.0263,
        (from, to) if from == to => 1.0,
        _ => {
            return Err(format!(
                "Unknown currency pair: {} -> {}",
                params.from, params.to
            ));
        }
    };

    Ok(CurrencyRate {
        converted: params.amount * rate,
        to: params.to,
    })
}
