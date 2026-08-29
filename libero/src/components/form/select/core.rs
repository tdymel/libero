use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, ComboboxCore, ComboboxOption, HtmlTag, Input, States,
        common::{attr, field_props, focus_ring_sx},
        form::{
            field_control_sx,
            glyphs::{ChevronIcon, CloseIcon},
            use_field, use_field_frame,
        },
        layout::use_box,
        use_combobox,
    },
    hooks::{PopoverWidth, use_element, use_theme},
    platform::ElementApi,
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
                sx().display("flex")
                    .flex_wrap("wrap")
                    .gap("4px")
                    // The single-line slot clips its overflow for the
                    // ellipsis. Chips wrap instead, and that clip cut the
                    // bottom row and the cursor's ring off at the slot's edge.
                    .overflow("visible"),
            ),
        )
        // One wrapper per selected item: it carries the id
        // `aria-activedescendant` points at, and nothing visual.
        .selector(
            "& [data-slot='chip']",
            sx().display("inline-flex").max_width("100%"),
        )
        // A flex line of its own, or the button hangs off the label's baseline.
        .selector(
            "& [data-slot='remove']",
            sx().display("inline-flex").align_items("center"),
        )
        // The chip the keyboard is on. The ring goes on what the skin drew, not
        // on the wrapper, so it follows that element's own radius. The trigger
        // keeps the DOM focus the whole time, so nothing else marks it.
        .selector(
            "& [data-slot='chip'][data-cursor='true'] > *",
            focus_ring_sx(),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// The search box at the top of the list. It is not a field control - it sits
/// inside the dropdown, above the rows and outside their scroll - so it carries
/// its own chrome rather than the field frame's.
static SEARCH_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .border("none")
        .outline("none")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .line_height("1.5")
        .padding("4px 8px")
        .border_bottom("1px solid")
        .border_color("grey.3")
        .selector("::placeholder", sx().color("grey.6"))
});

/// What the skin needs to draw the selection: which chip the keyboard is on,
/// and the prefix its ids are built from, so `aria-activedescendant` on the
/// trigger has something to point at.
#[derive(Clone, PartialEq)]
pub(crate) struct SelectionRenderArgs {
    pub cursor: Option<usize>,
    pub id_prefix: String,
}

