use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};

use crate::{
    api::converse, app::components::{chat_area::ChatArea, type_area::TypeArea}, model::conversation::{Conversation, Message}
};
pub mod components;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (conversation, set_conversation) = signal(Conversation::new());
    let send = Action::new(move |new_message: &String| {
        let user_message = Message {
            text: new_message.clone(),
            is_user: true,
        };

        set_conversation.update(move |c| {
            c.messages.push(user_message);
        });

        converse(conversation.get())
    });

    Effect::new(move |_| {
        if send.input().get().is_some() {
            let model_message = Message {
                text: String::from("..."),
                is_user: false,
            };

            set_conversation.update(move |c| {
                c.messages.push(model_message);
            });
        }
    });

    Effect::new(move |_|{
        if let Some(Ok(response)) = send.value().get() {
            set_conversation.update(move |c| {
                c.messages.last_mut().unwrap().text = response;
            });
        }
    });

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/video_13_chat.css"/>

        // sets the document title
        <Title text="Rusty llama"/>
        <ChatArea conv=conversation/>
        <TypeArea send/>
    }
}
