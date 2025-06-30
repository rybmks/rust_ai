use crate::model;
use crate::model::conversation::Conversation;
use leptos::prelude::ServerFnError;
use leptos::*;

#[server(Converse, "/api")]
pub async fn converse(prompt: Conversation) -> Result<String, ServerFnError> {
    use axum::http::Method;
    use axum::Extension;
    use kalosm::{language::*, *};
    use leptos_axum::extract;

    let model: Extension<Llama> = extract().await?;

    let character_name = "### Assistant";
    let user_name = "### Human";
    let persona = "A chat between a human and an assistant";
    let mut history = format!("");

    for message in prompt.messages.into_iter() {
        let msg = message.text;
        let curr_line = if message.is_user {
            format!("{character_name}:{msg}\n")
        } else {
            format!("{user_name}:{msg}\n")
        };

        history.push_str(&curr_line);
    }

    let mut response = String::new();

    let full_prompt = format!("{persona}\n{history}{character_name}:");

    let mut stream = model(&full_prompt);

    let mut response = String::new();
    while let Some(token) = stream.next().await {
        response.push_str(&token);
    }

    Ok(response)
}
