use dioxus::prelude::*;

use crate::{
    components::{ActionIcon, Chip, Input, Options, common::field_props, form::glyphs::CloseIcon},
    hooks::use_theme,
    sx::{ThemeAwareValue, sx},
    theme::{CHIP_HEIGHT, Size},
    utils::warn,
};

use super::{
    core::{SelectCore, SelectionRenderArgs},
    select::{SelectFilterArgs, SelectOptionArgs, SelectSelectionArgs, draw_rows},
};

field_props! {
    pub struct MultiSelectProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. Empty shows
        /// `placeholder`.
        #[props(default)]
        value: Vec<T>,
        /// Called with the whole selection the caller should hold next.
        #[props(default)]
        onchange: Option<EventHandler<Vec<T>>>,
        /// The options to list. Defaults to every `Options::options()` - which
        /// `String` and any other runtime type leave empty, so those pass them
        /// here.
        #[props(default)]
        options: Option<Vec<T>>,
        /// Draws one row's content. Defaults to `Options::label`.
        #[props(default)]
        option: Option<Callback<SelectOptionArgs<T>, Element>>,
        /// Draws one selected value inside the trigger. Defaults to the label
        /// in a `Chip` with an x. A caller who overrides it draws the whole
        /// chip, remove control included - `args.remove` is the wiring, and the
        /// keyboard stays the control's either way.
        #[props(default)]
        selection: Option<Callback<SelectSelectionArgs<T>, Element>>,
        /// Shown while `value` is empty.
        #[props(default)]
        placeholder: Option<String>,
        /// Emits a hidden input of that name carrying every selected option's
        /// `Options::value()`, comma-joined. The trigger is a `div`, so it
        /// cannot carry the name itself.
        #[props(default, into)]
        name: Option<String>,
        /// Shows an x that empties the selection.
        #[props(default)]
        clearable: Option<bool>,
        /// Puts a search box at the top of the list. The query survives a pick,
        /// so several matches of one search can be ticked without retyping it.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the options while searching. Defaults to a case-insensitive
        /// `contains` over `Options::label`.
        #[props(default)]
        filter: Option<Callback<SelectFilterArgs<T>, bool>>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
    }
}

