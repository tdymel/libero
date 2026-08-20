use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::base_props, inputs::Button},
    sx::Sx,
    utils::warn,
};

use super::ToggleGroupContext;

base_props! {
    pub struct ToggleButtonProps {
        /// Its identity in the group's selection.
        #[props(into)]
        value: String,
        /// Falls back to the group's `disabled`.
        #[props(default)]
        disabled: Option<bool>,
        children: Element,
    }
}

/// One member of a `ToggleButtonGroup`. A `Button` that reads its selected
/// state, look and click handling from the group - no styling of its own.
#[component]
pub fn ToggleButton(props: ToggleButtonProps) -> Element {
    // A hook, so it stays above the branch below.
    let context = try_use_context::<ToggleGroupContext>();

    let Some(context) = context else {
        warn("ToggleButton: used outside a ToggleButtonGroup, so it can never be selected.");
        return rsx! {};
    };

    let state = context.state.read();
    let selected = state.value.iter().any(|value| value == &props.value);
    let disabled = props.disabled.unwrap_or(state.disabled);

    // The context is `Copy`, so the handler reads the live selection rather
    // than a clone of this render's.
    let value = props.value.clone();
    let onclick = move |_: Event<MouseData>| {
        let onchange = context.onchange.peek();
        let Some(onchange) = onchange.as_ref() else {
            return;
        };

        let state = context.state.peek();
        let selected = state.value.iter().any(|entry| entry == &value);

        let next = match (selected, state.exclusive) {
            (true, true) => Vec::new(),
            (true, false) => state
                .value
                .iter()
                .filter(|entry| *entry != &value)
                .cloned()
                .collect(),
            (false, true) => vec![value.clone()],
            (false, false) => {
                let mut next = state.value.clone();
                next.push(value.clone());
                next
            }
        };

        onchange.call(next);
    };

    rsx! {
        Button {
            selected,
            disabled,
            onclick,
            variant: state.variant.clone(),
            color: state.color.clone(),
            size: state.size.clone(),
            radius: state.radius.clone(),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
            {props.children}
        }
    }
}
