use async_openai::types::{ChatCompletionToolChoiceOption, CreateChatCompletionRequest};
use serde_json::Value;

use crate::{
    chat::{self},
    error::Result,
    gpts,
    oa_client::OaClient,
    tools::AiTools,
};

pub async fn send_user_msg(
    oa_client: OaClient,
    ai_tools: AiTools,
    question: &str,
) -> Result<String> {
    let chat_client = oa_client.chat();
    let model = gpts::MODEL;

    let messages = vec![chat::user_msg(question)?];

    let rpc_router = ai_tools.router();
    let tools = Some(ai_tools.chat_tools_clone());

    let msg_req = CreateChatCompletionRequest {
        model: gpts::MODEL.to_string(),
        messages: messages.clone(),
        tools: tools.clone(),
        tool_choice: Some(ChatCompletionToolChoiceOption::Auto),
        ..Default::default()
    };

    let chat_response = chat_client.create(msg_req).await?;
    let first_choise = chat::first_chiose(chat_response)?;

    if let Some(response_content) = first_choise.message.content {
        return Ok(response_content);
    }

    struct ToolResponse {
        tool_call_id: String,
        response: Value,
    }

    let mut tool_responses: Vec<ToolResponse> = vec![];
    let tool_calls = first_choise.message.tool_calls;

    for tool_call in tool_calls.iter().flatten() {
        let tool_call_id = tool_call.id.clone();
        let fn_name = tool_call.function.name.clone();
        let params: Value = serde_json::from_str(&tool_call.function.arguments)?;

        let call_result = rpc_router
            .call_route(None, fn_name, Some(params))
            .await
            .map_err(Box::new)?;
        let response = call_result.value;

        tool_responses.push(ToolResponse {
            tool_call_id,
            response,
        });
    }

    let mut messages = messages;
    if let Some(tool_calls) = tool_calls {
        messages.push(chat::tool_calls_msg(tool_calls)?);
    }

    for ToolResponse {
        tool_call_id,
        response,
    } in tool_responses
    {
        messages.push(chat::tool_response_msg(tool_call_id, response)?);
    }

    let msg_req = CreateChatCompletionRequest {
        model: model.to_string(),
        messages,
        tools,
        tool_choice: Some(ChatCompletionToolChoiceOption::Auto),
        ..Default::default()
    };

    let chat_response = chat_client.create(msg_req).await?;
    let first_choise = chat::first_chiose(chat_response)?;

    let content = first_choise.message.content.ok_or("No final content")?;

    Ok(content)
}
