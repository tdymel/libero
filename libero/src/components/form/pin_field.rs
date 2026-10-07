use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, States, input_from_str, names_itself, navigation_chord,
            use_name_warning,
        },
        form::{
            FIELD_CONTROL_SX, FieldPart, PreparedFrame, field_props, use_bound, use_field,
            use_field_frame,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, use_element, use_localization, use_theme},
    localization::fill,
    platform::{ElementApi, PlatformError, set_value_by_id},
    sx::{StaticSx, sx},
    theme::{FIELD_HEIGHT, PinFieldDefaults},
};

pub use crate::theme::PinKind;

input_from_str!(PinKind);

/// A cell is a square of `FIELD_HEIGHT`, as tall as a `TextField`, and narrows
/// on a small screen rather than overflow it (todo 1597). `& > div` outranks
/// the frame's `padding_x`, which would make it a rectangle.
static PIN_FIELD_ROW_SX: StaticSx = StaticSx::new(|| {
    PinFieldDefaults::theme_vars()
        .display("flex")
        .align_items("center")
        .per_size(|size| {
            sx().selector(
                "& > div",
                // A basis, not a `width`: a centred field's fit-content width
                // would otherwise never drop below the cells' sum.
                sx().width("auto")
                    .flex(format!("0 1 {}", FIELD_HEIGHT.value(size)))
                    .min_width("0")
                    .padding_left("0")
                    .padding_right("0"),
            )
        })
        // A percentage drops the input's own ~150px from the cell's minimum.
        .selector("& input", sx().text_align("center").width("100%"))
});

field_props! {
    pub struct PinFieldProps {
        /// The pin so far. `None` leaves the cells to the field's own buffer.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per accepted character with the pin the field should hold next.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the pin, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<String>,
        /// Fires once when the last empty cell fills; clearing a cell re-arms it.
        #[props(default)]
        oncomplete: Option<EventHandler<String>>,
        /// How many cells.
        #[props(default, into)]
        length: Input<usize>,
        /// Which characters a cell accepts; others are dropped at the key.
        #[props(default, into)]
        kind: Input<PinKind>,
        /// Renders the cells as password inputs. The value is unaffected.
        #[props(default)]
        mask: Option<bool>,
        /// `autocomplete="one-time-code"` on the first cell. On by default.
        #[props(default)]
        one_time_code: Option<bool>,
        /// Rendered between the cells, hidden from screen readers.
        #[props(default, into)]
        separator: Option<Element>,
        /// Posts the pin through a hidden input. A path also binds it to the
        /// surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<String>,
        /// Focuses the first cell on mount.
        #[props(default)]
        autofocus: Option<bool>,
    }
}

