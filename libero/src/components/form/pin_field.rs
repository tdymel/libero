use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{field_props, input_from_str, navigation_chord},
        form::{FIELD_CONTROL_SX, PreparedFrame, use_bound, use_field, use_field_frame},
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::{FIELD_HEIGHT, PinFieldDefaults},
};

pub use crate::theme::PinKind;

input_from_str!(PinKind);

/// A cell is a square of `FIELD_HEIGHT`, so a `PinField` is exactly as tall as
/// a `TextField` at the same size. The frame's own `padding_x` would make it a
/// rectangle, so the row overrides it - `& > div` outranks the frame's own
/// class.
static PIN_FIELD_ROW_SX: StaticSx = StaticSx::new(|| {
    PinFieldDefaults::theme_vars()
        .display("flex")
        .align_items("center")
        .per_size(|size| {
            sx().selector(
                "& > div",
                sx().width(FIELD_HEIGHT.value(size))
                    .flex("0 0 auto")
                    .padding_left("0")
                    .padding_right("0"),
            )
        })
        .selector("& input", sx().text_align("center"))
});

field_props! {
    pub struct PinFieldProps {
        /// The pin so far, one character per filled cell. `None` leaves the
        /// cells to the field's own buffer - it always keeps one, because
        /// auto-advance has to know which cell just filled.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per accepted character with the pin the field should hold
        /// next. Native name, native timing.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the pin, shown once the field loses focus or its form is
        /// submitted.
        #[props(default, into)]
        validate: crate::components::Validators<String>,
        /// Fires once when the last empty cell fills. Clearing a cell arms it
        /// again.
        #[props(default)]
        oncomplete: Option<EventHandler<String>>,
        /// How many cells.
        #[props(default, into)]
        length: Input<usize>,
        /// Which characters a cell accepts. Anything else is dropped at the
        /// key, so a rejected character never appears and is never removed
        /// again.
        #[props(default, into)]
        kind: Input<PinKind>,
        /// Renders the cells as password inputs. The value is unaffected.
        #[props(default)]
        mask: Option<bool>,
        /// `autocomplete="one-time-code"` on the first cell, so a phone offers
        /// the code it just received. On by default - an OTP is what a pin
        /// field is usually for.
        #[props(default)]
        one_time_code: Option<bool>,
        /// Rendered between the cells - a dash, a wider gap.
        #[props(default, into)]
        separator: Option<Element>,
        /// Emits a hidden input of that name, so the pin posts with a form.
        /// The cells cannot carry it themselves - there are several of them,
        /// and each holds one character.
        /// A path - `Login::FIELDS.code()` - also binds the pin to the
        /// surrounding `Form`'s value when there is no `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<String>,
        /// Focuses the first cell on mount.
        #[props(default)]
        autofocus: Option<bool>,
    }
}

