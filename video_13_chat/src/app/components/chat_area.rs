use leptos::html::Div;
use leptos::{prelude::ReadSignal, *};
use leptos::prelude::{ ClassAttribute, Effect, ElementChild, Get, NodeRef, NodeRefAttribute};
use crate::model::conversation::Conversation;

const USER_MESSAGE_CLASS: &str = "max-w-md p-4 mb-5 rounded-lg self-end";
const MODEL_MESSAGE_CLASS: &str = "max-w-md p-4 mb-5 rounded-lg self-start";

#[component]
pub fn ChatArea(conv: ReadSignal<Conversation>) -> impl IntoView{
    let chat_div_ref: NodeRef<Div> = NodeRef::new();

    Effect::new(move |_|{
        conv.get();
        if let Some(div) = chat_div_ref.get(){
            div.set_scroll_top(div.scroll_height());
        }
    });
    view! {
        <div node_ref=chat_div_ref>
        { move || conv.get().messages.iter().map(move |mes| {
           let class_str = if mes.is_user {USER_MESSAGE_CLASS} else {MODEL_MESSAGE_CLASS};
            view! {
                <div class={class_str}>
                {
                    mes.text.clone()
                }
                </div>
            }
        }).collect::<Vec<_>>()

        }
        </div>
    }

}