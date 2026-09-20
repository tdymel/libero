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
        /// Arrows only move the focus; Enter or Space selects. For slow panels.
        Manual = "manual",
    }
}

input_from_str!(TabsActivation);

// Hand-written: `base_props!` is not generic.
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps<T: Options> {
    /// Strictly controlled; pair it with `onchange`.
    value: T,
    /// Called with the tab that should become selected.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
    /// The selected tab's body; called for `value` only.
    #[props(default)]
    panel: Option<Callback<T, Element>>,
    /// The tabs, default every `Options::options()`. Groups draw flattened; a pending source draws none.
    #[props(default, into)]
    options: OptionSource<T>,
    /// Overrides `Options::label` during render; `OptionLabel::rich` draws rsx.
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
    /// `manual`: arrows move the focus, Enter or Space selects.
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

/// A strip of tabs over an enum, with the selected tab's panel below it.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Options, Tabs};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq, Options)] enum Tab { Overview, Settings }
/// let mut tab = use_signal(|| Tab::Overview);
/// rsx! {
///     Tabs {
///         value: tab(),
///         onchange: move |t| tab.set(t),
///         panel: |t: Tab| match t {
///             Tab::Overview => rsx! { "Overview" },
///             Tab::Settings => rsx! { "Settings" },
///         },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/tabs>
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