/// A pin, one character per cell.
///
/// Typing fills a cell and moves to the next; Backspace clears and steps back;
/// the arrows, Home and End move without changing anything. Pasting a whole
/// code into any cell spreads it across the rest.
///
/// Controlled through `value` + `oninput`, and `oncomplete` fires the moment
/// the last cell fills. Holes are not representable: the value is the filled
/// cells joined, so typing into a cell past the end of the value lands in the
/// first empty one.
#[component]
pub fn PinField(props: PinFieldProps) -> Element {
    let theme = use_theme();
    let root = use_element();
    let buffer = use_signal(String::new);

    let size = props.size.copied_or(theme.pin_field.size);
    let radius = props.radius.copied_or(theme.pin_field.radius);
    let kind = props.kind.copied_or(theme.pin_field.kind);
    // A field with no cells has no reachable state, and every index below
    // would be out of bounds.
    let length = props
        .length
        .copied_or(theme.pin_field.length)
        .clamp(1, MAX_LENGTH);
    let readonly = props.readonly.unwrap_or(false);
    let required = props.required.unwrap_or(false);
    let mask = props.mask.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let value = bound
        .value()
        .or_else(|| props.value.clone())
        .unwrap_or_else(&*buffer);
    let cells = to_cells(&value, length);

    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&value))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(kind.state_name(), true)
        .into();

    let row = use_box()
        .framework_sx(&PIN_FIELD_ROW_SX)
        .states(&states)
        .prepare();

    // One prepared frame and one prepared control, rendered once per cell.
    // Neither carries an id, so a clone is a class, a `data-state`, the
    // attribute list and the frame's two listeners - no hook runs again.
    let frame = use_field_frame().states(&states).prepare();
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let oninput = bound.emit(props.oninput);
    let oncomplete = props.oncomplete;
    let controlled = props.value.is_some() || bound.is_bound();
    // The latch reads the rendered value, not a flag of our own: a parent
    // resetting `value` re-arms it just as the field's own edits do.
    let was_full = cells.iter().all(Option::is_some);
    // An `Rc` rather than a `use_callback`: every cell holds one, and it writes
    // signals this component reads from inside an input event
    // ([[codebase/reentrant-handlers]]).
    let report: Rc<dyn Fn(Vec<Option<char>>)> = Rc::new(move |cells: Vec<Option<char>>| {
        let mut buffer = buffer;
        let next: String = cells.iter().flatten().collect();
        let full = next.chars().count() == length;
        if !controlled {
            buffer.set(next.clone());
        }
        if let Some(oninput) = &oninput {
            oninput(next.clone());
        }
        // Latched: without it every keystroke on a full pin reports a
        // completion, and a submit handler would run again on each.
        if full
            && !was_full
            && let Some(oncomplete) = &oncomplete
        {
            oncomplete.call(next);
        }
    });

    // One handle for the field's life, refreshed each render: a cell whose
    // character did not change skips, and its handlers still see this render.
    let edit = PinEdit {
        cells: cells.clone(),
        length,
        kind,
        root,
        readonly,
        report,
    };
    let editor = use_hook(|| Editor(Rc::new(RefCell::new(edit.clone()))));
    *editor.0.borrow_mut() = edit;

    let one_time_code = props.one_time_code.unwrap_or(true);
    let autofocus = props.autofocus.unwrap_or(false);
    let input_type = match (mask, kind) {
        (true, _) => "password",
        // `tel`, not `number`: a numeric keypad without a spinner.
        (false, PinKind::Numeric) => "tel",
        (false, PinKind::Alphanumeric) => "text",
    };

    let mut children = pin_cells(
        Cells {
            cells,
            editor,
            control,
            frame: CellFrame { frame, states },
            id: field.id().to_string(),
            length,
            kind,
            input_type,
            disabled,
            readonly,
            required,
            invalid: field.invalid(),
            one_time_code,
            autofocus,
        },
        props.separator.as_ref(),
    );

    if let Some(name) = bound.name().map(str::to_string) {
        // `Some(true)` or nothing - a `false` bool reaches a native renderer
        // as the string "false", which reads as disabled.
        children.push(rsx! {
            input {
                r#type: "hidden",
                name,
                value: "{value}",
                disabled: disabled.then_some(true),
            }
        });
    }

    let row = row
        .attr("role", "group")
        // A pin reads left to right in an RTL locale too - the first cell is
        // the first character either way.
        .attr("dir", "ltr")
        .attr("aria-labelledby", field.label_id())
        .attr("aria-describedby", field.describedby())
        .attr("aria-invalid", field.invalid().then_some("true"))
        .element(&root)
        .render(HtmlTag::Div, props.attributes, children);

    field.render(row)
}

/// Past this a pin is not a pin, and the DOM cost is the caller's mistake
/// rather than a typo we rendered.
const MAX_LENGTH: usize = 32;

/// The value, one character per cell. A value shorter than the field leaves
/// the trailing cells empty; a longer one is cut.
fn to_cells(value: &str, length: usize) -> Vec<Option<char>> {
    let mut cells = vec![None; length];
    for (cell, character) in cells.iter_mut().zip(value.chars()) {
        *cell = Some(character);
    }
    cells
}

/// The last index focus may land on after a spread.
fn next_len_floor(length: usize) -> usize {
    length - 1
}

/// Scoped to this field's own root, so two pin fields on one page do not take
/// each other's cells. Out-of-range indices are how the ends stop moving -
/// `wrapping_sub` on cell zero lands far past the last cell.
fn focus_cell(root: &ElementHandle, index: usize, length: usize) {
    if index >= length {
        return;
    }
    let selector = format!("input[data-pin-index=\"{index}\"]");
    let _ = root.query_selector(&selector).and_then(|el| el.focus());
}

/// The pin's editing engine. Every path that can change a cell - a keystroke,
/// a paste, Backspace, Delete - goes through `report`, so the completion latch
/// cannot disagree between them.
#[derive(Clone)]
struct PinEdit {
    cells: Vec<Option<char>>,
    length: usize,
    kind: PinKind,
    root: ElementHandle,
    readonly: bool,
    report: Rc<dyn Fn(Vec<Option<char>>)>,
}

