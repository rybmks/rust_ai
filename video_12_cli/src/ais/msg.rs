use async_openai::types::{
    CreateMessageRequest, CreateMessageRequestContent, MessageContent, MessageObject, MessageRole,
};

use crate::Result;

pub fn user_msg(content: impl Into<String>) -> CreateMessageRequest {
    CreateMessageRequest {
        role: MessageRole::User,
        content: CreateMessageRequestContent::Content(content.into()),
        ..Default::default()
    }
}

pub fn get_text_content(msg: MessageObject) -> Result<String> {
    let msg_content = msg
        .content
        .into_iter()
        .next()
        .ok_or("No message content found")?;

    let txt = match msg_content {
        MessageContent::Text(text) => text.text.value,
        ct => {
            return Err(format!("Content type {ct:?} not supported yet").into());
        }
    };

    Ok(txt)
}
