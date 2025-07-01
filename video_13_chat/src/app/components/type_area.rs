use leptos::html::Input;
use leptos::prelude::Action;
use leptos::prelude::IntoMaybeErased;
use leptos::prelude::{ElementChild, Get, NodeRef, NodeRefAttribute, OnAttribute, ServerFnError};
use leptos::*;

#[component]
pub fn TypeArea(send: Action<String, Result<String, ServerFnError>>) -> impl IntoView {
    let input_ref: NodeRef<Input> = NodeRef::new();

    view! {
        <div>
            <form on:submit=move |ev| {
                ev.prevent_default();
                let input = input_ref.get().expect("failed to get input");
                send.dispatch(input.value());
                input.set_value("");
            }>
                <input type="text" node_ref=input_ref/>
                <input type="submit"/>
            </form>
        </div>
    }
}
