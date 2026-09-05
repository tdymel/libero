use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{field_props, input_from_str},
        form::{FIELD_CONTROL_SX, use_bound, use_field, use_field_frame},
        layout::use_box,
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
    let mut buffer = use_signal(String::new);
    let mut completed = use_signal(|| false);

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
    // Neither carries an id or an event, so a clone is a class, a `data-state`
    // and the attribute list - no hook runs again.
    let frame = use_field_frame().states(&states).prepare();
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let oninput = bound.emit(props.oninput);
    let oncomplete = props.oncomplete;
    let controlled = props.value.is_some() || bound.is_bound();
    let report = move |cells: Vec<Option<char>>| {
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
        match full {
            true if !completed() => {
                completed.set(true);
                if let Some(oncomplete) = &oncomplete {
                    oncomplete.call(next);
                }
            }
            false => completed.set(false),
            true => {}
        }
    };

    let edit = {
        let cells = cells.clone();
        let mut report = report.clone();
        move |index: usize, character: Option<char>| {
            let mut next = cells.clone();
            next[index] = character;
            report(next);
        }
    };

    let spread = {
        let cells = cells.clone();
        let mut report = report.clone();
        move |index: usize, characters: Vec<char>| {
            let mut next = cells.clone();
            let mut cursor = index;
            for character in characters {
                if cursor >= next.len() {
                    break;
                }
                next[cursor] = Some(character);
                cursor += 1;
            }
            report(next);
            cursor.min(next_len_floor(length))
        }
    };

    let typed = {
        let cells = cells.clone();
        let mut edit = edit.clone();
        let mut spread = spread.clone();
        move |index: usize, raw: String| {
            let accepted: Vec<char> = raw.chars().filter(|c| kind.accepts(*c)).collect();
            match accepted.len() {
                // The cell was emptied - Backspace handles its own focus.
                0 if raw.is_empty() => edit(index, None),
                // Every character was rejected. The key handler drops those
                // before they land, so this is a paste of junk.
                0 => {}
                1 => {
                    edit(index, Some(accepted[0]));
                    focus_cell(&root, index + 1, length);
                }
                // Two characters in a cell that already held one is a
                // replacement, not a paste: the caret sat beside the old
                // character and the new one joined it.
                2 if cells[index].is_some() => {
                    edit(index, Some(accepted[1]));
                    focus_cell(&root, index + 1, length);
                }
                _ => {
                    let cursor = spread(index, accepted);
                    focus_cell(&root, cursor, length);
                }
            }
        }
    };

    let keys = {
        let cells = cells.clone();
        let mut edit = edit.clone();
        move |index: usize, event: Event<KeyboardData>| {
            let modified = event.modifiers().ctrl() || event.modifiers().meta();
            match event.key() {
                Key::ArrowLeft => {
                    event.prevent_default();
                    focus_cell(&root, index.wrapping_sub(1), length);
                }
                Key::ArrowRight => {
                    event.prevent_default();
                    focus_cell(&root, index + 1, length);
                }
                Key::Home => {
                    event.prevent_default();
                    focus_cell(&root, 0, length);
                }
                Key::End => {
                    event.prevent_default();
                    focus_cell(&root, length - 1, length);
                }
                // Read-only cells still take every key that only moves:
                // the native `readonly` stops typing, but a handler that
                // clears a cell itself is not typing and it does not stop.
                Key::Delete => {
                    event.prevent_default();
                    if !readonly {
                        edit(index, None);
                    }
                }
                Key::Backspace => {
                    event.prevent_default();
                    match cells[index].is_some() && !readonly {
                        true => {
                            edit(index, None);
                            // The last cell keeps focus: it is where the next
                            // character goes, and the pin is one short.
                            if index + 1 < length {
                                focus_cell(&root, index.wrapping_sub(1), length);
                            }
                        }
                        false => focus_cell(&root, index.wrapping_sub(1), length),
                    }
                }
                Key::Character(character) if !modified => {
                    let character = character.chars().next();
                    match character {
                        // Space moves on rather than typing a character no
                        // pin accepts.
                        Some(' ') => {
                            event.prevent_default();
                            focus_cell(&root, index + 1, length);
                        }
                        // Retyping what the cell already holds reads as
                        // confirming it, so move on instead of rewriting it.
                        Some(character) if cells[index] == Some(character) => {
                            event.prevent_default();
                            focus_cell(&root, index + 1, length);
                        }
                        // Dropped at the key, so a rejected character never
                        // reaches the DOM - there is nothing to take back out
                        // of an input the value prop did not change.
                        Some(character) if !kind.accepts(character) => event.prevent_default(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    };

    let one_time_code = props.one_time_code.unwrap_or(true);
    let autofocus = props.autofocus.unwrap_or(false);
    let input_type = match (mask, kind) {
        (true, _) => "password",
        // `tel`, not `number`: a numeric keypad without a spinner.
        (false, PinKind::Numeric) => "tel",
        (false, PinKind::Alphanumeric) => "text",
    };

    let id = field.id().to_string();
    let mut children: Vec<Element> = Vec::with_capacity(length * 2);
    for (index, cell) in cells.iter().enumerate() {
        if index > 0
            && let Some(separator) = &props.separator
        {
            children.push(separator.clone());
        }

        let mut typed = typed.clone();
        let mut keys = keys.clone();
        let input = control
            .clone()
            .attr("id", format!("{id}-{}", index + 1))
            .attr("type", input_type)
            .attr("inputmode", (kind == PinKind::Numeric).then_some("numeric"))
            .attr(
                "autocomplete",
                match one_time_code && index == 0 {
                    true => "one-time-code",
                    false => "off",
                },
            )
            .attr("value", cell.map(String::from).unwrap_or_default())
            .attr("data-controlled", true)
            .attr("data-pin-index", index.to_string())
            .attr("disabled", disabled)
            .attr("readonly", readonly)
            .attr("autofocus", autofocus && index == 0)
            .event("oninput", move |event: FormEvent| {
                typed(index, event.value())
            })
            .event("onkeydown", move |event: Event<KeyboardData>| {
                keys(index, event)
            })
            .render(HtmlTag::Input, Vec::new(), ());

        children.push(frame.clone().render(input));
    }

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
        .attr("aria-required", required.then_some("true"))
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
