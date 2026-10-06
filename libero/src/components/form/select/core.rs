use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{
            ComboboxState, Glyph, HtmlTag, Input, Part, Parts, States, TOOLBAR_ITEM, ToolbarItem,
            attr, focus_ring_sx, has_shortcut_modifier, inset_focus_ring_sx, navigation_chord,
            ring_overlay, use_toolbar_item,
        },
        form::{
            CaretKeys, ComboboxCore, ComboboxOption, DropdownPart, PreparedField, RowCache,
            clear_button, field_control_sx, field_parts_enum, field_props, use_chip_announcer,
            use_field, use_field_frame, use_refocus_on_close, use_row_cache,
            with_drawn_placeholder,
        },
        layout::{BoxStyle, use_box},
    },
    context::IconSlot,
    hooks::{
        ElementHandle, PopoverWidth, TYPEAHEAD_RESET, Typeahead, typeahead_match, use_element,
        use_localization, use_theme, use_typeahead,
    },
    platform::{ElementApi, blur_counts, logical_key},
    sx::{StaticSx, sx},
    theme::{FOCUS_RING_WIDTH, Size},
};

/// The chevron sits inside the control, so clicking it opens the list. A multi-select's trigger sits in `MULTI_VALUE_SX`'s flow.
static SELECT_TRIGGER_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        // The frame's height: empty, the trigger was 0px tall (todo 532, as 520).
        .align_self("stretch")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            format!("& > [data-slot='{}']", SelectPart::Value.slot()),
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
        // After the last chip: a zero basis and a margin cancelling the gap keep it off a row of its own.
        .when(
            "multiple",
            sx().flex("1 1 0")
                .selector("&:not(:first-child)", sx().margin_left("-4px"))
                .rtl(sx().selector(
                    "&:not(:first-child)",
                    sx().margin_left("0").margin_right("-4px"),
                )),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// `MultiSelect`'s chips and trigger in one wrapping flow. Chips sit beside the combobox,
/// or their x's joined its value ("Cherry Remove Cherry"); the trigger carries hidden text.
static MULTI_VALUE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .align_items("center")
        .gap("4px")
        .flex("1 1 auto")
        // Fills the frame, so the trigger can stretch to a whole chip row.
        .align_self("stretch")
        // Without it a long chip pushes the frame wider instead of wrapping.
        .min_width("0")
        .cursor("pointer")
        .user_select("none")
        // Carries the `aria-activedescendant` id. `min-width: 0` lets a long chip ellipsise.
        .selector(
            format!("& > [data-slot='{}']", SelectPart::Chip.slot()),
            sx().display("inline-flex").max_width("100%").min_width("0"),
        )
        // A flex line of its own, or the button hangs off the label's baseline.
        .selector(
            "& [data-slot='remove']",
            sx().display("inline-flex").align_items("center"),
        )
        // The keyboard's chip; the only mark, as the trigger keeps focus. On the drawn chip, for its radius.
        .selector(
            format!(
                "& > [data-slot='{}'][data-cursor='true'] > *",
                SelectPart::Chip.slot()
            ),
            focus_ring_sx(),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// `MultiSelect`'s chevron, in the frame's trailing slot, not on the last chip row.
/// The margin gives back the frame's wider gap.
static MULTI_CHEVRON_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .margin_left("-4px")
        .cursor("pointer")
        .selector("& > svg", sx().width("1em").height("1em").color("muted.6"))
        .when("disabled", sx().cursor("not-allowed"))
});

/// The search box above the rows. Not a field control, so it carries its own chrome.
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
        // The box's only boundary: 3:1, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .selector("::placeholder", sx().color("text-dimmed"))
        // Inset: the dropdown clips an outset ring (todo 1547).
        .focus_visible(inset_focus_ring_sx(&format!(
            "calc(-1 * {})",
            FOCUS_RING_WIDTH.value()
        )))
});

/// [`SEARCH_SX`]'s padding and bottom border, where a drawn placeholder sits.
const SEARCH_INSET: &str = "4px 8px 5px";

/// The keyboard's chip, and the id prefix `aria-activedescendant` points into.
#[derive(Clone, PartialEq)]
pub(crate) struct SelectionRenderArgs {
    pub cursor: Option<usize>,
    pub id_prefix: String,
}

/// A skin's drawing may compare equal; a caller's never does, as it can read unseen state.
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