impl PinEdit {
    /// One cell set or cleared.
    fn edit(&self, index: usize, character: Option<char>) {
        let mut next = self.cells.clone();
        next[index] = character;
        (self.report)(next);
    }

    /// A paste, from `index` onwards. Returns the cell the caret lands on.
    fn spread(&self, index: usize, characters: Vec<char>) -> usize {
        let mut next = self.cells.clone();
        let mut cursor = index;
        for character in characters {
            if cursor >= next.len() {
                break;
            }
            next[cursor] = Some(character);
            cursor += 1;
        }
        (self.report)(next);
        cursor.min(next_len_floor(self.length))
    }

    /// Where a character for cell `index` lands: that cell when it is filled,
    /// else the first empty one, since the pin has no holes.
    fn landing(&self, index: usize) -> usize {
        match self.cells[index] {
            Some(_) => index,
            None => self.cells.iter().flatten().count(),
        }
    }

    /// What arrived in one cell's `oninput`: a paste, or a character a soft
    /// keyboard typed without naming its key.
    fn typed(&self, index: usize, raw: String) {
        let (root, length) = (&self.root, self.length);
        // Back to what the cell last rendered: dioxus writes it again only if
        // the edit changes it, so an unchanged cell would keep the raw text.
        let shown = self.cells[index].map(String::from).unwrap_or_default();
        let _ = root
            .query_selector(&format!("input[data-pin-index=\"{index}\"]"))
            .and_then(|cell| cell.set_value(&shown));
        let mut accepted: Vec<char> = raw.chars().filter(|c| self.kind.accepts(*c)).collect();
        // The cell's old character sits on whichever side the caret was not.
        if accepted.len() > 1
            && let Some(old) = self.cells[index]
        {
            if accepted[0] == old {
                accepted.remove(0);
            } else if accepted.last() == Some(&old) {
                accepted.pop();
            }
        }
        let at = self.landing(index);
        match accepted.len() {
            // The cell was emptied - Backspace handles its own focus.
            0 if raw.is_empty() => self.edit(index, None),
            // Every character was rejected. The key handler drops those
            // before they land, so this is a paste of junk.
            0 => {}
            1 => {
                self.edit(at, Some(accepted[0]));
                focus_cell(root, at + 1, length);
            }
            _ => {
                let cursor = self.spread(at, accepted);
                focus_cell(root, cursor, length);
            }
        }
    }