field_props! {
    pub(crate) struct SelectCoreProps {
        /// Each row's content, already drawn by the skin. The core wraps every
        /// one in a `ComboboxOption`, which is what wires `aria-selected`, the
        /// highlight and the pick.
        rows: Vec<Element>,
        /// Parallel to `rows`.
        selected: Vec<bool>,
        onpick: EventHandler<usize>,
        /// Drawn inside the trigger, as a function of the chip cursor - the
        /// skin owns no state, so the cursor lives here and the selection is
        /// redrawn from it. `None` shows `placeholder`.
        #[props(default)]
        selection: Option<Callback<SelectionRenderArgs, Element>>,
        /// How many removable chips `selection` draws. The core moves a cursor
        /// over items it cannot see, so it has to be told how many there are.
        #[props(default)]
        chip_count: usize,
        /// Called with an index into the selection order, never into `rows`.
        /// Without it the chip keys do nothing.
        #[props(default)]
        onremove: Option<EventHandler<usize>>,
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
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: bool,
        #[props(default)]
        search_placeholder: Option<String>,
        /// Which rows survive the query, one `bool` per row. The skin closes
        /// over its own `Vec<T>` and the caller's filter, so `T` never reaches
        /// here - the mask is the same erasure `rows: Vec<Element>` performs.
        #[props(default)]
        matches: Option<Callback<String, Vec<bool>>>,
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
    let searchable = props.searchable && !disabled;

    // The query lives here, beside the open and highlight state. The skins
    // stay stateless: they hand down `matches` and nothing else.
    let mut query = use_signal(String::new);
    // Which chip the keyboard is on. An index into the *selection* order, which
    // is the skin's `value`, not into `rows`.
    let mut cursor = use_signal(|| None::<usize>);
    let search = use_element();
    let trigger_element = use_element();
    let mut was_open = use_signal(|| false);

    // Which rows survive the query, and what each one's index was in the full
    // list. `onpick` reports the original index, so the skins never remap.
    let visible: Vec<usize> = match (searchable, props.matches.as_ref(), query().is_empty()) {
        (true, Some(matches), false) => matches
            .call(query())
            .into_iter()
            .enumerate()
            .filter(|(_, keep)| *keep)
            .map(|(index, _)| index)
            .collect(),
        _ => (0..props.rows.len()).collect(),
    };

    // Removing the chip under the cursor leaves the index pointing at the one
    // that took its place; past the end it clamps to the new last, and with no
    // chips left there is nothing to point at.
    let chip_count = props.chip_count;
    let chip_cursor = match chip_count {
        0 => None,
        count => cursor().map(|index| index.min(count - 1)),
    };

    let has_selection = props.selected.iter().any(|selected| *selected);
    // A list opens on what is already selected, like a native `<select>` -
    // counted among the rows actually on screen.
    let first_selected = visible
        .iter()
        .position(|index| props.selected.get(*index).copied().unwrap_or(false))
        .unwrap_or(0);
    let open = move |next: bool| {
        if next && !state.opened() {
            state.set_active(Some(first_selected));
            // One `aria-activedescendant`, one owner: the open list takes it.
            // A local copy, so `open` stays `Fn` for the callers that share it.
            let mut cursor = cursor;
            cursor.set(None);
        }
        state.set_opened(next);
    };

    // Closing clears the query and hands focus back to the trigger, which would
    // otherwise be lost to the body - the box the user was typing in has just
    // unmounted.
    //
    // Opening is deliberately *not* handled here. The list is
    // `visibility: hidden` until `use_popover` has measured it, and focusing a
    // hidden element does nothing while still reporting success, so focusing
    // the box on mount never took. `ComboboxCore` does it instead, once the box
    // is on screen - that is what `autofocus` is.
    use_effect(use_reactive!(|(opened, searchable)| {
        if !searchable {
            return;
        }
        // `peek`, so writing it below cannot re-trigger this effect forever.
        let previously = *was_open.peek();
        if previously && !opened {
            query.set(String::new());
            let _ = trigger_element.focus();
        }
        if previously != opened {
            was_open.set(opened);
        }
    }));

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
    let onremove = props.onremove;
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
    let rows: Vec<Element> = visible
        .iter()
        .copied()
        .filter_map(|index| {
            let row = props.rows.get(index)?.clone();
            let selected = props.selected.get(index).copied().unwrap_or(false);
            Some(rsx! {
                ComboboxOption {
                    selected,
                    // `index` is the row's place in the *full* list, so a
                    // filtered list still reports what the skin expects.
                    onpick: move |_| {
                        onpick.call(index);
                        if close_on_pick {
                            state.close();
                        }
                    },
                    {row}
                }
            })
        })
        .collect();

    // A hook, so it is prepared unconditionally and only used when searching.
    let search_box = use_box().framework_sx(&SEARCH_SX).prepare();
    let search_placeholder = props.search_placeholder.clone().unwrap_or_default();
    let header = searchable.then(|| {
        search_box
            .element(&search)
            .attr_default("type", "text")
            .attr("value", query())
            .attr("placeholder", search_placeholder)
            // Ours is the list underneath; the browser's would cover it.
            .attr("autocomplete", "off")
            .attr("aria-autocomplete", "list")
            .event("oninput", move |event: FormEvent| {
                query.set(event.value());
                // The list under the highlight just changed; arm its top row.
                state.set_active(Some(0));
            })
            // The trigger's blur no longer closes while searchable - this does,
            // and the rows and the list cancel `mousedown`, so a click inside
            // never reaches it.
            .event("onblur", move |_: FocusEvent| state.close())
            .render(HtmlTag::Input, state.a11y_attributes(), ())
    });

    let placeholder = props.placeholder.clone().unwrap_or_default();
    let id_prefix = format!("{}-chip", state.id());
    let content = match props.selection {
        Some(selection) => {
            let drawn = selection.call(SelectionRenderArgs {
                cursor: chip_cursor,
                id_prefix: id_prefix.clone(),
            });
            rsx! {
                span { "data-slot": "value", {drawn} }
            }
        }
        None => rsx! {
            span { "data-slot": "value", "data-placeholder": "true", "{placeholder}" }
        },
    };

    // Two elements cannot both be the combobox. While the search box is open it
    // owns the role, `aria-controls` and `aria-activedescendant`; the trigger
    // keeps only what says a list hangs off it.
    let searching = searchable && opened;
    let mut attributes = match searching {
        true => vec![
            attr("aria-haspopup", "listbox"),
            attr("aria-expanded", "true"),
        ],
        false => state.a11y_attributes(),
    };
    // Only while closed: the open list is the other owner of this attribute.
    if let Some(index) = chip_cursor.filter(|_| !opened) {
        attributes.push(attr(
            "aria-activedescendant",
            format!("{id_prefix}-{index}"),
        ));
    }
    attributes.extend(props.attributes);
    let trigger = field
        .aria(control)
        .element(&trigger_element)
        .attr("aria-labelledby", field.label_id())
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        .event("onclick", move |_: MouseEvent| {
            if !disabled {
                open(!state.opened());
            }
        })
        // Two keyboards on one element. The chips answer Left, Right and
        // Backspace, which `ComboboxCore` leaves alone; the list answers the
        // rest. While `searchable` and open the focus is in the search box, so
        // none of this fires and Backspace only ever edits the query.
        .event("onkeydown", move |event: KeyboardEvent| {
            if disabled {
                return;
            }
            match event.key() {
                Key::ArrowLeft if chip_count > 0 => {
                    event.prevent_default();
                    let index = match chip_cursor {
                        Some(index) => index.saturating_sub(1),
                        // From no cursor, the last chip - the one Backspace
                        // would have taken.
                        None => chip_count - 1,
                    };
                    cursor.set(Some(index));
                }
                Key::ArrowRight if chip_count > 0 => {
                    event.prevent_default();
                    cursor.set(match chip_cursor {
                        Some(index) if index + 1 < chip_count => Some(index + 1),
                        // Past the last chip is back to no cursor, not a wrap.
                        _ => None,
                    });
                }
                Key::Backspace | Key::Delete if chip_count > 0 => {
                    let Some(onremove) = onremove else {
                        return;
                    };
                    event.prevent_default();
                    onremove.call(chip_cursor.unwrap_or(chip_count - 1));
                }
                // `ComboboxCore` opens on ArrowDown; a select-only combobox
                // opens on Enter and Space as well. Enter on an *open* list is
                // the core's pick.
                Key::Enter if !state.opened() => {
                    event.prevent_default();
                    open(true);
                }
                Key::Character(ref key) if key == " " && !state.opened() => {
                    event.prevent_default();
                    open(true);
                }
                _ => {}
            }
        })
        // While searchable the focus moves into the search box, so closing on
        // the trigger's blur would shut the list before a key could land.
        .event("onblur", move |_: FocusEvent| {
            if !searchable {
                state.close();
            }
        })
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
            onactive: move |row| state.set_active(Some(row)),
            opened,
            onopened: open,
            id: state.id(),
            size,
            radius,
            disabled,
            close_on_pick,
            multiselectable: props.multiple,
            header,
            // Focused once the list has been measured and is visible. Doing it
            // any earlier is a no-op that reports success.
            autofocus: searchable.then_some(search),
            width: PopoverWidth::Min,
            // A pick on a multi-select adds or drops a chip, which resizes the
            // trigger under an open list.
            remeasure: props.selected.iter().filter(|selected| **selected).count() as u64,
            {frame.render(trigger)}
        }
    };

    field.render(listbox)
}