/// The selection, in a signal: a pick on a closed select redraws the value, not the field.
#[derive(Clone, PartialEq, Default)]
pub(crate) struct Picked {
    /// One per option, whether or not `rows` is drawn.
    pub selected: Vec<bool>,
    /// One `Options::value()` per hidden input, as `<select multiple>` posts.
    pub form_values: Vec<String>,
}

/// Written in the skin's render only on a change, so an unchanged render wakes no reader.
pub(crate) fn use_picked(picked: Picked) -> Signal<Picked> {
    let mut signal = use_signal(|| picked.clone());
    if *signal.peek() != picked {
        signal.set(picked);
    }
    signal
}

field_parts_enum! {
    /// [`Select`](super::Select)'s and [`MultiSelect`](super::MultiSelect)'s
    /// inner parts, for their `parts` prop: a field's, the value and the chips.
    pub enum SelectPart framed {
        /// The picked value or the placeholder, in the trigger.
        Value = "value" => "& > * > [data-slot='frame'] > [data-slot='control'] > [data-slot='value'], & > * > [data-slot='frame'] > * > [data-slot='control'] > [data-slot='value']",
        /// A picked value's chip, `MultiSelect` only.
        Chip = "chip" => "& > * > [data-slot='frame'] > * > [data-slot='chip']",
    }
}

field_props! {
    parts(SelectPart);
    pub(crate) struct SelectCoreProps {
        /// Drawn only while open: an empty list compares equal, so a closed select skips a render.
        rows: Vec<Element>,
        /// Read only where drawn, so a pick on a closed select redraws just the value.
        picked: Signal<Picked>,
        state: ComboboxState,
        /// One group label per row, parallel to `rows`, filtered with them.
        #[props(default)]
        groups: Vec<Option<String>>,
        /// Parallel to `rows`. Drawn and read out; arrows, typeahead and click pass over it.
        #[props(default)]
        row_disabled: Vec<bool>,
        /// `Some(label)` while fetching: a loader replaces the rows; a status region says it.
        #[props(default)]
        loading: Option<String>,
        onpick: EventHandler<usize>,
        /// Drawn in the trigger (beside it when `multiple`) from the chip cursor. `None` shows `placeholder`.
        #[props(default)]
        selection: Option<SelectionDraw>,
        /// A `multiple` trigger's hidden value text; its live region announces changes.
        #[props(default)]
        value_labels: Vec<String>,
        /// How many chips `selection` draws, for the cursor over them.
        #[props(default)]
        chip_count: usize,
        /// An index into the selection order, not `rows`. Without it the chip keys do nothing.
        #[props(default)]
        onremove: Option<EventHandler<usize>>,
        #[props(default)]
        placeholder: Option<String>,
        /// Shows an x in place of the chevron while something is selected.
        #[props(default)]
        clearable: bool,
        onclear: EventHandler<()>,
        /// Stays open on a pick, sets `aria-multiselectable`, lets the selection wrap.
        #[props(default)]
        multiple: bool,
        #[props(default)]
        searchable: bool,
        #[props(default)]
        search_placeholder: Option<String>,
        /// One hidden input per `form_values` entry: the trigger is a `div` and can't post.
        #[props(default)]
        name: Option<String>,
        #[props(default)]
        rules: Option<crate::components::form::FieldStatus>,
        /// Plain text per row for typeahead; empty turns it off. Not `matches`: that
        /// is the caller's filter, typeahead is prefix-from-current-with-wrap.
        #[props(default)]
        row_labels: Vec<String>,
        /// Which rows survive the query, one `bool` per row; erases `T` like `rows`.
        #[props(default)]
        matches: Option<Callback<String, Vec<bool>>>,
        #[props(default, into)]
        dropdown_parts: Input<Parts<DropdownPart>>,
    }
}

