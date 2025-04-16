use std::fmt::Display;

use crate::{error::Result, tools};
use async_openai::types::{
    ChatChoice, ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionTool, ChatCompletionToolArgs,
    CreateChatCompletionResponse, FunctionObject,
};
use schemars::JsonSchema;
use serde_json::Value;

pub fn tool_fn(
    name: impl Into<String>,
    description: impl Into<String>,
    params: Value,
) -> Result<ChatCompletionTool> {
    let tool = ChatCompletionToolArgs::default()
        .function(FunctionObject {
            name: name.into(),
            description: Some(description.into()),
            parameters: Some(params),
            ..Default::default()
        })
        .build()?;

    Ok(tool)
}

pub fn tool_response_msg(
    tool_call_id: String,
    content: impl Display,
) -> Result<ChatCompletionRequestMessage> {
    let msg = ChatCompletionRequestToolMessageArgs::default()
        .content(content.to_string())
        .tool_call_id(tool_call_id)
        .build()?;

    Ok(msg.into())
}

pub fn tool_calls_msg(
    tool_calls: Vec<ChatCompletionMessageToolCall>,
) -> Result<ChatCompletionRequestMessage> {
    let msg = ChatCompletionRequestAssistantMessageArgs::default()
        .tool_calls(tool_calls)
        .build()?;

    Ok(msg.into())
}
pub fn tool_fn_from_type<T: JsonSchema>() -> Result<ChatCompletionTool> {
    let spec = tools::tool_spec::<T>()?;
    tool_fn(spec.fn_name, spec.fn_description, spec.params)
}

pub fn user_msg(content: impl Into<String>) -> Result<ChatCompletionRequestMessage> {
    let msg = ChatCompletionRequestUserMessageArgs::default()
        .content(content.into())
        .build()?;

    Ok(msg.into())
}

pub fn first_chiose(chat_response: CreateChatCompletionResponse) -> Result<ChatChoice> {
    let f_choise = chat_response
        .choices
        .into_iter()
        .next()
        .ok_or("No first choise")?;

    Ok(f_choise)
}