    /// One cell's keyboard: the moves, the two deletions, and the characters
    /// that are dropped before they reach the DOM.
    fn keys(&self, index: usize, event: Event<KeyboardData>) {
        let (root, length, readonly) = (&self.root, self.length, self.readonly);
        let modified = event.modifiers().ctrl() || event.modifiers().meta();
        // Ctrl/Alt/Meta+arrow, Home or End is the caret's or the browser's.
        if navigation_chord(&event).is_some() {
            return;
        }
        match event.key() {
            Key::ArrowLeft => {
                event.prevent_default();
                focus_cell(root, index.wrapping_sub(1), length);
            }
            Key::ArrowRight => {
                event.prevent_default();
                focus_cell(root, index + 1, length);
            }
            Key::Home => {
                event.prevent_default();
                focus_cell(root, 0, length);
            }
            Key::End => {
                event.prevent_default();
                focus_cell(root, length - 1, length);
            }
            // Read-only cells still take every key that only moves:
            // the native `readonly` stops typing, but a handler that
            // clears a cell itself is not typing and it does not stop.
            Key::Delete => {
                event.prevent_default();
                if !readonly {
                    self.edit(index, None);
                }
            }
            Key::Backspace => {
                event.prevent_default();
                match self.cells[index].is_some() && !readonly {
                    true => {
                        self.edit(index, None);
                        // The last cell keeps focus: it is where the next
                        // character goes, and the pin is one short.
                        if index + 1 < length {
                            focus_cell(root, index.wrapping_sub(1), length);
                        }
                    }
                    false => focus_cell(root, index.wrapping_sub(1), length),
                }
            }
            Key::Character(character) if !modified => {
                let character = character.chars().next();
                match character {
                    // Space moves on rather than typing a character no
                    // pin accepts.
                    Some(' ') => {
                        event.prevent_default();
                        focus_cell(root, index + 1, length);
                    }
                    // Retyping what the cell already holds reads as
                    // confirming it, so move on instead of rewriting it.
                    Some(character) if self.cells[index] == Some(character) => {
                        event.prevent_default();
                        focus_cell(root, index + 1, length);
                    }
                    // Dropped at the key, so a rejected character never
                    // reaches the DOM - there is nothing to take back out
                    // of an input the value prop did not change.
                    Some(character) if !self.kind.accepts(character) => event.prevent_default(),
                    // Written here rather than by the browser, which would
                    // put it beside the old one on whichever side the caret is.
                    Some(character) => {
                        event.prevent_default();
                        if !readonly {
                            let at = self.landing(index);
                            self.edit(at, Some(character));
                            focus_cell(root, at + 1, length);
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }
}

/// The field's current [`PinEdit`]. Always equal: the field refreshes it in
/// place, so it never needs to redraw a cell.
#[derive(Clone)]
struct Editor(Rc<RefCell<PinEdit>>);

impl PartialEq for Editor {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Editor {
    /// Cloned out, so `report` runs with the cell released.
    fn current(&self) -> PinEdit {
        self.0.borrow().clone()
    }
}

/// The frame a cell sits in, equal while its `states` are: that is all its
/// look reads, and its listeners hold only this field's own press state.
#[derive(Clone)]
struct CellFrame {
    frame: PreparedFrame,
    states: Input<States>,
}

impl PartialEq for CellFrame {
    fn eq(&self, other: &Self) -> bool {
        self.states == other.states
    }
}

/// What one cell is drawn from. `control` and `frame` are prepared once and
/// cloned per cell, so no hook runs again.
struct Cells {
    cells: Vec<Option<char>>,
    editor: Editor,
    control: BoxStyle,
    frame: CellFrame,
    id: String,
    length: usize,
    kind: PinKind,
    input_type: &'static str,
    disabled: bool,
    readonly: bool,
    required: bool,
    invalid: bool,
    one_time_code: bool,
    autofocus: bool,
}

/// The cells, with the caller's separator between them.
fn pin_cells(parts: Cells, separator: Option<&Element>) -> Vec<Element> {
    let Cells {
        cells,
        editor,
        control,
        frame,
        id,
        length,
        kind,
        input_type,
        disabled,
        readonly,
        required,
        invalid,
        one_time_code,
        autofocus,
    } = parts;

    let mut children: Vec<Element> = Vec::with_capacity(length * 2);
    for (index, cell) in cells.iter().enumerate() {
        if index > 0
            && let Some(separator) = separator
        {
            children.push(separator.clone());
        }

        children.push(rsx! {
            PinCell {
                index,
                cell: *cell,
                editor: editor.clone(),
                control: control.clone(),
                frame: frame.clone(),
                id: id.clone(),
                kind,
                input_type,
                disabled,
                readonly,
                required,
                invalid,
                autocomplete: match one_time_code && index == 0 {
                    true => "one-time-code",
                    false => "off",
                },
                autofocus: autofocus && index == 0,
            }
        });
    }
    children
}

/// One cell, in its own scope: a keystroke redraws the cell it changed.
#[allow(clippy::too_many_arguments)]
#[component]
fn PinCell(
    index: usize,
    cell: Option<char>,
    editor: Editor,
    control: BoxStyle,
    frame: CellFrame,
    id: String,
    kind: PinKind,
    input_type: &'static str,
    disabled: bool,
    readonly: bool,
    required: bool,
    invalid: bool,
    autocomplete: &'static str,
    autofocus: bool,
) -> Element {
    let typed = editor.clone();
    let keys = editor;
    let input = control
        .attr("id", format!("{id}-{}", index + 1))
        .attr("type", input_type)
        .attr("inputmode", (kind == PinKind::Numeric).then_some("numeric"))
        .attr("autocomplete", autocomplete)
        .attr("value", cell.map(String::from).unwrap_or_default())
        .attr("data-controlled", true)
        .attr("data-pin-index", index.to_string())
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        // `aria-required` is not allowed on the group, so each cell says it.
        .attr("aria-required", required.then_some("true"))
        .attr("aria-invalid", invalid.then_some("true"))
        .attr("autofocus", autofocus)
        .event("oninput", move |event: FormEvent| {
            typed.current().typed(index, event.value())
        })
        .event("onkeydown", move |event: Event<KeyboardData>| {
            keys.current().keys(index, event)
        })
        .render(HtmlTag::Input, Vec::new(), ());

    frame.frame.render(input)
}