/// The `T`-free engine under `Select` and `MultiSelect`: drawn rows in, indices out.
/// The trigger keeps focus (rows cancel `mousedown`), so its blur closes the list.
#[component]
pub(crate) fn SelectCore(props: SelectCoreProps) -> Element {
    let theme = use_theme();
    let nothing_found = use_localization().combobox.nothing_found;
    let size = props.size.copied_or(theme.select.size);
    let radius = props.radius.copied_or(theme.select.radius);
    let disabled = props.disabled.unwrap_or(false);
    // Read-only stays focusable and posting, but never opens; chip keys and clear go too.
    let readonly = props.readonly.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    let state = props.state;
    let opened = state.is_open() && !disabled && !readonly;
    let searchable = props.searchable && !disabled;

    // A native `<select>`'s buffer: "ber" finds Berlin, a lone "b" after a pause cycles.
    let typeahead = use_typeahead(TYPEAHEAD_RESET);
    // Never while `searchable`: the search box is the affordance.
    let typeahead_on = !searchable && !props.row_labels.is_empty();

    // Here, so the skins stay stateless.
    let query = use_signal(String::new);
    // The keyboard's chip, an index into the selection order, not `rows`.
    let cursor = use_signal(|| None::<usize>);
    let search = use_element();
    let trigger_element = use_element();
    let toolbar_item = use_toolbar_item();
    let row_cache = use_row_cache();

    let picked = props.picked;
    let visible = visible_rows(searchable, props.matches, query, picked);

    // After a removal the index points at the next chip, clamped to the last.
    let chip_count = props.chip_count;
    let chip_cursor = match chip_count {
        0 => None,
        count => cursor().map(|index| index.min(count - 1)),
    };

    // Subscribed only where a pick changes what is drawn.
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

    let blurred = use_refocus_on_close(opened, searchable, trigger_element, query);

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
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let onclear = props.onclear;
    let clear = clear_button(
        props.clearable && has_selection && !disabled && !readonly,
        size,
        trigger_element,
        Some(&field),
        move |_| onclear.call(()),
    );

    let typed = SelectTypeahead {
        // Shares the hook's `Rc`, so the keydown arms can ask whether a space is typed.
        buffer: typeahead.clone(),
        labels: props.row_labels.clone(),
        row_disabled: props.row_disabled.clone(),
        multiple: props.multiple,
        onpick: props.onpick,
        open,
        on: typeahead_on,
    };

    let multiple = props.multiple;

    // Hooks, so prepared either way.
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

    // A multi-select's chevron, replaced by the clear button. Neither takes focus from the trigger.
    let trailing = match (multiple, &clear) {
        (true, None) => Some(
            chevron_box
                .event("onmousedown", |event: MouseEvent| event.prevent_default())
                .event("onclick", move |_: MouseEvent| open.toggle())
                .render(HtmlTag::Span, Vec::new(), rsx! { Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined } }),
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
    // The frame draws the ring.
    let control = use_box()
        .framework_sx(&SELECT_TRIGGER_SX)
        .focus_ring(false)
        .states(&trigger_states)
        .prepare();

    let rows = select_rows(&props, &visible, state, &row_cache);
    // Written before the trigger reads it; from `ComboboxCore` it re-ran this scope per open (todo 842).
    if opened {
        state.set_rows(if props.loading.is_some() {
            0
        } else {
            rows.rows.len()
        });
    }
    let aria = state.a11y_attributes_keyed(&rows.keys);

    let search_box = use_box().framework_sx(&SEARCH_SX).prepare();
    // The caller's name follows the combobox role onto the open search box.
    let searching = searchable && opened;
    let (naming, attributes): (Vec<Attribute>, Vec<Attribute>) =
        props.attributes.iter().cloned().partition(|attribute| {
            searching && matches!(attribute.name, "aria-label" | "aria-labelledby")
        });
    let header = searchable.then(|| {
        let search_placeholder = props.search_placeholder.clone().unwrap_or_default();
        with_drawn_placeholder(
            Some(&search_placeholder),
            SEARCH_INSET,
            select_search_box(
                search_box.attr("placeholder", search_placeholder.clone()),
                SelectSearch {
                    element: search,
                    query,
                    blurred,
                    trigger: trigger_element,
                },
                state,
                aria.clone(),
                naming,
                &field,
                required,
            ),
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

    // The open search box is the combobox; a role-less trigger may not carry `aria-required`.
    let control = match searching {
        true => control
            .attr("data-slot", SelectPart::Control.slot())
            .attr("id", field.id().to_string()),
        false => field
            .aria(control)
            .attr("aria-labelledby", field.label_id()),
    };
    let trigger = select_trigger(
        control,
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
            chevron: !multiple && clear.is_none(),
            toolbar_item,
            aria,
        },
        content,
        attributes,
    );
    let control = select_control(multiple, value_slot, open, chips, trigger);

    let listbox = select_listbox(Listbox {
        rows,
        header,
        control: frame.render(control),
        open,
        opened,
        size,
        radius,
        loading: props.loading,
        nothing_found: (searchable && !query.read().is_empty()).then(|| nothing_found.to_string()),
        multiple,
        // A multi-select pick adds or drops a chip, resizing the trigger.
        remeasure: match opened {
            true => picked
                .read()
                .selected
                .iter()
                .filter(|selected| **selected)
                .count() as u64,
            false => 0,
        },
        // Focused once measured; earlier is a silent no-op.
        autofocus: searchable.then_some(search),
        labelled_by: field.label_id(),
        parts: props.dropdown_parts,
    });

    field.render(rsx! {
        {listbox}
        {select_hidden_inputs(props.name, picked, disabled)}
        if multiple {
            {announcer}
        }
    })
}

/// The dropdown, hung off the whole field frame.
struct Listbox {
    rows: SelectRows,
    header: Option<Element>,
    /// The frame, already rendered around the trigger.
    control: Element,
    open: SelectOpen,
    opened: bool,
    size: Size,
    radius: Size,
    loading: Option<String>,
    nothing_found: Option<String>,
    multiple: bool,
    remeasure: u64,
    /// The search box, if any.
    autofocus: Option<ElementHandle>,
    labelled_by: Option<String>,
    parts: Input<Parts<DropdownPart>>,
}

fn select_listbox(list: Listbox) -> Element {
    let Listbox {
        parts,
        rows:
            SelectRows {
                rows,
                keys,
                groups,
                row_disabled,
            },
        header,
        control,
        open,
        opened,
        size,
        radius,
        loading,
        nothing_found,
        multiple,
        remeasure,
        autofocus,
        labelled_by,
    } = list;
    let state = open.state;

    rsx! {
        ComboboxCore {
            rows,
            row_keys: keys,
            groups,
            row_disabled,
            loading,
            nothing_found,
            active: state.active(),
            onactive: move |row| state.set_active(Some(row)),
            opened,
            onopened: move |next| open.open(next),
            state,
            size,
            radius,
            disabled: open.disabled || open.readonly,
            close_on_pick: !multiple,
            // Only the search box has a caret; a bare trigger's Home/End are the rows'.
            caret_keys: match autofocus.is_some() {
                true => CaretKeys::Always,
                false => CaretKeys::Off,
            },
            commit_on_leave: !multiple,
            // A search box types its spaces.
            space_picks: autofocus.is_none(),
            multiselectable: multiple,
            header,
            autofocus,
            width: PopoverWidth::Min,
            remeasure,
            labelled_by,
            parts,
            {control}
        }
    }
}

/// Opening arms the selected row and drops the chip cursor: one `aria-activedescendant`, one owner.
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
    /// The first selected visible row, like a native `<select>`. Read at the press, not render.
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
            let mut cursor = self.cursor;
            cursor.set(None);
        }
        self.state.set_open(next);
    }

    /// For the chip slot and chevron: focuses the trigger and toggles. State is read first,
    /// since a searchable list closes on its box's blur.
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

/// The typeahead buffer (shared with the keydown arms) and what it searches.
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
    /// `true` when the character landed on a row, so the key was ours.
    fn type_to(&self, ch: char) -> bool {
        let state = self.open.state;
        let query = self.buffer.push(ch);
        // From the highlight while open, the selection while closed.
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
            // Open: moves the highlight only, as the arrows do.
            (true, _) => state.set_active(Some(row)),
            // Closed single select: picks in place, like a native `<select>`.
            (false, false) => self.onpick.call(row),
            // Closed multi-select opens instead: a pick toggles, so typing could drop a value.
            (false, true) => {
                self.open.open(true);
                state.set_active(Some(row));
            }
        }
        true
    }
}

