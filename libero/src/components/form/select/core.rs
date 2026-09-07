use dioxus::prelude::*;

use crate::{
    components::{
        ComboboxCore, ComboboxOption, HtmlTag, Input, States, VisuallyHidden,
        common::{ChevronDownIcon, attr, field_props, focus_ring_sx, ring_overlay},
        form::{
            clear_button, field_control_sx, use_chip_announcer, use_field, use_field_frame,
            use_refocus_on_close,
        },
        layout::use_box,
        use_combobox,
    },
    hooks::{
        PopoverWidth, TYPEAHEAD_RESET, typeahead_match, use_element, use_theme, use_typeahead,
    },
    platform::ElementApi,
    sx::{StaticSx, sx},
};

/// A single select's trigger is the frame's control: one line, the selection
/// or the placeholder, and the chevron at its end - inside the control rather
/// than in the frame's trailing slot, so a click on the chevron opens the list
/// too. A multi-select's trigger sits in `MULTI_VALUE_SX`'s flow instead.
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
        .selector("& > [data-placeholder]", sx().color("text-dimmed"))
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("grey.6"),
        )
        // In the chips' flow, after the last one. A zero basis keeps it on the
        // last row whatever is left there, and the negative margin cancels
        // the gap before it, so it never wraps onto a row of its own and the
        // chips break exactly where they did inside the trigger.
        .when(
            "multiple",
            sx().flex("1 1 0")
                .selector("&:not(:first-child)", sx().margin_left("-4px")),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// `MultiSelect`'s control: the chips and the trigger on one wrapping flow.
///
/// The chips sit beside the `role="combobox"` element, not in it. Inside it,
/// each chip's x was part of the combobox's value, which read "Cherry Remove
/// Cherry". The trigger carries the selection as hidden text instead, and this
/// slot takes the clicks the trigger used to own.
static MULTI_VALUE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .align_items("center")
        .gap("4px")
        .flex("1 1 auto")
        // Without it a long chip pushes the frame wider instead of wrapping.
        .min_width("0")
        .cursor("pointer")
        .user_select("none")
        // One wrapper per selected item: it carries the id
        // `aria-activedescendant` points at, and nothing visual. `min-width: 0`,
        // or a long option's chip is floored at its whole label before the
        // chip's own ellipsis can apply.
        .selector(
            "& > [data-slot='chip']",
            sx().display("inline-flex").max_width("100%").min_width("0"),
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
            "& > [data-slot='chip'][data-cursor='true'] > *",
            focus_ring_sx(),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// `MultiSelect`'s chevron, in the frame's trailing slot rather than at the
/// end of the trigger, where it would sit on the last chip row. The frame's
/// gap is wider than the trigger's was, so the margin gives the difference
/// back and the chevron stays where it was.
static MULTI_CHEVRON_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .margin_left("-4px")
        .cursor("pointer")
        .selector("& > svg", sx().width("1em").height("1em").color("grey.6"))
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
        .selector("::placeholder", sx().color("text-dimmed"))
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
        /// One group label per row, parallel to `rows`, `None` for a row in no
        /// group. Filtered alongside the rows and handed on, so a search
        /// narrows the groups with the list.
        #[props(default)]
        groups: Vec<Option<String>>,
        /// One flag per row, parallel to `rows`. A disabled row is drawn and
        /// read out; the arrows, typeahead and the click all pass over it.
        #[props(default)]
        row_disabled: Vec<bool>,
        /// `Some(label)` while the options are being fetched: the list shows a
        /// loader in place of the rows and the empty state, and a status region
        /// beside the trigger says `label`.
        #[props(default)]
        loading: Option<String>,
        onpick: EventHandler<usize>,
        /// Drawn inside the trigger - beside it when `multiple` - as a
        /// function of the chip cursor - the
        /// skin owns no state, so the cursor lives here and the selection is
        /// redrawn from it. `None` shows `placeholder`.
        #[props(default)]
        selection: Option<Callback<SelectionRenderArgs, Element>>,
        /// The selection's labels, in order. A `multiple` trigger carries them
        /// as its hidden value text - the chips sit beside it - and its live
        /// region announces what they gained or lost.
        #[props(default)]
        value_labels: Vec<String>,
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
        /// Emits one hidden input of that name per `form_values` entry, so the
        /// selection posts with a native form. The trigger is a `div` and
        /// cannot carry a `name` itself.
        #[props(default)]
        name: Option<String>,
        /// What those hidden inputs post, one `Options::value()` each - the
        /// shape a native `<select multiple>` sends.
        #[props(default)]
        form_values: Vec<String>,
        /// What the skin's `validate` rules say; `T` never reaches here.
        #[props(default)]
        rules: Option<crate::components::FieldStatus>,
        /// Each row's plain text, parallel to `rows`, which is what the
        /// typeahead searches. Empty turns typeahead off.
        ///
        /// A `Vec<String>` rather than a callback, for the same reason
        /// `selected` is a `Vec<bool>`: it erases the skin's `T` just as well,
        /// and it lets the core do the matching itself against the disabled
        /// mask it already holds. `matches` cannot serve - it carries the
        /// caller's `filter` semantics, and typeahead is
        /// prefix-from-current-with-wrap.
        #[props(default)]
        row_labels: Vec<String>,
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
    // The list is the only editor, so read-only keeps the trigger focusable
    // and posting and refuses to open it. The chip keys and the clear button
    // go with it, and `aria-readonly` says so on the `combobox` role.
    let readonly = props.readonly.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    let state = use_combobox();
    let opened = state.is_open() && !disabled && !readonly;
    let searchable = props.searchable && !disabled;

    // The typed characters, forgotten after a pause - the buffer a native
    // `<select>` keeps, so "b","e","r" inside the window finds Berlin while a
    // lone "b" after it cycles the B rows. A hook, so it is prepared whether
    // or not this list answers typeahead at all.
    let typeahead = use_typeahead(TYPEAHEAD_RESET);
    // Never while `searchable`: there the search box is the affordance, and
    // once the list is open the focus is inside it anyway.
    let typeahead_on = !searchable && !props.row_labels.is_empty();

    // The query lives here, beside the open and highlight state. The skins
    // stay stateless: they hand down `matches` and nothing else.
    let mut query = use_signal(String::new);
    // Which chip the keyboard is on. An index into the *selection* order, which
    // is the skin's `value`, not into `rows`.
    let mut cursor = use_signal(|| None::<usize>);
    let search = use_element();
    let trigger_element = use_element();

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
        if next && !state.is_open() {
            state.set_active(Some(first_selected));
            // One `aria-activedescendant`, one owner: the open list takes it.
            // A local copy, so `open` stays `Fn` for the callers that share it.
            let mut cursor = cursor;
            cursor.set(None);
        }
        state.set_open(next);
    };

    use_refocus_on_close(opened, searchable, trigger_element, query);

    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.rules.clone())
        .name(props.name.as_deref())
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
    let clear = clear_button(
        props.clearable && has_selection && !disabled && !readonly,
        size,
        trigger_element,
        move |_| onclear.call(()),
    );

    // Where typeahead searches from: the highlight while the list is open, the
    // selection while it is closed. Neither starts at the top.
    let typeahead_from = match state.is_open() {
        true => state.active(),
        false => props.selected.iter().position(|selected| *selected),
    };
    let labels = props.row_labels.clone();
    let typed_mask = props.row_disabled.clone();
    let typed_pick = props.onpick;
    let typed_multiple = props.multiple;
    // One typed character, folded into the buffer and acted on. `true` when it
    // landed on a row, which is what decides whether the key was ours.
    // A clone, so the keydown arms can still ask the buffer whether a space is
    // being typed. Both halves share one `Rc`, so it is the same buffer.
    let buffer = typeahead.clone();
    let type_to = move |ch: char| {
        let query = buffer.push(ch);
        let found = typeahead_match(labels.len(), typeahead_from, &query, |row| match typed_mask
            .get(row)
            .copied()
            .unwrap_or(false)
        {
            true => None,
            false => labels.get(row).map(String::as_str),
        });
        let Some(row) = found else {
            return false;
        };
        match (state.is_open(), typed_multiple) {
            // An open list moves its highlight and picks nothing: Enter or a
            // click still commits, exactly as with the arrows.
            (true, _) => state.set_active(Some(row)),
            // A closed single select changes its value in place, the way a
            // native `<select>` does.
            (false, false) => typed_pick.call(row),
            // A closed multi-select opens instead. A pick there *toggles*, so
            // typing would silently drop a value the caller had chosen - the
            // one thing the native control never has to worry about.
            (false, true) => {
                open(true);
                state.set_active(Some(row));
            }
        }
        true
    };

    let multiple = props.multiple;
    // The trigger no longer spans the chips, so the slot around them and the
    // chevron each open the list and hand the focus to the trigger. Read
    // before the focus moves: a searchable list closes on its box's blur.
    let toggle = move || {
        if disabled {
            return;
        }
        let next = !state.is_open();
        let _ = trigger_element.focus();
        if !readonly {
            open(next);
        }
    };

    // Hooks, so all three are prepared whether or not this is a multi-select.
    let value_slot = use_box()
        .framework_sx(&MULTI_VALUE_SX)
        .focus_ring(false)
        .states(field.states())
        .prepare();
    let chevron_box = use_box()
        .framework_sx(&MULTI_CHEVRON_SX)
        .states(field.states())
        .prepare();
    let announcer = use_chip_announcer(props.value_labels.clone());

    // A multi-select's chevron is in the trailing slot, where the clear
    // button replaces it just as it does inside a single select's trigger.
    // Neither takes the focus: it stays on the trigger, whose blur closes the
    // list.
    let trailing = match (multiple, &clear) {
        (true, None) => Some(
            chevron_box
                .event("onmousedown", |event: MouseEvent| event.prevent_default())
                .event("onclick", move |_: MouseEvent| toggle())
                .render(HtmlTag::Span, Vec::new(), rsx! { ChevronDownIcon {} }),
        ),
        _ => clear.clone(),
    };

    let frame = use_field_frame()
        .trailing(&trailing)
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

    // Filtered exactly as `rows` is, so the three stay parallel: a search that
    // empties a group simply leaves no row carrying its label, and the group
    // is gone from the list.
    let groups: Vec<Option<String>> = visible
        .iter()
        .map(|index| props.groups.get(*index).cloned().flatten())
        .collect();
    let row_disabled: Vec<bool> = visible
        .iter()
        .map(|index| props.row_disabled.get(*index).copied().unwrap_or(false))
        .collect();

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
            .attr("data-controlled", true)
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
    let drawn = props.selection.map(|selection| {
        selection.call(SelectionRenderArgs {
            cursor: chip_cursor,
            id_prefix: id_prefix.clone(),
        })
    });
    // A multi-select's chips go beside the trigger, so what it holds is said
    // inside it as text - the combobox's value - and shown by the chips.
    let (content, chips) = match (drawn, multiple) {
        (Some(drawn), false) => (rsx! { span { "data-slot": "value", {drawn} } }, None),
        (Some(drawn), true) => {
            let spoken = props.value_labels.join(", ");
            (rsx! { VisuallyHidden { "{spoken}" } }, Some(drawn))
        }
        (None, _) => (
            rsx! {
                span { "data-slot": "value", "data-placeholder": "true", "{placeholder}" }
            },
            None,
        ),
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
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        // A multi-select's click bubbles to the slot around it, which owns it.
        .event("onclick", move |_: MouseEvent| {
            if !multiple && !disabled && !readonly {
                open(!state.is_open());
            }
        })
        // Two keyboards on one element. The chips answer Left, Right and
        // Backspace, which `ComboboxCore` leaves alone; the list answers the
        // rest. While `searchable` and open the focus is in the search box, so
        // none of this fires and Backspace only ever edits the query.
        .event("onkeydown", move |event: KeyboardEvent| {
            if disabled || readonly {
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
                Key::Enter if !state.is_open() => {
                    event.prevent_default();
                    open(true);
                }
                // A browser or OS shortcut is never typeahead.
                Key::Character(_) if has_shortcut_modifier(&event) => {}
                // A space mid-query is part of "new york", not an activation.
                Key::Character(ref key) if key == " " && typeahead_on && typeahead.is_typing() => {
                    event.prevent_default();
                    type_to(' ');
                }
                Key::Character(ref key) if key == " " && !state.is_open() => {
                    event.prevent_default();
                    open(true);
                }
                Key::Character(ref key) if typeahead_on && key != " " => {
                    // Only when it lands somewhere: a key that matches no row
                    // still belongs to the page, as it does in a native
                    // control.
                    if let Some(ch) = key.chars().next()
                        && type_to(ch)
                    {
                        event.prevent_default();
                    }
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
                if !multiple && clear.is_none() {
                    ChevronDownIcon {}
                }
            },
        );

    // The chips and the trigger share one wrapping flow, the shape `TagsField`
    // has. The slot cancels `mousedown`, so a press anywhere in it - a chip, its
    // x, the trigger itself - leaves the focus where it was, and a click
    // focuses the trigger by hand. The trigger is not the frame's child any
    // more, so it needs a ring overlay of its own as its sibling.
    let control = match multiple {
        true => value_slot
            .event("onmousedown", |event: MouseEvent| event.prevent_default())
            .event("onclick", move |_: MouseEvent| toggle())
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
                    {chips}
                    {trigger}
                    {ring_overlay()}
                },
            ),
        false => trigger,
    };

    let listbox = rsx! {
        ComboboxCore {
            rows,
            groups,
            row_disabled,
            loading: props.loading,
            active: state.active(),
            onactive: move |row| state.set_active(Some(row)),
            opened,
            onopened: open,
            state,
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
            {frame.render(control)}
        }
    };

    // A hidden input is the only way a control that is not a form element can
    // post - the shape `Slider` and `PinField` use. `disabled` goes on it too,
    // so a disabled select sends nothing.
    let hidden = props.name.map(|name| {
        rsx! {
            for value in props.form_values.iter().cloned() {
                input {
                    r#type: "hidden",
                    name: name.clone(),
                    value,
                    disabled: disabled.then_some(true),
                }
            }
        }
    });

    field.render(rsx! {
        {listbox}
        {hidden}
        if multiple {
            {announcer}
        }
    })
}

// Shift is part of ordinary typing; the rest mark a browser or OS shortcut.
// `Menu`'s rule, and its fourth private copy - worth one helper one day.
fn has_shortcut_modifier(event: &KeyboardEvent) -> bool {
    let modifiers = event.modifiers();
    modifiers.ctrl() || modifiers.alt() || modifiers.meta()
}
