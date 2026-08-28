use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, ComboboxCore, ComboboxOption, HtmlTag, Input, States,
        common::field_props,
        form::{field_control_sx, use_field, use_field_frame},
        layout::use_box,
        use_combobox,
    },
    hooks::{PopoverWidth, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
};

/// The trigger is the frame's control: one line, the selection or the
/// placeholder, and the chevron at its end - inside the control rather than
/// in the frame's trailing slot, so a click on the chevron opens the list too.
static SELECT_TRIGGER_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            "& > [data-slot='value']",
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("& > [data-placeholder]", sx().color("grey.6"))
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("grey.6"),
        )
        .when(
            "multiple",
            sx().selector(
                "& > [data-slot='value']",
                sx().display("flex").flex_wrap("wrap").gap("4px"),
            ),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

field_props! {
    pub(crate) struct SelectCoreProps {
        /// Each row's content, already drawn by the skin. The core wraps every
        /// one in a `ComboboxOption`, which is what wires `aria-selected`, the
        /// highlight and the pick.
        rows: Vec<Element>,
        /// Parallel to `rows`.
        selected: Vec<bool>,
        onpick: EventHandler<usize>,
        /// Drawn inside the trigger. `None` shows `placeholder`.
        #[props(default)]
        selection: Option<Element>,
        #[props(default)]
        placeholder: Option<String>,
        /// Shows an x in place of the chevron while something is selected.
        #[props(default)]
        clearable: bool,
        onclear: EventHandler<()>,
        /// Stay open on a pick, mark the listbox `aria-multiselectable`, and
        /// let the selection wrap.
        #[props(default)]
        multiple: bool,
    }
}

/// The engine under `Select` and `MultiSelect`. It never sees `T`: the skins
/// hand it drawn rows and a selection, and take indices back.
///
/// Composes a field frame, `ComboboxCore` and a `use_combobox` state. The
/// trigger holds focus the whole time - the rows and the list cancel
/// `mousedown` - so losing it is what closes the list on an outside click,
/// with no window-level listener.
#[component]
pub(crate) fn SelectCore(props: SelectCoreProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.select.size);
    let radius = props.radius.copied_or(theme.select.radius);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    let state = use_combobox();
    let opened = state.opened() && !disabled;
    let has_selection = props.selected.iter().any(|selected| *selected);
    // A list opens on what is already selected, like a native `<select>`.
    let first_selected = props
        .selected
        .iter()
        .position(|selected| *selected)
        .unwrap_or(0);
    let open = move |next: bool| {
        if next && !state.opened() {
            state.set_active(first_selected);
        }
        state.set_opened(next);
    };

    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let onclear = props.onclear;
    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(size).into();
    let clear = (props.clearable && has_selection && !disabled).then(|| {
        rsx! {
            ActionIcon {
                aria_label: "Clear",
                size: icon_size,
                onclick: move |_| onclear.call(()),
                CloseIcon {}
            }
        }
    });

    let frame = use_field_frame()
        .trailing(&clear)
        .states(field.states())
        .prepare();

    let trigger_states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("multiple", props.multiple)
        .into();
    // The frame draws the ring, so the trigger must not draw a second one.
    let control = use_box()
        .framework_sx(&SELECT_TRIGGER_SX)
        .focus_ring(false)
        .states(&trigger_states)
        .prepare();

    let onpick = props.onpick;
    let close_on_pick = !props.multiple;
    let rows: Vec<Element> = props
        .rows
        .into_iter()
        .zip(props.selected.iter().copied())
        .enumerate()
        .map(|(index, (row, selected))| {
            rsx! {
                ComboboxOption {
                    selected,
                    onpick: move |_| {
                        onpick.call(index);
                        if close_on_pick {
                            state.close();
                        }
                    },
                    {row}
                }
            }
        })
        .collect();

    let placeholder = props.placeholder.clone().unwrap_or_default();
    let content = match props.selection {
        Some(selection) => rsx! {
            span { "data-slot": "value", {selection} }
        },
        None => rsx! {
            span { "data-slot": "value", "data-placeholder": "true", "{placeholder}" }
        },
    };

    let mut attributes = state.a11y_attributes();
    attributes.extend(props.attributes);
    let trigger = field
        .aria(control)
        .attr("aria-labelledby", field.label_id())
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        .event("onclick", move |_: MouseEvent| {
            if !disabled {
                open(!state.opened());
            }
        })
        // `ComboboxCore` opens on ArrowDown; a select-only combobox opens on
        // Enter and Space as well. Enter on an *open* list is the core's pick.
        .event("onkeydown", move |event: KeyboardEvent| {
            if disabled || state.opened() {
                return;
            }
            let opens = match event.key() {
                Key::Enter => true,
                Key::Character(key) => key == " ",
                _ => false,
            };
            if opens {
                event.prevent_default();
                open(true);
            }
        })
        .event("onblur", move |_: FocusEvent| state.close())
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {content}
                if clear.is_none() {
                    ChevronIcon {}
                }
            },
        );

    let listbox = rsx! {
        ComboboxCore {
            rows,
            active: state.active(),
            onactive: move |row| state.set_active(row),
            opened,
            onopened: open,
            id: state.id(),
            size,
            radius,
            disabled,
            close_on_pick,
            multiselectable: props.multiple,
            width: PopoverWidth::Min,
            // A pick on a multi-select adds or drops a chip, which resizes the
            // trigger under an open list.
            remeasure: props.selected.iter().filter(|selected| **selected).count() as u64,
            {frame.render(trigger)}
        }
    };

    field.render(listbox)
}

/// libero ships no icon set, so the two glyphs a select cannot do without live
/// here.
#[component]
fn ChevronIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M6 9l6 6 6-6" }
        }
    }
}

#[component]
fn CloseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 6L6 18" }
            path { d: "M6 6l12 12" }
        }
    }
}
