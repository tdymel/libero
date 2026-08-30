use dioxus::prelude::*;

use crate::{
    components::{ClassList, Input, States},
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
    state: ComboboxState,
    /// The options to list, already filtered. There is no query prop: a
    /// suggestion list narrows by handing a shorter `options` in.
    options: Vec<T>,
    /// Draws one row - typically a [`ComboboxOption`](super::ComboboxOption),
    /// which is themed and wires the click for you.
    option: Callback<ComboboxOptionArgs<T>, Element>,
    /// Shown in place of the list when `options` is empty.
    #[props(default)]
    empty: Option<Element>,
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
    let count = props.options.len();
    let state = props.state;

    // Opening always starts at the top; nothing carries over from last time.
    // Reading `opened` is what makes the effect re-run on it.
    use_effect(move || {
        let _ = state.opened();
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
    let rows: Vec<Element> = props
        .options
        .iter()
        .enumerate()
        .map(|(index, value)| {
            option.call(ComboboxOptionArgs {
                value: value.clone(),
                index,
                active: active_row == Some(index),
            })
        })
        .collect();

    rsx! {
        ComboboxCore {
            rows,
            active: active_row,
            onactive: move |row| state.set_active(Some(row)),
            opened: state.opened(),
            onopened: move |opened| state.set_opened(opened),
            id: state.id(),
            empty: props.empty,
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