#[derive(Clone, Copy)]
struct TriggerKeys {
    open: SelectOpen,
    chip_count: usize,
    chip_cursor: Option<usize>,
    cursor: Signal<Option<usize>>,
    onremove: Option<EventHandler<usize>>,
}

struct TriggerParts {
    element: ElementHandle,
    id_prefix: String,
    /// The list is open and usable.
    opened: bool,
    searchable: bool,
    multiple: bool,
    /// A single select with nothing to clear draws its own chevron.
    chevron: bool,
    /// Inside a `Toolbar`: a roving tab stop, focusable even disabled.
    toolbar_item: Option<ToolbarItem>,
    /// The state's combobox ARIA, by row key.
    aria: Vec<Attribute>,
}

/// The one element `ComboboxCore` doesn't draw; it holds focus throughout.
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
        toolbar_item,
        aria,
    } = parts;
    let open = keys.open;
    let state = open.state;
    let (disabled, readonly) = (open.disabled, open.readonly);

    // Only one element is the combobox: with the search box open, the trigger keeps `aria-haspopup`.
    let searching = searchable && opened;
    let mut attributes = match searching {
        true => vec![attr("aria-haspopup", "listbox")],
        false => aria,
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
        .attr(
            "tabindex",
            match toolbar_item {
                Some(item) => Some(item.tabindex()),
                None => (!disabled).then_some("0"),
            },
        )
        .attr(TOOLBAR_ITEM, toolbar_item.map(ToolbarItem::key))
        // Keeps the open search box focused, so the click reads the list open and closes it (todo 2294).
        .event(
            "onmousedown",
            searchable.then_some(move |event: MouseEvent| {
                if state.is_open() {
                    event.prevent_default();
                }
            }),
        )
        // A multi-select's click bubbles to the slot around it, which owns it.
        .event("onclick", move |_: MouseEvent| {
            if let Some(item) = toolbar_item {
                item.take_stop();
            }
            if !multiple && !disabled && !readonly {
                open.open(!state.is_open());
            }
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            trigger_key(&event, keys, &typed)
        })
        // While searchable, focus moves to the search box, whose blur closes instead.
        .event("onblur", move |event: FocusEvent| {
            if !searchable && blur_counts(&event) {
                state.close();
            }
        })
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {content}
                if chevron {
                    Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                }
            },
        )
}

