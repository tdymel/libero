use dioxus::prelude::*;

use crate::{
    components::{
        common::{ClassList, Input, OptionSource, Parts, States},
        form::DropdownPart,
    },
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
    /// Said when an open list has no options, and drawn there without `empty`. Defaults to
    /// [`ComboboxLabels::nothing_found`](crate::localization::ComboboxLabels::nothing_found).
    #[props(default, into)]
    empty_label: Option<String>,
    /// Said while fetching. Defaults to [`CommonLabels::loading`](crate::localization::CommonLabels::loading).
    #[props(default, into)]
    loading_label: Option<String>,
    /// Names the listbox, usually the id of the trigger's label.
    #[props(default, into)]
    labelled_by: Option<String>,
    /// A row's height and font size.
    #[props(default, into)]
    size: Input<Size>,
    /// The dropdown's corner radius.
    #[props(default, into)]
    radius: Input<Size>,
    /// Draws no list and ignores the keys; the trigger reads as closed. Disable the trigger too.
    #[props(default)]
    disabled: Option<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    /// Styles the dropdown, not the wrapper it hangs off.
    #[props(default, into)]
    sx: Input<Sx>,
    /// Styles the dropdown and its inner parts, under `sx`.
    #[props(default, into)]
    parts: Input<Parts<DropdownPart>>,
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
    let localization = use_localization();
    let common = localization.common;
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

    // Read only while open: the mount effect's write then redraws nothing.
    let opened = state.is_open();
    let active_row = opened
        .then(|| state.active())
        .flatten()
        .filter(|_| count > 0)
        .map(|row| row.min(count - 1));

    // Drawn eagerly: `Callback`s from one scope compare equal across renders and left stale rows.
    // A `Vec<Element>` never compares equal, so the subtree never memoizes.
    // Closed, no list is drawn: placeholders keep the count the keys move through.
    let option = props.option;
    let row_disabled = list.disabled();
    let groups = list.group_labels();
    let rows: Vec<Element> = match opened {
        true => values
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
            .collect(),
        false => vec![VNode::empty(); count],
    };

    rsx! {
        ComboboxCore {
            rows,
            groups,
            row_disabled,
            active: active_row,
            onactive: move |row| state.set_active(Some(row)),
            opened,
            onopened: move |opened| state.set_open(opened),
            state,
            empty: props.empty,
            nothing_found: Some(
                props
                    .empty_label
                    .unwrap_or_else(|| localization.combobox.nothing_found.to_string()),
            ),
            labelled_by: props.labelled_by,
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
            parts: props.parts,
            states: props.states,
            {props.children}
        }
    }
}
