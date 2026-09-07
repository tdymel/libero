use dioxus::prelude::*;

use crate::{
    components::{ClassList, Input, OptionSource, States},
    hooks::use_theme,
    sx::Sx,
    theme::Size,
};

use super::{core::ComboboxCore, option::ComboboxOptionArgs, state::ComboboxState};

// Hand-written rather than `base_props!`, which is not generic.
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Clone + PartialEq + 'static> {
    /// Whether the list is open and which row the arrows are on, from
    /// [`use_combobox`](crate::hooks::use_combobox). It lives in the caller's
    /// scope, and `state.a11y_attributes()` is what wires the control up.
    /// Close it on your trigger's blur (`state.close()`): on the web an open
    /// list takes Escape first, so an enclosing `Modal` or `HoverCard` stops
    /// hearing Escape while it stays open.
    state: ComboboxState,
    /// The options to list, already filtered. There is no query prop: a
    /// suggestion list narrows by handing a shorter `options` in.
    ///
    /// A `Vec<T>` converts, which is the flat list. An
    /// [`OptionList`](crate::components::OptionList) adds named groups and
    /// per-option `disabled`, and a [`Resource`] adds the fetch: the list then
    /// reads pending against ready itself, and derives the loader, `aria-busy`
    /// and the held-back empty state from it.
    #[props(into)]
    options: OptionSource<T>,
    /// Draws one row - typically a [`ComboboxOption`](super::ComboboxOption),
    /// which is themed and wires the click for you.
    option: Callback<ComboboxOptionArgs<T>, Element>,
    /// Shown in place of the list when `options` is empty.
    #[props(default)]
    empty: Option<Element>,
    /// What the status region says while the options are being fetched. Unset, the theme's
    /// [`ComboboxLabels`](crate::theme::ComboboxLabels) says it.
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
    /// Styles the dropdown - the wrapper it hangs off is scaffolding, not a
    /// user-facing element.
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// The trigger, and anything else that belongs with it - a hidden input,
    /// say. `Combobox` renders no control of its own.
    children: Element,
}

/// A listbox that hangs off whatever control you put in it.
///
/// It holds no state: `opened` and the selection are the caller's, the rows
/// are drawn by `option`, and the trigger is just `children`. All it adds is
/// the placement, the arrow keys, and the row theming.
///
/// Generic only at this boundary: the options are erased to indices here, and
/// everything below compiles once.
#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(props: ComboboxProps<T>) -> Element {
    let theme = use_theme();
    let list = props.options.list();
    let values = list.values();
    let count = values.len();
    // The list has been asked for and has not answered yet. It wins over
    // `empty`: an async list is empty between the request and its results, so
    // without this every keystroke would flash "no results" before the data
    // lands. It replaces the rows too, which belong to the previous query.
    let loading = props.options.is_pending();
    let state = props.state;

    // Opening always starts at the top; nothing carries over from last time.
    // Reading `opened` is what makes the effect re-run on it.
    use_effect(move || {
        let _ = state.is_open();
        state.set_active(Some(0));
    });

    let active_row = state
        .active()
        .filter(|_| count > 0)
        .map(|row| row.min(count - 1));

    // Drawn here, eagerly, and handed down as values. A `Callback` would be
    // the obvious way to keep this lazy, but two `Callback`s built in the same
    // scope on different renders compare *equal* - `GenerationalBox::ptr_eq`
    // sees the recycled slot - so the whole subtree below would memoize and a
    // filtered `options` would leave stale rows on screen. A `Vec<Element>`
    // never compares equal, which is exactly the guarantee this needs.
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
                    .unwrap_or_else(|| theme.combobox.labels.loading.to_string())
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
