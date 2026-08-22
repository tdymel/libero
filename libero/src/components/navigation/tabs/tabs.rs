use dioxus::prelude::*;

use super::core::{TabSpec, TabsView, render_tabs};
use super::tab_value::TabValue;
use crate::{
    components::{ClassList, Input, States, common::base_color},
    hooks::{use_root_id, use_theme},
    sx::{Sx, ThemeAwareValue},
    theme::Size,
    utils::warn,
};

// Hand-written rather than `base_props!`, which is not generic - as
// `SliderProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps<T: TabValue> {
    /// Strictly controlled - pair it with `onchange`.
    value: T,
    /// Called with the tab that should become selected.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
    /// The body of the selected tab. Called for `value` only, so the other
    /// panels cost nothing.
    #[props(default)]
    panel: Option<Callback<T, Element>>,
    /// The tabs to show. Defaults to every `TabValue::options()`.
    #[props(default)]
    tabs: Option<Vec<T>>,
    /// Overrides `TabValue::label`. Runs during render, so it can read a
    /// locale from context - which is how a translated strip is written.
    #[props(default)]
    label: Option<Callback<T, String>>,
    /// Rich tab content (an icon beside the text, a badge). `label` still
    /// supplies the accessible name.
    #[props(default)]
    render_label: Option<Callback<T, Element>>,
    /// Tabs that render but cannot be picked.
    #[props(default)]
    disabled: Vec<T>,
    #[props(default, into)]
    size: Input<Size>,
    /// Indicator and selected-label colour.
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    /// Tabs share the row evenly instead of sizing to their label.
    #[props(default)]
    full_width: Option<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A strip of tabs over an enum, with the selected one's panel below it.
/// Controlled: it renders `value` and asks for a new one through `onchange`.
///
/// The tabs are `T::options()` unless `tabs` narrows them, and `panel` is a
/// match over `T` - so a forgotten or misspelled tab is a compile error.
#[component]
pub fn Tabs<T: TabValue>(props: TabsProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();

    if props.onchange.is_none() {
        warn("Tabs: without `onchange` the selection can never change.");
    }
    if props.panel.is_none() {
        warn("Tabs: without `panel` there is nothing below the tabs to show.");
    }

    let values = props.tabs.clone().unwrap_or_else(|| T::options().to_vec());
    let selected = values.iter().position(|value| *value == props.value);
    if selected.is_none() {
        warn("Tabs: `value` is not one of the tabs, so none is selected.");
    }

    let name = |value: &T| match &props.label {
        Some(label) => label.call(value.clone()),
        None => value.label(),
    };

    let tabs: Vec<TabSpec> = values
        .iter()
        .map(|value| {
            let name = name(value);
            TabSpec {
                content: match &props.render_label {
                    Some(render) => render.call(value.clone()),
                    None => rsx! { "{name}" },
                },
                name,
                disabled: props.disabled.contains(value),
            }
        })
        .collect();

    let panel = match (&props.panel, selected) {
        (Some(panel), Some(_)) => panel.call(props.value.clone()),
        _ => rsx! {},
    };

    let onchange = props.onchange;
    let pick = use_callback(move |index: usize| {
        if let Some(onchange) = &onchange
            && let Some(value) = values.get(index)
        {
            onchange.call(value.clone());
        }
    });

    render_tabs(
        TabsView {
            tabs,
            selected,
            panel,
            onselect: pick,
            color: base_color(props.color.as_ref()),
            full_width: props.full_width.unwrap_or(false),
            size: props.size.copied_or(theme.tabs.size),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        },
        root(),
    )
}
