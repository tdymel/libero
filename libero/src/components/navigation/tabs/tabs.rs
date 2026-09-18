use dioxus::prelude::*;

use super::core::{TabSpec, TabsView, render_tabs};
use crate::{
    components::common::{
        ClassList, Input, OptionLabel, OptionSource, Options, States, base_color, input_from_str,
    },
    hooks::{use_root_id, use_theme},
    str_enum::str_enum,
    sx::{Sx, ThemeAwareValue},
    theme::Size,
    utils::warn,
};

str_enum! {
    /// When an arrow key selects the tab it moves to.
    pub enum TabsActivation {
        /// Arrows select as they move: the panel follows the focus.
        #[default]
        Automatic = "automatic",
        /// Arrows only move the focus; Enter or Space selects. For panels that
        /// are slow to render or fetch (APG manual activation).
        Manual = "manual",
    }
}

input_from_str!(TabsActivation);

impl From<TabsActivation> for Input<TabsActivation> {
    fn from(value: TabsActivation) -> Self {
        Input::Value(value)
    }
}

// Hand-written rather than `base_props!`, which is not generic - as
// `SliderProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps<T: Options> {
    /// Strictly controlled - pair it with `onchange`.
    value: T,
    /// Called with the tab that should become selected.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
    /// The body of the selected tab. Called for `value` only, so the other
    /// panels cost nothing.
    #[props(default)]
    panel: Option<Callback<T, Element>>,
    /// The tabs to show. Defaults to every `Options::options()`.
    ///
    /// `OptionItem::new(tab).disabled(true)` renders a tab that cannot be
    /// picked. A grouped [`OptionList`](crate::components::OptionList) is
    /// accepted and its tabs are drawn flattened, in the order given: a strip
    /// has no room for group headings and does not draw them.
    ///
    /// A strip has no dropdown to put a loader in, so a **pending**
    /// [`OptionSource`](crate::components::OptionSource) - one built from a
    /// [`Resource`] that has not answered yet - simply draws no tabs. Await
    /// the fetch above the strip if that matters.
    #[props(default, into)]
    options: OptionSource<T>,
    /// Overrides `Options::label`. Runs during render, so it can read a
    /// locale from context - which is how a renamed strip stays renamed.
    ///
    /// `"Konto".into()` names a tab; `OptionLabel::rich(name, rsx! { .. })`
    /// draws it and names it, because the rsx is what a screen reader cannot
    /// use.
    #[props(default)]
    option_label: Option<Callback<T, OptionLabel>>,
    #[props(default, into)]
    size: Input<Size>,
    /// Indicator and selected-label colour.
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    /// Tabs share the row evenly instead of sizing to their label.
    #[props(default)]
    full_width: Option<bool>,
    /// `Manual` lets the arrows move the focus without selecting; Enter or
    /// Space selects the focused tab.
    #[props(default, into)]
    activation: Input<TabsActivation>,
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
/// The tabs are `T::options()` unless `options` narrows them, and `panel` is a
/// match over `T` - so a forgotten or misspelled tab is a compile error.
#[component]
pub fn Tabs<T: Options>(props: TabsProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();

    if props.onchange.is_none() {
        warn("Tabs: without `onchange` the selection can never change.");
    }
    if props.panel.is_none() {
        warn("Tabs: without `panel` there is nothing below the tabs to show.");
    }

    let list = props.options.or_static();
    let values = list.values();
    let option_disabled = list.disabled();
    let selected = values.iter().position(|value| *value == props.value);
    if selected.is_none() {
        warn("Tabs: `value` is not one of the tabs, so none is selected.");
    }

    let tabs: Vec<TabSpec> = values
        .iter()
        .zip(&option_disabled)
        .map(|(value, &disabled)| {
            let label = match &props.option_label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            };
            let name = label.name;
            TabSpec {
                content: label.content.unwrap_or_else(|| rsx! { "{name}" }),
                name,
                disabled,
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
            manual: props.activation.copied_or(TabsActivation::Automatic) == TabsActivation::Manual,
            focusable: true,
            panel_stop: true,
            size: props.size.copied_or(theme.tabs.size),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        },
        root(),
    )
}