/// The chips take Left, Right and Backspace; the list the rest. Silent while the search box has focus.
fn trigger_key(event: &KeyboardEvent, keys: TriggerKeys, typed: &SelectTypeahead) {
    let TriggerKeys {
        open,
        chip_count,
        chip_cursor,
        mut cursor,
        onremove,
    } = keys;
    let state = open.state;
    // A chord is the browser's, or Alt+ArrowDown/Up for `ComboboxCore` (APG).
    if open.disabled || open.readonly || navigation_chord(event).is_some() {
        return;
    }
    match logical_key(event) {
        Key::ArrowLeft if chip_count > 0 => {
            event.prevent_default();
            let index = match chip_cursor {
                Some(index) => index.saturating_sub(1),
                // From no cursor, the last chip, the one Backspace takes.
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
        // APG select-only: ArrowUp opens on the selection, Home/End on the first/last enabled row.
        key @ (Key::ArrowUp | Key::Home | Key::End) if !state.is_open() => {
            event.prevent_default();
            open.open(true);
            let last = open.picked.peek().selected.len().saturating_sub(1);
            match key {
                Key::Home => state.set_active(Some(0)),
                Key::End => state.set_active(Some(last)),
                _ => {}
            }
        }
        // Select-only also opens on Enter and Space; Enter on an open list is the core's pick.
        Key::Enter if !state.is_open() => {
            event.prevent_default();
            open.open(true);
        }
        // A browser or OS shortcut is never typeahead.
        Key::Character(_) if has_shortcut_modifier(event) => {}
        // A space mid-query is part of "new york": kept from `ComboboxCore`, whose Space picks.
        Key::Character(ref key) if key == " " && typed.on && typed.buffer.is_typing() => {
            event.prevent_default();
            event.stop_propagation();
            typed.type_to(' ');
        }
        Key::Character(ref key) if key == " " && !state.is_open() => {
            event.prevent_default();
            open.open(true);
        }
        Key::Character(ref key) if typed.on && key != " " => {
            // A key matching no row stays the page's, as in a native control.
            if let Some(ch) = key.chars().next()
                && typed.type_to(ch)
            {
                event.prevent_default();
            }
        }
        _ => {}
    }
}

/// Chips and trigger in one flow, as `TagsField`. The slot cancels `mousedown` and focuses the
/// trigger on click; the trigger, no longer the frame's child, needs its own ring overlay.
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

/// The full-list indices of the rows the query keeps, so the skins never remap.
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
        // `selected`, not `rows`, which a closed list leaves empty. Peeked: options redraw the core anyway.
        _ => (0..picked.peek().selected.len()).collect(),
    }
}

/// Everything a row draws from: a change to any of them redraws it.
#[derive(Clone, PartialEq)]
struct SelectRowInputs {
    label: Element,
    selected: bool,
    close_on_pick: bool,
    onpick: EventHandler<usize>,
    state: ComboboxState,
}

