use dioxus::prelude::*;

use crate::{
    components::{
        ComboboxCore, ComboboxOption, ComboboxState, HtmlTag, Input, States, VisuallyHidden,
        common::{
            ChevronDownIcon, attr, field_props, focus_ring_sx, has_shortcut_modifier, ring_overlay,
        },
        form::{
            clear_button, field_control_sx, use_chip_announcer, use_field, use_field_frame,
            use_refocus_on_close,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{
        ElementHandle, PopoverWidth, TYPEAHEAD_RESET, Typeahead, typeahead_match, use_element,
        use_theme, use_typeahead,
    },
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::Size,
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
                .color("muted.6"),
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
        .selector("& > svg", sx().width("1em").height("1em").color("muted.6"))
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
        .border_color("muted.3")
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

/// A skin's own drawing reads only compared props, so it may compare equal; a
/// caller's never does, since its closure can read state no prop carries.
#[derive(Clone, Copy)]
pub(crate) struct SelectionDraw {
    draw: Callback<SelectionRenderArgs, Element>,
    by_caller: bool,
}

impl SelectionDraw {
    pub(crate) fn new(draw: Callback<SelectionRenderArgs, Element>, by_caller: bool) -> Self {
        Self { draw, by_caller }
    }
}

impl PartialEq for SelectionDraw {
    fn eq(&self, other: &Self) -> bool {
        !self.by_caller && !other.by_caller && self.draw == other.draw
    }
}

/// The selection, handed down in a signal rather than as props: a pick on a
/// closed select then redraws the trigger's value, not the whole field.
#[derive(Clone, PartialEq, Default)]
pub(crate) struct Picked {
    /// One per option, whether or not `rows` is drawn.
    pub selected: Vec<bool>,
    /// What the hidden inputs post, one `Options::value()` each - the shape a
    /// native `<select multiple>` sends.
    pub form_values: Vec<String>,
}

/// The skin's half of [`Picked`]: written in its render, and only on a change,
/// so a re-render that picked nothing wakes no reader.
pub(crate) fn use_picked(picked: Picked) -> Signal<Picked> {
    let mut signal = use_signal(|| picked.clone());
    if *signal.peek() != picked {
        signal.set(picked);
    }
    signal
}

field_props! {
    pub(crate) struct SelectCoreProps {
        /// Each row's content, drawn by the skin only while open: an empty
        /// list compares equal, so a closed select skips a re-render.
        rows: Vec<Element>,
        /// The selection. Read only where it is drawn, so a closed single
        /// select skips a pick and only its value redraws.
        picked: Signal<Picked>,
        /// Held by the skin, which draws `rows` only while it is open.
        state: ComboboxState,
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
        selection: Option<SelectionDraw>,
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
        /// What the skin's `validate` rules say; `T` never reaches here.
        #[props(default)]
        rules: Option<crate::components::FieldStatus>,
        /// Each row's plain text, parallel to `rows`, which is what the
        /// typeahead searches. Empty turns typeahead off.
        ///
        /// A `Vec<String>` rather than a callback, for the same reason
        /// `Picked::selected` is a `Vec<bool>`: it erases the skin's `T` just as well,
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

    let state = props.state;
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
    let query = use_signal(String::new);
    // Which chip the keyboard is on. An index into the *selection* order, which
    // is the skin's `value`, not into `rows`.
    let cursor = use_signal(|| None::<usize>);
    let search = use_element();
    let trigger_element = use_element();

    let picked = props.picked;
    let visible = visible_rows(searchable, props.matches, query, picked);

    // Removing the chip under the cursor leaves the index pointing at the one
    // that took its place; past the end it clamps to the new last, and with no
    // chips left there is nothing to point at.
    let chip_count = props.chip_count;
    let chip_cursor = match chip_count {
        0 => None,
        count => cursor().map(|index| index.min(count - 1)),
    };

    // Read, and so subscribed to, only where a pick changes what is drawn.
    let has_selection = props.clearable && picked.read().selected.iter().any(|selected| *selected);
    let open = SelectOpen {
        state,
        cursor,
        picked,
        query,
        matches: props.matches,
        searchable,
        trigger: trigger_element,
        disabled,
        readonly,
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
    let clear = clear_button(
        props.clearable && has_selection && !disabled && !readonly,
        size,
        trigger_element,
        move |_| onclear.call(()),
    );

    let typed = SelectTypeahead {
        // A clone of the hook's buffer, so the keydown arms can still ask it
        // whether a space is being typed. Both halves share one `Rc`, so it is
        // the same buffer.
        buffer: typeahead.clone(),
        labels: props.row_labels.clone(),
        row_disabled: props.row_disabled.clone(),
        multiple: props.multiple,
        onpick: props.onpick,
        open,
        on: typeahead_on,
    };

    let multiple = props.multiple;

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
                .event("onclick", move |_: MouseEvent| open.toggle())
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

    let (rows, groups, row_disabled) = select_rows(&props, &visible, state);

    // A hook, so it is prepared unconditionally and only used when searching.
    let search_box = use_box().framework_sx(&SEARCH_SX).prepare();
    let header = searchable.then(|| {
        select_search_box(
            search_box,
            search,
            state,
            query,
            props.search_placeholder.clone().unwrap_or_default(),
        )
    });

    let id_prefix = format!("{}-chip", state.id());
    let placeholder = props.placeholder.clone().unwrap_or_default();
    let (content, chips) = match multiple {
        true => {
            // The chips are drawn here, so a pick has to redraw this scope.
            picked.read();
            select_value_content(
                props.selection.map(|selection| {
                    selection.draw.call(SelectionRenderArgs {
                        cursor: chip_cursor,
                        id_prefix: id_prefix.clone(),
                    })
                }),
                &props.value_labels,
                &placeholder,
            )
        }
        false => (
            rsx! {
                SelectValue { picked, selection: props.selection, placeholder }
            },
            None,
        ),
    };

    let trigger = select_trigger(
        field
            .aria(control)
            .attr("aria-labelledby", field.label_id()),
        TriggerKeys {
            open,
            chip_count,
            chip_cursor,
            cursor,
            onremove: props.onremove,
        },
        typed,
        TriggerParts {
            element: trigger_element,
            id_prefix,
            opened,
            searchable,
            multiple,
            // A single select with nothing to clear draws its own chevron.
            chevron: !multiple && clear.is_none(),
        },
        content,
        props.attributes,
    );
    let control = select_control(multiple, value_slot, open, chips, trigger);

    let listbox = select_listbox(Listbox {
        rows,
        groups,
        row_disabled,
        header,
        control: frame.render(control),
        open,
        opened,
        size,
        radius,
        loading: props.loading,
        multiple,
        // A pick on a multi-select adds or drops a chip, which resizes the
        // trigger under an open list.
        remeasure: match opened {
            true => picked
                .read()
                .selected
                .iter()
                .filter(|selected| **selected)
                .count() as u64,
            false => 0,
        },
        // Focused once the list has been measured and is visible. Doing it any
        // earlier is a no-op that reports success.
        autofocus: searchable.then_some(search),
    });

    field.render(rsx! {
        {listbox}
        {select_hidden_inputs(props.name, picked, disabled)}
        if multiple {
            {announcer}
        }
    })
}

/// The dropdown, with the field frame - trigger, chips and all - as the thing
/// it hangs off.
struct Listbox {
    rows: Vec<Element>,
    groups: Vec<Option<String>>,
    row_disabled: Vec<bool>,
    header: Option<Element>,
    /// The frame, already rendered around the trigger.
    control: Element,
    open: SelectOpen,
    opened: bool,
    size: Size,
    radius: Size,
    loading: Option<String>,
    multiple: bool,
    remeasure: u64,
    /// The search box to focus once the list is measured, when there is one.
    autofocus: Option<ElementHandle>,
}

fn select_listbox(list: Listbox) -> Element {
    let Listbox {
        rows,
        groups,
        row_disabled,
        header,
        control,
        open,
        opened,
        size,
        radius,
        loading,
        multiple,
        remeasure,
        autofocus,
    } = list;
    let state = open.state;

    rsx! {
        ComboboxCore {
            rows,
            groups,
            row_disabled,
            loading,
            active: state.active(),
            onactive: move |row| state.set_active(Some(row)),
            opened,
            onopened: move |next| open.open(next),
            state,
            size,
            radius,
            disabled: open.disabled || open.readonly,
            close_on_pick: !multiple,
            multiselectable: multiple,
            header,
            autofocus,
            width: PopoverWidth::Min,
            remeasure,
            {control}
        }
    }
}

/// Opening and closing the list. It arms the row already selected on the way
/// in and drops the chip cursor, because one `aria-activedescendant` has one
/// owner.
#[derive(Clone, Copy)]
struct SelectOpen {
    state: ComboboxState,
    cursor: Signal<Option<usize>>,
    picked: Signal<Picked>,
    query: Signal<String>,
    matches: Option<Callback<String, Vec<bool>>>,
    searchable: bool,
    trigger: ElementHandle,
    disabled: bool,
    readonly: bool,
}

impl SelectOpen {
    /// A list opens on what is already selected, like a native `<select>` -
    /// counted among the rows actually on screen. Read at the press, since a
    /// pick on a closed select does not redraw the scope holding this.
    fn first_selected(self) -> usize {
        let picked = self.picked.peek();
        visible_rows(self.searchable, self.matches, self.query, self.picked)
            .iter()
            .position(|index| picked.selected.get(*index).copied().unwrap_or(false))
            .unwrap_or(0)
    }

    fn open(self, next: bool) {
        if next && !self.state.is_open() {
            self.state.set_active(Some(self.first_selected()));
            // One `aria-activedescendant`, one owner: the open list takes it.
            let mut cursor = self.cursor;
            cursor.set(None);
        }
        self.state.set_open(next);
    }

    /// The trigger no longer spans the chips, so the slot around them and the
    /// chevron each open the list and hand the focus to the trigger. The open
    /// state is read before the focus moves: a searchable list closes on its
    /// box's blur.
    fn toggle(self) {
        if self.disabled {
            return;
        }
        let next = !self.state.is_open();
        let _ = self.trigger.focus();
        if !self.readonly {
            self.open(next);
        }
    }
}

/// The typed-character buffer and everything it searches. A clone of the
/// hook's `Typeahead`, so this and the keydown arms share one buffer.
#[derive(Clone)]
struct SelectTypeahead {
    buffer: Typeahead,
    labels: Vec<String>,
    row_disabled: Vec<bool>,
    multiple: bool,
    onpick: EventHandler<usize>,
    open: SelectOpen,
    /// Off while `searchable`, and off with no row labels.
    on: bool,
}

impl SelectTypeahead {
    /// One typed character, folded into the buffer and acted on. `true` when
    /// it landed on a row, which is what decides whether the key was ours.
    fn type_to(&self, ch: char) -> bool {
        let state = self.open.state;
        let query = self.buffer.push(ch);
        // Where typeahead searches from: the highlight while the list is open,
        // the selection while it is closed. Neither starts at the top.
        let from = match state.is_open() {
            true => state.active(),
            false => self.open.picked.peek().selected.iter().position(|on| *on),
        };
        let found = typeahead_match(self.labels.len(), from, &query, |row| {
            match self.row_disabled.get(row).copied().unwrap_or(false) {
                true => None,
                false => self.labels.get(row).map(String::as_str),
            }
        });
        let Some(row) = found else {
            return false;
        };
        match (state.is_open(), self.multiple) {
            // An open list moves its highlight and picks nothing: Enter or a
            // click still commits, exactly as with the arrows.
            (true, _) => state.set_active(Some(row)),
            // A closed single select changes its value in place, the way a
            // native `<select>` does.
            (false, false) => self.onpick.call(row),
            // A closed multi-select opens instead. A pick there *toggles*, so
            // typing would silently drop a value the caller had chosen - the
            // one thing the native control never has to worry about.
            (false, true) => {
                self.open.open(true);
                state.set_active(Some(row));
            }
        }
        true
    }
}

/// What the trigger's two keyboards move.
#[derive(Clone, Copy)]
struct TriggerKeys {
    open: SelectOpen,
    chip_count: usize,
    chip_cursor: Option<usize>,
    cursor: Signal<Option<usize>>,
    onremove: Option<EventHandler<usize>>,
}

/// The trigger's drawn shape, and the ids it owns while the list is shut.
struct TriggerParts {
    element: ElementHandle,
    id_prefix: String,
    /// The list is open and usable.
    opened: bool,
    searchable: bool,
    multiple: bool,
    /// The trigger draws its own chevron: a single select with nothing to
    /// clear.
    chevron: bool,
}

/// The trigger: the one element `ComboboxCore` does not draw, and the one that
/// holds the focus the whole time.
fn select_trigger(
    control: BoxStyle,
    keys: TriggerKeys,
    typed: SelectTypeahead,
    parts: TriggerParts,
    content: Element,
    extra: Vec<Attribute>,
) -> Element {
    let TriggerParts {
        element,
        id_prefix,
        opened,
        searchable,
        multiple,
        chevron,
    } = parts;
    let open = keys.open;
    let state = open.state;
    let (disabled, readonly) = (open.disabled, open.readonly);

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
    if let Some(index) = keys.chip_cursor.filter(|_| !opened) {
        attributes.push(attr(
            "aria-activedescendant",
            format!("{id_prefix}-{index}"),
        ));
    }
    attributes.extend(extra);

    control
        .element(&element)
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        // A multi-select's click bubbles to the slot around it, which owns it.
        .event("onclick", move |_: MouseEvent| {
            if !multiple && !disabled && !readonly {
                open.open(!state.is_open());
            }
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            trigger_key(&event, keys, &typed)
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
                if chevron {
                    ChevronDownIcon {}
                }
            },
        )
}

/// Two keyboards on one element. The chips answer Left, Right and Backspace,
/// which `ComboboxCore` leaves alone; the list answers the rest. While
/// `searchable` and open the focus is in the search box, so none of this fires
/// and Backspace only ever edits the query.
fn trigger_key(event: &KeyboardEvent, keys: TriggerKeys, typed: &SelectTypeahead) {
    let TriggerKeys {
        open,
        chip_count,
        chip_cursor,
        mut cursor,
        onremove,
    } = keys;
    let state = open.state;
    if open.disabled || open.readonly {
        return;
    }
    match event.key() {
        Key::ArrowLeft if chip_count > 0 => {
            event.prevent_default();
            let index = match chip_cursor {
                Some(index) => index.saturating_sub(1),
                // From no cursor, the last chip - the one Backspace would have
                // taken.
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
        // `ComboboxCore` opens on ArrowDown; a select-only combobox opens on
        // Enter and Space as well. Enter on an *open* list is the core's pick.
        Key::Enter if !state.is_open() => {
            event.prevent_default();
            open.open(true);
        }
        // A browser or OS shortcut is never typeahead.
        Key::Character(_) if has_shortcut_modifier(event) => {}
        // A space mid-query is part of "new york", not an activation.
        Key::Character(ref key) if key == " " && typed.on && typed.buffer.is_typing() => {
            event.prevent_default();
            typed.type_to(' ');
        }
        Key::Character(ref key) if key == " " && !state.is_open() => {
            event.prevent_default();
            open.open(true);
        }
        Key::Character(ref key) if typed.on && key != " " => {
            // Only when it lands somewhere: a key that matches no row still
            // belongs to the page, as it does in a native control.
            if let Some(ch) = key.chars().next()
                && typed.type_to(ch)
            {
                event.prevent_default();
            }
        }
        _ => {}
    }
}

/// The chips and the trigger share one wrapping flow, the shape `TagsField`
/// has. The slot cancels `mousedown`, so a press anywhere in it - a chip, its
/// x, the trigger itself - leaves the focus where it was, and a click focuses
/// the trigger by hand. The trigger is not the frame's child any more, so it
/// needs a ring overlay of its own as its sibling.
fn select_control(
    multiple: bool,
    value_slot: BoxStyle,
    open: SelectOpen,
    chips: Option<Element>,
    trigger: Element,
) -> Element {
    match multiple {
        true => value_slot
            .event("onmousedown", |event: MouseEvent| event.prevent_default())
            .event("onclick", move |_: MouseEvent| open.toggle())
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
    }
}

/// Which rows survive the query, and what each one's index was in the full
/// list. `onpick` reports the original index, so the skins never remap.
fn visible_rows(
    searchable: bool,
    matches: Option<Callback<String, Vec<bool>>>,
    query: Signal<String>,
    picked: Signal<Picked>,
) -> Vec<usize> {
    match (searchable, matches, query.read().is_empty()) {
        (true, Some(matches), false) => matches
            .call(query())
            .into_iter()
            .enumerate()
            .filter(|(_, keep)| *keep)
            .map(|(index, _)| index)
            .collect(),
        // `selected`, not `rows`: a closed list draws no rows. Peeked, as the
        // count moves only with the options, which redraw the core anyway.
        _ => (0..picked.peek().selected.len()).collect(),
    }
}

/// The rows that survive the query, each wrapped in a `ComboboxOption`, plus
/// the group label and the disabled flag for each.
///
/// The three are filtered together so they stay parallel: a search that
/// empties a group simply leaves no row carrying its label, and the group is
/// gone from the list.
fn select_rows(
    props: &SelectCoreProps,
    visible: &[usize],
    state: ComboboxState,
) -> (Vec<Element>, Vec<Option<String>>, Vec<bool>) {
    if props.rows.is_empty() {
        return Default::default();
    }
    let groups = visible
        .iter()
        .map(|index| props.groups.get(*index).cloned().flatten())
        .collect();
    let row_disabled = visible
        .iter()
        .map(|index| props.row_disabled.get(*index).copied().unwrap_or(false))
        .collect();

    let onpick = props.onpick;
    let close_on_pick = !props.multiple;
    let picked = props.picked.read();
    let rows = visible
        .iter()
        .copied()
        .filter_map(|index| {
            let row = props.rows.get(index)?.clone();
            let selected = picked.selected.get(index).copied().unwrap_or(false);
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

    (rows, groups, row_disabled)
}

/// The search box at the top of the list. It owns the combobox role while it
/// is there, so it carries the a11y attributes the trigger gives up.
fn select_search_box(
    search_box: BoxStyle,
    search: ElementHandle,
    state: ComboboxState,
    mut query: Signal<String>,
    placeholder: String,
) -> Element {
    search_box
        .element(&search)
        .attr_default("type", "text")
        .attr("value", query())
        .attr("data-controlled", true)
        .attr("placeholder", placeholder)
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
}

/// What the trigger says it holds, and - for a multi-select - the chips that
/// go beside it. A multi-select's chips are not inside the trigger, so what it
/// holds is said inside it as text, the combobox's value, and shown by the
/// chips.
fn select_value_content(
    drawn: Option<Element>,
    value_labels: &[String],
    placeholder: &str,
) -> (Element, Option<Element>) {
    match drawn {
        Some(drawn) => {
            let spoken = value_labels.join(", ");
            (rsx! { VisuallyHidden { "{spoken}" } }, Some(drawn))
        }
        None => (placeholder_value(placeholder), None),
    }
}

fn placeholder_value(placeholder: &str) -> Element {
    rsx! {
        span { "data-slot": "value", "data-placeholder": "true", "{placeholder}" }
    }
}

/// A single select's value inside the trigger, in a scope of its own: a pick
/// rewrites `picked` and redraws this alone.
#[component]
fn SelectValue(
    picked: Signal<Picked>,
    selection: Option<SelectionDraw>,
    placeholder: String,
) -> Element {
    // Only subscribes: `selection` already draws the skin's newest value.
    picked.read();
    match selection {
        Some(selection) => {
            let drawn = selection.draw.call(SelectionRenderArgs {
                cursor: None,
                id_prefix: String::new(),
            });
            rsx! { span { "data-slot": "value", {drawn} } }
        }
        None => placeholder_value(&placeholder),
    }
}

/// A hidden input is the only way a control that is not a form element can
/// post - the shape `Slider` and `PinField` use. `disabled` goes on it too, so
/// a disabled select sends nothing.
fn select_hidden_inputs(
    name: Option<String>,
    picked: Signal<Picked>,
    disabled: bool,
) -> Option<Element> {
    name.map(|name| {
        let values = picked.read().form_values.clone();
        rsx! {
            for value in values.iter().cloned() {
                input {
                    r#type: "hidden",
                    name: name.clone(),
                    value,
                    disabled: disabled.then_some(true),
                }
            }
        }
    })
}