/// A listbox over an enum that holds any number of its options.
///
/// Stays open on a pick, and a pick toggles the row: Escape, clicking
/// elsewhere and the trigger close it. The selection is drawn in the trigger,
/// as chips unless `selection` says otherwise - in the order it was picked.
#[component]
pub fn MultiSelect<T: Options>(props: MultiSelectProps<T>) -> Element {
    let theme = use_theme();

    if props.onchange.is_none() {
        warn("MultiSelect: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("MultiSelect: no options - a `T` without static `options()` needs `options`.");
    }

    let selected: Vec<bool> = values
        .iter()
        .map(|option| props.value.contains(option))
        .collect();
    let rows = draw_rows(&values, &selected, props.option.as_ref());
    // The chips, redrawn from the cursor the core owns. Each wrapper carries the
    // id `aria-activedescendant` points at; what is inside it is the skin's, or
    // the caller's.
    let onchange = props.onchange;
    let size = props.size.copied_or(theme.multi_select.size);
    // Chips ride inside the control, so they sit one step down the same scale
    // the field is on - `xs` has nowhere lower to go.
    let chip_size = Size::ALL[size.index().saturating_sub(1)];
    let picked = props.value.clone();
    let draw_selection = props.selection;
    let selection = (!picked.is_empty()).then(|| {
        Callback::new(move |args: SelectionRenderArgs| {
            let chips = picked.iter().cloned().enumerate().map(|(index, value)| {
                let removing = picked.clone();
                let remove = Callback::new(move |_: ()| drop_at(&removing, index, &onchange));
                match &draw_selection {
                    Some(selection) => selection.call(SelectSelectionArgs { value, remove }),
                    None => default_chip(&value, remove, chip_size),
                }
            });
            rsx! {
                for (index, chip) in chips.enumerate() {
                    span {
                        key: "{index}",
                        "data-slot": "chip",
                        id: "{args.id_prefix}-{index}",
                        "data-cursor": (args.cursor == Some(index)).then_some("true"),
                        {chip}
                    }
                }
            }
        })
    });

    // The same mask `Select` builds, and the same memoization reasoning - see
    // the comment there.
    let searchable = props.searchable.unwrap_or(false);
    let filter = props.filter;
    let filtered = values.clone();
    let matches = searchable.then(|| {
        Callback::new(move |query: String| {
            let needle = query.to_lowercase();
            filtered
                .iter()
                .map(|value| match &filter {
                    Some(filter) => filter.call(SelectFilterArgs {
                        value: value.clone(),
                        query: query.clone(),
                    }),
                    None => value.label().to_lowercase().contains(&needle),
                })
                // Annotated for the same reason as in `select.rs`.
                .collect::<Vec<bool>>()
        })
    });

    let current = props.value.clone();
    // Built before the pick closure takes `current`.
    let posted = form_value(&current);
    let removable = props.value.clone();
    rsx! {
        SelectCore {
            rows,
            selected,
            multiple: true,
            onpick: move |index: usize| {
                let (Some(onchange), Some(value)) = (&onchange, values.get(index)) else {
                    return;
                };
                let mut next = current.clone();
                match next.iter().position(|picked| picked == value) {
                    Some(at) => {
                        next.remove(at);
                    }
                    None => next.push(value.clone()),
                }
                onchange.call(next);
            },
            selection,
            chip_count: removable.len(),
            onremove: move |index: usize| drop_at(&removable, index, &onchange),
            placeholder: props.placeholder,
            name: props.name,
            form_value: posted,
            clearable: props.clearable.unwrap_or(false),
            searchable,
            search_placeholder: props.search_placeholder,
            matches,
            onclear: move |_| {
                if let Some(onchange) = &onchange {
                    onchange.call(Vec::new());
                }
            },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size,
            radius: props.radius.copied_or(theme.multi_select.radius),
            disabled: props.disabled,
            required: props.required,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
}

/// The default chip: the label, and an x that drops it.
fn default_chip<T: Options>(value: &T, remove: Callback<()>, size: Size) -> Element {
    let label = value.label();
    // A fraction of the chip's own height, not of its font: the two do not
    // scale at the same rate (20 -> 36px against 11 -> 15px), so an `em` x
    // shrinks against its chip as the field grows.
    let icon_size: Input<ThemeAwareValue> =
        ThemeAwareValue::String(format!("calc({} * 0.6)", CHIP_HEIGHT.value(size))).into();
    rsx! {
        Chip { size,
            "{label}"
            span {
                // Centred by the trigger's own sx - a bare inline span would
                // hang the button off the label's baseline.
                "data-slot": "remove",
                // The trigger holds the focus that keeps the list open, so the
                // press must not move it onto the button.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                // A remove is not a click on the trigger, which would open the
                // list under the chip that just went away.
                onclick: move |event: MouseEvent| event.stop_propagation(),
                ActionIcon {
                    aria_label: "Remove {label}",
                    size: icon_size,
                    // A native `<button>` inherits neither `color` nor
                    // `font-size` - it takes the UA's `buttontext` and 13.3px.
                    // `color` is the fix `Chip`'s removed `ondelete` needed.
                    //
                    // The hover tint is `currentColor` at 20%, so it reads on a
                    // filled chip and a tonal one alike without either knowing
                    // the other's colour.
                    sx: sx()
                        .color("inherit")
                        .font_size("inherit")
                        .border_radius("50%")
                        .selector(
                            "&:hover",
                            sx().background("color-mix(in srgb, currentColor 20%, transparent)"),
                        ),
                    // The control is one tab stop: the keys, not the buttons,
                    // are how a keyboard removes a chip.
                    tabindex: "-1",
                    onclick: move |_| remove.call(()),
                    CloseIcon {}
                }
            }
        }
    }
}

/// Every selected option's wire value, joined for the hidden input.
///
/// One comma-joined field rather than one input per value, which is what
/// Mantine sends too. A backend that wants `fruits[]` repeated has to split it;
/// which of the two libero should send is W5's to settle
/// ([[todos]] item 20 in the brain).
fn form_value<T: Options>(values: &[T]) -> Option<String> {
    match values.is_empty() {
        true => None,
        false => Some(
            values
                .iter()
                .map(Options::value)
                .collect::<Vec<_>>()
                .join(","),
        ),
    }
}

/// Drops one value from the selection - the same edit as picking its row again.
fn drop_at<T: Options>(values: &[T], index: usize, onchange: &Option<EventHandler<Vec<T>>>) {
    let Some(onchange) = onchange else {
        return;
    };
    if index >= values.len() {
        return;
    }
    let mut next = values.to_vec();
    next.remove(index);
    onchange.call(next);
}
