use dioxus::prelude::*;

use crate::{
    components::common::{ClassList, Input, OptionSource, States},
    hooks::use_localization,
    sx::Sx,
    theme::Size,
};

use super::{ComboboxState, core::ComboboxCore, option::ComboboxOptionArgs};

// Hand-written rather than `base_props!`, which is not generic.
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Clone + PartialEq + 'static> {
    /// Open state and highlight, from [`use_combobox`](crate::hooks::use_combobox); wire the trigger with
    /// `state.a11y_attributes()`. Close it on the trigger's blur: an open list takes Escape from a `Modal`.
    state: ComboboxState,
    /// The options, already filtered. A `Vec<T>`, an [`OptionList`](crate::components::OptionList)
    /// (groups, `disabled`), or a [`Resource`], which adds the loader and `aria-busy`.
    #[props(into)]
    options: OptionSource<T>,
    /// Draws one row, typically a [`ComboboxOption`](super::ComboboxOption).
    option: Callback<ComboboxOptionArgs<T>, Element>,
    /// Shown in place of the list when `options` is empty.
    #[props(default)]
    empty: Option<Element>,
    /// Said while fetching. Defaults to [`CommonLabels::loading`](crate::localization::CommonLabels::loading).
    #[props(default, into)]
    loading_label: Option<String>,
    /// A row's height and font size.
    #[props(default, into)]
    size: Input<Size>,
    /// The dropdown's corner radius.
    #[props(default, into)]
    radius: Input<Size>,
    #[props(default)]
    disabled: Option<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    /// Styles the dropdown, not the wrapper it hangs off.
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// The trigger, plus anything that belongs with it. `Combobox` renders no control of its own.
    children: Element,
}

/// A stateless listbox that hangs off whatever trigger you put in it.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Combobox, ComboboxOption, ComboboxOptionArgs, use_combobox};
/// # fn app() -> Element {
/// let state = use_combobox();
/// let mut picked = use_signal(|| "Apple");
/// rsx! {
///     Combobox {
///         state,
///         options: vec!["Apple", "Pear"],
///         option: move |row: ComboboxOptionArgs<&'static str>| rsx! {
///             ComboboxOption {
///                 selected: picked() == row.value,
///                 onpick: move |_| {
///                     picked.set(row.value);
///                     state.close();
///                 },
///                 "{row.value}"
///             }
///         },
///         Button {
///             attributes: state.a11y_attributes(),
///             onclick: move |_| state.toggle(),
///             onblur: move |_| state.close(),
///             "{picked}"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/combobox>
#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(props: ComboboxProps<T>) -> Element {
    let common = use_localization().common;
    let list = props.options.list();
    let values = list.values();
    let count = values.len();
    // Wins over `empty` and the old rows, or each keystroke flashes "no results" before the data lands.
    let loading = props.options.is_pending();
    let state = props.state;

    // Every open starts at the top; reading `is_open` re-runs the effect on it.
    use_effect(move || {
        let _ = state.is_open();
        state.set_active(Some(0));
    });

    let active_row = state
        .active()
        .filter(|_| count > 0)
        .map(|row| row.min(count - 1));

    // Drawn eagerly: `Callback`s from one scope compare equal across renders and left stale rows.
    // A `Vec<Element>` never compares equal, so the subtree never memoizes.
    let option = props.option;
    let row_disabled = list.disabled();
    let groups = list.group_labels();
    let rows: Vec<Element> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            option.call(ComboboxOptionArgs {
                value: value.clone(),
                index,
                active: active_row == Some(index),
                disabled: row_disabled[index],
            })
        })
        .collect();

    rsx! {
        ComboboxCore {
            rows,
            groups,
            row_disabled,
            active: active_row,
            onactive: move |row| state.set_active(Some(row)),
            opened: state.is_open(),
            onopened: move |opened| state.set_open(opened),
            state,
            empty: props.empty,
            loading: loading.then(|| {
                props
                    .loading_label
                    .unwrap_or_else(|| common.loading.to_string())
            }),
            size: props.size,
            radius: props.radius,
            disabled: props.disabled.unwrap_or(false),
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            states: props.states,
            {props.children}
        }
    }
}