/// A pin or one-time code, one character per cell.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::PinField;
/// # fn app() -> Element {
/// let mut code = use_signal(String::new);
/// rsx! {
///     PinField {
///         label: "Verification code",
///         length: 6usize,
///         value: code(),
///         oninput: move |next| code.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/pin-field>
#[component]
pub fn PinField(props: PinFieldProps) -> Element {
    let theme = use_theme();
    let cell_label = use_localization().pin_field.cell;
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
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();
    use_name_warning(
        field.label_id().is_some() || names_itself(&props.attributes),
        "PinField: no `label`, `aria-label` or `aria-labelledby`, so its cells are read as \
         \"Character 1 of 6\" with no question.",
    );

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

    // One prepared frame and control, cloned per cell: neither carries an id,
    // so no hook runs again.
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
    // An `Rc`, not a `use_callback`: it writes signals read inside an input
    // event ([[codebase/reentrant-handlers]]).
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
    let handles: CellHandles = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    handles.borrow_mut().resize(length, None);
    let edit = PinEdit {
        cells: cells.clone(),
        length,
        kind,
        root,
        handles,
        id: field.id().to_string(),
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
            cell_label,
            describedby: field.describedby(),
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

/// Each cell's own handle, by index, filled in as the cells render.
type CellHandles = Rc<RefCell<Vec<Option<ElementHandle>>>>;

/// Cell `index`, scoped to this field's root. A WebView cannot query below the
/// root, so it falls back to the cell's own handle.
fn cell_at(
    root: &ElementHandle,
    handles: &CellHandles,
    index: usize,
) -> Result<Box<dyn ElementApi>, PlatformError> {
    root.query_selector(&format!("input[data-pin-index=\"{index}\"]"))
        .or_else(
            |error| match handles.borrow().get(index).copied().flatten() {
                Some(handle) => Ok(Box::new(handle) as Box<dyn ElementApi>),
                None => Err(error),
            },
        )
}

/// The pin's editing engine. Every change goes through `report`, so the
/// completion latch cannot disagree between paths.
#[derive(Clone)]
struct PinEdit {
    cells: Vec<Option<char>>,
    length: usize,
    kind: PinKind,
    root: ElementHandle,
    handles: CellHandles,
    /// The field's id; cell `n` is `{id}-{n + 1}`.
    id: String,
    readonly: bool,
    report: Rc<dyn Fn(Vec<Option<char>>)>,
}

impl PinEdit {
    /// An out-of-range index stops at the ends: `wrapping_sub` on cell zero
    /// lands far past the last cell.
    fn focus(&self, index: usize) {
        if index >= self.length {
            return;
        }
        let _ = cell_at(&self.root, &self.handles, index).and_then(|el| el.focus());
    }

    /// One cell set or cleared. Returns the cells it reported.
    fn edit(&self, index: usize, character: Option<char>) -> Vec<Option<char>> {
        let mut next = self.cells.clone();
        next[index] = character;
        (self.report)(next.clone());
        next
    }

    /// A paste, from `index` onwards. Returns the cells it reported and the
    /// cell the caret lands on.
    fn spread(&self, index: usize, characters: Vec<char>) -> (Vec<Option<char>>, usize) {
        let mut next = self.cells.clone();
        let mut cursor = index;
        for character in characters {
            if cursor >= next.len() {
                break;
            }
            next[cursor] = Some(character);
            cursor += 1;
        }
        (self.report)(next.clone());
        (next, cursor.min(next_len_floor(self.length)))
    }

    /// Writes cell `index`'s text. A WebView handle cannot, so it goes by id there.
    fn show(&self, index: usize, character: Option<char>) {
        let text = character.map(String::from).unwrap_or_default();
        let _ = cell_at(&self.root, &self.handles, index)
            .and_then(|cell| cell.set_value(&text))
            .or_else(|_| set_value_by_id(&format!("{}-{}", self.id, index + 1), &text));
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
        let mut raw: Vec<char> = raw.chars().collect();
        let emptied = raw.is_empty();
        // The cell's old character sits on whichever side the caret was not.
        // Out before the filter, so a rejected insertion does not keep it as new.
        if raw.len() > 1
            && let Some(old) = self.cells[index]
        {
            if raw[0] == old {
                raw.remove(0);
            } else if raw.last() == Some(&old) {
                raw.pop();
            }
        }
        let accepted: Vec<char> = raw.into_iter().filter(|c| self.kind.accepts(*c)).collect();
        let next = match accepted.len() {
            // The cell was emptied - Backspace handles its own focus.
            0 if emptied && self.cells[index].is_some() => self.edit(index, None),
            // Every character was rejected, or an empty cell stayed empty.
            0 => self.cells.clone(),
            _ => self.place(index, accepted),
        };
        // Dioxus writes the cell only if the edit changes it, so an unchanged
        // cell would keep the raw text. Its final text, as a WebView's write lands late.
        self.show(index, next[index]);
    }

    /// A paste, read from the clipboard rather than the cell: over a selected
    /// character the cell's text cannot tell what was old (todo 2440).
    fn pasted(&self, index: usize, event: ClipboardEvent) {
        // No text on this platform: the browser inserts it and `typed` runs.
        let Some(text) = event.data().data_transfer().get_as_text() else {
            return;
        };
        event.prevent_default();
        let accepted: Vec<char> = text.chars().filter(|c| self.kind.accepts(*c)).collect();
        if !self.readonly && !accepted.is_empty() {
            self.place(index, accepted);
        }
    }

    /// Accepted characters for cell `index`, written from where they land,
    /// with focus after the last.
    fn place(&self, index: usize, accepted: Vec<char>) -> Vec<Option<char>> {
        let at = self.landing(index);
        match accepted.as_slice() {
            [character] => {
                let next = self.edit(at, Some(*character));
                self.focus(at + 1);
                next
            }
            _ => {
                let (next, cursor) = self.spread(at, accepted);
                self.focus(cursor);
                next
            }
        }
    }

    /// One cell's keyboard: the moves, the two deletions, and the characters
    /// that are dropped before they reach the DOM.
    fn keys(&self, index: usize, event: Event<KeyboardData>) {
        let (length, readonly) = (self.length, self.readonly);
        let modified = event.modifiers().ctrl() || event.modifiers().meta();
        // Ctrl/Alt/Meta+arrow, Home or End is the caret's or the browser's.
        if navigation_chord(&event).is_some() {
            return;
        }
        match event.key() {
            Key::ArrowLeft => {
                event.prevent_default();
                self.focus(index.wrapping_sub(1));
            }
            Key::ArrowRight => {
                event.prevent_default();
                self.focus(index + 1);
            }
            Key::Home => {
                event.prevent_default();
                self.focus(0);
            }
            Key::End => {
                event.prevent_default();
                self.focus(length - 1);
            }
            // Native `readonly` does not stop a handler clearing a cell.
            Key::Delete => {
                event.prevent_default();
                // An empty cell reports nothing: `oninput` would see an unchanged pin.
                if self.cells[index].is_some() && !readonly {
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
                            self.focus(index.wrapping_sub(1));
                        }
                    }
                    false => self.focus(index.wrapping_sub(1)),
                }
            }
            Key::Character(character) if !modified => {
                let character = character.chars().next();
                match character {
                    // Space moves on rather than typing a character no
                    // pin accepts.
                    Some(' ') => {
                        event.prevent_default();
                        self.focus(index + 1);
                    }
                    // Retyping what the cell already holds reads as
                    // confirming it, so move on instead of rewriting it.
                    Some(character) if self.cells[index] == Some(character) => {
                        event.prevent_default();
                        self.focus(index + 1);
                    }
                    // Dropped at the key: an unchanged value prop could not take it back out.
                    Some(character) if !self.kind.accepts(character) => event.prevent_default(),
                    // Written here rather than by the browser, which would
                    // put it beside the old one on whichever side the caret is.
                    Some(character) => {
                        event.prevent_default();
                        if !readonly {
                            let at = self.landing(index);
                            self.edit(at, Some(character));
                            self.focus(at + 1);
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
    cell_label: &'static str,
    describedby: Option<String>,
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
        cell_label,
        describedby,
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
            // Decoration: a reader would say "dash" between every two cells.
            children.push(rsx! {
                span { aria_hidden: "true", {separator.clone()} }
            });
        }

        children.push(rsx! {
            PinCell {
                index,
                cell: *cell,
                editor: editor.clone(),
                control: control.clone(),
                frame: frame.clone(),
                id: id.clone(),
                label: fill(cell_label, &[("n", &(index + 1)), ("m", &length)]),
                describedby: describedby.clone(),
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
    label: String,
    describedby: Option<String>,
    kind: PinKind,
    input_type: &'static str,
    disabled: bool,
    readonly: bool,
    required: bool,
    invalid: bool,
    autocomplete: &'static str,
    autofocus: bool,
) -> Element {
    let handle = use_element();
    if let Some(slot) = editor.0.borrow().handles.borrow_mut().get_mut(index) {
        *slot = Some(handle);
    }
    let typed = editor.clone();
    let pasted = editor.clone();
    let keys = editor;
    let input = control
        .element(&handle)
        .attr("data-slot", FieldPart::Control.slot())
        .attr("id", format!("{id}-{}", index + 1))
        .attr("type", input_type)
        .attr("inputmode", (kind == PinKind::Numeric).then_some("numeric"))
        .attr("autocomplete", autocomplete)
        // A code is not a word: no capital first letter, no correction.
        .attr("autocapitalize", "off")
        .attr("autocorrect", "off")
        .attr("spellcheck", "false")
        .attr("value", cell.map(String::from).unwrap_or_default())
        .attr("data-controlled", true)
        .attr("data-pin-index", index.to_string())
        // The group names the pin; each cell names its place in it, and the
        // error or helper reaches the focused cell too (todos 507, 591).
        .attr("aria-label", label)
        .attr("aria-describedby", describedby)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        // `aria-required` is not allowed on the group, so each cell says it.
        .attr("aria-required", required.then_some("true"))
        .attr("aria-invalid", invalid.then_some("true"))
        .attr("autofocus", autofocus)
        .event("oninput", move |event: FormEvent| {
            typed.current().typed(index, event.value())
        })
        .event("onpaste", move |event: ClipboardEvent| {
            pasted.current().pasted(index, event)
        })
        .event("onkeydown", move |event: Event<KeyboardData>| {
            keys.current().keys(index, event)
        })
        .render(HtmlTag::Input, Vec::new(), ());

    frame.frame.render(input)
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use crate::{LiberoProvider, components::form::PinField, utils::warnings_of};

    fn warns(app: fn() -> Element) -> bool {
        warnings_of(app)
            .iter()
            .any(|warning| warning.starts_with("PinField:"))
    }

    /// Todo 2442.
    #[test]
    fn an_unnamed_pin_field_warns() {
        assert!(warns(|| rsx! { LiberoProvider { PinField {} } }));
        assert!(!warns(
            || rsx! { LiberoProvider { PinField { label: "Code" } } }
        ));
        assert!(!warns(
            || rsx! { LiberoProvider { PinField { aria_label: "Code" } } }
        ));
    }
}