/// One kept row; `index` is the full-list index.
fn select_row(index: usize, inputs: &SelectRowInputs) -> Element {
    let SelectRowInputs {
        label,
        selected,
        close_on_pick,
        onpick,
        state,
    } = inputs.clone();
    rsx! {
        ComboboxOption {
            selected,
            onpick: move |_| {
                // A single select's re-pick emits nothing, as a native `<select>`.
                if !(close_on_pick && selected) {
                    onpick.call(index);
                }
                if close_on_pick {
                    state.close();
                }
            },
            {label}
        }
    }
}

/// The kept rows, keyed by their full-list index, and their parallel group labels and disabled flags.
#[derive(Default)]
struct SelectRows {
    rows: Vec<Element>,
    keys: Vec<usize>,
    groups: Vec<Option<String>>,
    row_disabled: Vec<bool>,
}

/// The kept rows as `ComboboxOption`s, with their group labels and disabled flags, filtered in parallel.
fn select_rows(
    props: &SelectCoreProps,
    visible: &[usize],
    state: ComboboxState,
    cache: &RowCache<SelectRowInputs>,
) -> SelectRows {
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
    let inputs: Vec<(usize, SelectRowInputs)> = visible
        .iter()
        .copied()
        .filter_map(|index| {
            let inputs = SelectRowInputs {
                label: props.rows.get(index)?.clone(),
                selected: picked.selected.get(index).copied().unwrap_or(false),
                close_on_pick,
                onpick,
                state,
            };
            Some((index, inputs))
        })
        .collect();
    let keys = inputs.iter().map(|(index, _)| *index).collect();

    SelectRows {
        rows: cache.rows(inputs, select_row),
        keys,
        groups,
        row_disabled,
    }
}

#[derive(Clone, Copy)]
struct SelectSearch {
    element: ElementHandle,
    query: Signal<String>,
    /// [`use_refocus_on_close`]'s blur mark.
    blurred: Signal<bool>,
    trigger: ElementHandle,
}

/// The open search box owns the combobox role and the trigger's a11y wiring, field label included.
/// `naming` is the caller's `aria-label`/`aria-labelledby`, moved off the trigger.
fn select_search_box(
    search_box: BoxStyle,
    search: SelectSearch,
    state: ComboboxState,
    // The state's combobox ARIA, by row key.
    mut attributes: Vec<Attribute>,
    naming: Vec<Attribute>,
    field: &PreparedField,
    required: bool,
) -> Element {
    let SelectSearch {
        element: search,
        mut query,
        mut blurred,
        trigger,
    } = search;
    attributes.extend(naming);
    search_box
        .element(&search)
        .attr_default("type", "text")
        .attr("data-slot", DropdownPart::Search.slot())
        .attr("value", query())
        .attr("data-controlled", true)
        // Ours is the list underneath; the browser's would cover it.
        .attr("autocomplete", "off")
        .attr("aria-autocomplete", "list")
        .attr("aria-labelledby", field.label_id())
        .attr("aria-describedby", field.describedby())
        .attr("aria-invalid", field.invalid().then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // A new list: arm its top row.
            state.set_active(Some(0));
        })
        // Portaled after the page: Tab moves on from the trigger; the dropdown's keys still commit (todo 2289).
        .event("onkeydown", move |event: KeyboardEvent| {
            if event.key() == Key::Tab {
                let _ = trigger.focus();
                // Blitz moves focus without a blur, and the close would refocus the trigger (todo 2354).
                blurred.set(true);
            }
        })
        // Closes while searchable; the rows cancel `mousedown`, so a click inside never blurs.
        .event("onblur", move |event: FocusEvent| {
            if blur_counts(&event) {
                blurred.set(true);
                state.close();
            }
        })
        .render(HtmlTag::Input, attributes, ())
}

/// The trigger's content, and a multi-select's chips beside it; the trigger then says the value as hidden text.
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
        span { "data-slot": SelectPart::Value.slot(), "data-placeholder": "true", "{placeholder}" }
    }
}

/// A single select's value in its own scope, so a pick redraws this alone.
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
            rsx! { span { "data-slot": SelectPart::Value.slot(), {drawn} } }
        }
        None => placeholder_value(&placeholder),
    }
}

/// Hidden inputs post the value, as on `Slider`; a disabled select sends nothing.
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
