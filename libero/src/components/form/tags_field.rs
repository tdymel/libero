use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        CaretKeys, ComboboxCore, ComboboxOption, ComboboxState, HtmlTag, Input, SelectionArgs,
        common::{field_props, ring_overlay},
        form::{
            clear_button, field_control_sx, removable_chip, row_label, use_announcer, use_bound,
            use_chip_announcer, use_field, use_field_frame,
        },
        layout::{BoxStyle, use_box},
        use_combobox,
    },
    hooks::{ElementHandle, PopoverWidth, use_element, use_localization, use_theme},
    localization::{TagsFieldLabels, fill},
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::Size,
    utils::warn,
};

/// The frame's control: the chips and the input on one wrapping flow, so a tag
/// editor types *between* its chips rather than in a box beside them.
///
/// It is not the `<input>` itself, because the input is only the draft. The
/// field's id, its label and its `aria-describedby` still land on the input -
/// that is what the caller types into and what `<label for>` may name.
static TAGS_VALUE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .align_items("center")
        .gap("4px")
        .flex("1 1 auto")
        // Without it a long tag pushes the frame wider instead of wrapping.
        .min_width("0")
        // `min-width: 0`, or a long tag's chip is floored at its whole label
        // before the chip's own ellipsis can apply.
        .selector(
            "& > [data-slot='tag']",
            sx().display("inline-flex").max_width("100%").min_width("0"),
        )
        // A flex line of its own, or the x hangs off the label's baseline.
        .selector(
            "& [data-slot='remove']",
            sx().display("inline-flex").align_items("center"),
        )
});

/// The draft input, sharing the value slot's lines with the chips.
///
/// `flex-basis` rather than `min-width`: the basis is what a wrapping flex
/// container measures before it shrinks anything, so the input drops onto its
/// own line once the chips have taken the one it was on, instead of being
/// squeezed to nothing beside them.
static TAGS_INPUT_SX: StaticSx = StaticSx::new(|| field_control_sx().flex("1 1 60px"));

field_props! {
    extends(input);
    pub struct TagsFieldProps {
        /// The tags, in order. Strictly controlled - pair it with `onchange`.
        #[props(default)]
        value: Vec<String>,
        /// Called with the whole list the caller should hold next.
        #[props(default)]
        onchange: Option<EventHandler<Vec<String>>>,
        /// Offers a dropdown of tags to pick instead of typing. Absent means no
        /// dropdown at all - no listbox, no portal, and the input is a plain
        /// text input rather than a combobox. Tags already held are never
        /// offered.
        #[props(default)]
        suggestions: Option<Vec<String>>,
        /// Each one commits the text before it. Defaults to `[","]`, and
        /// applies to a paste as much as to typing - a paste arrives as one
        /// `oninput` with the whole resulting string, so one splitter covers
        /// both and nothing reads the clipboard.
        #[props(default)]
        split_chars: Option<Vec<String>>,
        /// Lets the same tag be added twice. Off, the comparison is trimmed and
        /// case-insensitive.
        #[props(default)]
        allow_duplicates: Option<bool>,
        /// The most tags the field accepts. Everything past it is refused, one
        /// tag at a time - a paste of five into a field with room for two adds
        /// those two and refuses the rest, rather than refusing the paste.
        #[props(default)]
        max_tags: Option<usize>,
        /// Accepts or refuses **one** tag, before it is added. `validate` is
        /// the ordinary field line and rules over the whole list; this one
        /// never shows a message, because "you already added that" is heavier
        /// as an error under the field than the mistake it describes.
        #[props(default)]
        tag_rules: Option<Callback<String, bool>>,
        /// A tag was refused - a duplicate, one past `max_tags`, or one
        /// `tag_rules` turned down. The only thing `onchange` cannot report,
        /// which is why it is the only callback here beside it.
        #[props(default)]
        onrefuse: Option<EventHandler<String>>,
        /// Rules over the whole list, shown once the field loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Vec<String>>,
        /// Emits one hidden input of that name per tag. The visible input holds
        /// the draft, not the value, so it cannot carry the name itself.
        /// A path - `Article::FIELDS.topics()` - also binds the list to the
        /// surrounding `Form`'s value when there is no `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Vec<String>>,
        /// Shown while there are no tags and nothing has been typed.
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows an x that empties the field.
        #[props(default)]
        clearable: Option<bool>,
        /// Draws one tag. Defaults to the text in a `Chip` with an x. A caller
        /// who overrides it draws the whole chip, remove control included -
        /// `args.remove` is the wiring.
        ///
        /// Make that control a `<button>` with `tabindex: "-1"`: the input is
        /// the field's one tab stop, and the chip cursor (ArrowLeft from an
        /// empty input) focuses each tag's button in turn. The field already
        /// cancels `mousedown` on each tag, so a click never moves the focus
        /// there.
        #[props(default)]
        tag: Option<Callback<SelectionArgs<String>, Element>>,
    }
}

/// A field whose value is a list of free-typed strings, drawn as chips with the
/// editor between them.
///
/// A comma - or any `split_chars` entry - and Enter both commit what was typed;
/// Backspace on an empty input takes the last tag back. ArrowLeft on an empty
/// input walks the tags, Delete or Backspace removes the focused one and
/// ArrowRight past the last returns to the input. `suggestions` adds a
/// dropdown, which is the only thing that makes this more than a text field
/// with chips, and a picked suggestion becomes the same kind of tag a typed one
/// does.
///
/// To choose out of a fixed set instead, reach for `MultiSelect`: its value is
/// a `Vec<T>` over a real domain type, and it cannot be typed into.
#[component]
pub fn TagsField(props: TagsFieldProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.tags_field.size);
    let radius = props.radius.copied_or(theme.tags_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    // Three ways to change the list, so three refusals: the draft input
    // (native `readonly`), the keys that commit or delete a tag, and the
    // chips' own remove buttons. The clear button stands down with them.
    let readonly = props.readonly.unwrap_or(false);
    let held = bound.value().unwrap_or_else(|| props.value.clone());
    let announcer = use_chip_announcer(held.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("TagsField: without `onchange` the tags can never change.");
    }

    // The draft is the component's, the way `SelectCore` owns its query: it is
    // not the value, nothing outside can act on it, and a controlled copy would
    // be one more thing for every call site to hold.
    let mut text = use_signal(String::new);
    let state = use_combobox();
    // What the x hands the focus to once it has cleared the field.
    let input_element = use_element();
    let cursor = TagCursor {
        slot: use_element(),
        input: input_element,
    };

    let rules = TagRules {
        allow_duplicates: props.allow_duplicates.unwrap_or(false),
        max_tags: props.max_tags,
        rule: props.tag_rules,
    };
    let split_chars = props
        .split_chars
        .clone()
        .unwrap_or_else(|| vec![",".to_string()]);

    let suggestions = props.suggestions.clone();
    let has_suggestions = suggestions.is_some();

    // The label also names the listbox, which `for` cannot reach.
    let field = use_field()
        .label_with_id()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&held))
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

    let onchange = bound.emit(props.onchange);

    // Every path that can add a tag goes through this one: typing a splitter,
    // pasting, Enter, blur and picking a suggestion all merge the same way, so
    // `max_tags` and the duplicate rule cannot disagree between them.
    let merging = held.clone();
    let onrefuse = props.onrefuse;
    let emit = onchange.clone();
    // An `Rc` rather than a `use_callback`: blur is one of the paths that calls
    // it, and nothing focus or blur can re-enter goes through `use_callback`
    // ([[codebase/reentrant-handlers]]).
    // Returns the first refused tag, which the draft keeps.
    let refusals = use_announcer();
    let localization = use_localization();
    let words = localization.tags_field;
    let add: Add = Rc::new(move |pieces: Vec<String>| {
        let merged = merge(&merging, pieces, &rules, &onrefuse);
        if let Some(next) = merged.next
            && let Some(emit) = &emit
        {
            emit(next);
        }
        if let Some(message) = refusal_message(&merged.refused, &words) {
            refusals.say(message);
        }
        merged.refused.into_iter().next().map(|(tag, _)| tag)
    });

    let tags = tags_field_chips(
        &held,
        &onchange,
        props.tag,
        size,
        disabled || readonly,
        cursor,
    );

    let rows = tags_field_rows(suggestions, &held, &text(), &add, text, state);
    let row_count = rows.len();
    let nothing_found = (!text().trim().is_empty()
        && props
            .suggestions
            .as_ref()
            .is_some_and(|all| !all.is_empty()))
    .then(|| localization.combobox.nothing_found.to_string());

    let clear_change = onchange.clone();
    let clearable = props.clearable.unwrap_or(false);
    let clear = clear_button(
        clearable && !(held.is_empty() && text().is_empty()) && !disabled && !readonly,
        size,
        input_element,
        move |_| {
            text.set(String::new());
            state.set_active(None);
            if let Some(onchange) = &clear_change {
                onchange(Vec::new());
            }
        },
    );

    let frame = use_field_frame()
        .trailing(&clear)
        .states(field.states())
        .prepare();

    // The frame draws the ring, so neither the slot nor the input draws a
    // second one.
    let slot = use_box()
        .framework_sx(&TAGS_VALUE_SX)
        .focus_ring(false)
        .prepare();
    let control = use_box()
        .framework_sx(&TAGS_INPUT_SX)
        .focus_ring(false)
        .prepare();

    let input = tags_field_input(
        field.aria(control),
        Draft {
            text,
            state,
            held: held.clone(),
            add,
            split_chars,
            has_suggestions,
            disabled,
            readonly,
            required,
            row_count,
            placeholder: props.placeholder,
            cursor,
        },
        onchange.clone(),
        props.attributes,
    );

    let control = slot.element(&cursor.slot).render(
        HtmlTag::Div,
        Vec::new(),
        // The input is not the frame's child, so the frame's own ring overlay
        // is not its sibling; this one is, and the frame still positions it.
        rsx! {
            {tags}
            {input}
            {ring_overlay()}
        },
    );

    let hidden = tags_field_hidden(bound.name().map(str::to_string), &held, disabled);

    let framed = frame.render(control);
    let body = match has_suggestions {
        true => rsx! {
            ComboboxCore {
                rows,
                active: state.active(),
                onactive: move |row| state.set_active(Some(row)),
                // No row left to offer is not open: `aria-expanded` must not
                // claim a popup that draws nothing.
                opened: state.is_open()
                    && !disabled
                    && !readonly
                    && (row_count > 0 || nothing_found.is_some()),
                onopened: move |opened| state.set_open(opened),
                state,
                nothing_found,
                caret_keys: CaretKeys::Unhighlighted,
                labelled_by: field.label_id(),
                size,
                radius,
                disabled: disabled || readonly,
                // The list stays up after a pick, the way Mantine's `TagsInput`
                // and our own `MultiSelect` keep it up: the picked row leaves
                // the list and the next one is one key away. `Autocomplete`
                // closes because its single value is then settled.
                close_on_pick: false,
                width: PopoverWidth::Match,
                // A tag added or taken back resizes the frame under an open
                // list.
                remeasure: held.len() as u64,
                {framed}
            }
        },
        false => framed,
    };

    field.render(rsx! {
        {body}
        {hidden}
        {announcer}
        {refusals.render()}
    })
}

/// The one path that adds tags. Returns the first refused tag, if any.
type Add = Rc<dyn Fn(Vec<String>) -> Option<String>>;

/// What one tag has to satisfy before it joins the list.
struct TagRules {
    allow_duplicates: bool,
    max_tags: Option<usize>,
    rule: Option<Callback<String, bool>>,
}

/// Why a tag was turned away.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Refusal {
    Duplicate,
    Full,
    NotAllowed,
}

/// What one edit did: the list to emit, and the tags it turned away.
#[derive(Debug, PartialEq)]
struct Merged {
    /// `None` when nothing was added, which keeps a refused edit from
    /// emitting a change.
    next: Option<Vec<String>>,
    refused: Vec<(String, Refusal)>,
}

/// Folds every candidate into the list one at a time, so a batch behaves
/// exactly like the same tags typed one after another.
fn merge(
    held: &[String],
    pieces: Vec<String>,
    rules: &TagRules,
    onrefuse: &Option<EventHandler<String>>,
) -> Merged {
    let mut next = held.to_vec();
    let mut added = false;
    let mut refused = Vec::new();
    for piece in pieces {
        let tag = piece.trim().to_string();
        if tag.is_empty() {
            continue;
        }
        let folded = tag.to_lowercase();
        let refusal = if !rules.allow_duplicates
            && next.iter().any(|held| held.trim().to_lowercase() == folded)
        {
            Some(Refusal::Duplicate)
        } else if rules.max_tags.is_some_and(|max| next.len() >= max) {
            Some(Refusal::Full)
        } else if rules.rule.is_some_and(|rule| !rule.call(tag.clone())) {
            Some(Refusal::NotAllowed)
        } else {
            None
        };
        if let Some(refusal) = refusal {
            if let Some(onrefuse) = onrefuse {
                onrefuse.call(tag.clone());
            }
            refused.push((tag, refusal));
            continue;
        }
        next.push(tag);
        added = true;
    }
    Merged {
        next: added.then_some(next),
        refused,
    }
}

/// One sentence per reason, in a fixed order; `None` when nothing was refused.
fn refusal_message(refused: &[(String, Refusal)], words: &TagsFieldLabels) -> Option<String> {
    let sentences: Vec<String> = [
        (Refusal::Duplicate, words.duplicate),
        (Refusal::Full, words.full),
        (Refusal::NotAllowed, words.not_allowed),
    ]
    .into_iter()
    .filter_map(|(reason, template)| {
        let labels: Vec<&str> = refused
            .iter()
            .filter(|(_, why)| *why == reason)
            .map(|(tag, _)| tag.as_str())
            .collect();
        (!labels.is_empty()).then(|| fill(template, &[("labels", &labels.join(", "))]))
    })
    .collect();
    (!sentences.is_empty()).then(|| sentences.join(". "))
}

/// The pieces a string breaks into, or nothing at all when it holds no splitter
/// - which is how the caller tells "commit these" from "this is still a draft".
fn split(text: &str, split_chars: &[String]) -> Vec<String> {
    let splitting = split_chars
        .iter()
        .any(|char| !char.is_empty() && text.contains(char.as_str()));
    if !splitting {
        return Vec::new();
    }
    let mut pieces = vec![text.to_string()];
    for separator in split_chars.iter().filter(|char| !char.is_empty()) {
        pieces = pieces
            .into_iter()
            .flat_map(|piece| {
                piece
                    .split(separator.as_str())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    pieces.retain(|piece| !piece.trim().is_empty());
    pieces
}

/// Where the chip cursor moves the focus: the tags' slot and the draft input.
#[derive(Clone, Copy)]
struct TagCursor {
    slot: ElementHandle,
    input: ElementHandle,
}

impl TagCursor {
    /// Focuses tag `index`'s button, or the input for `None`.
    fn focus(self, index: Option<usize>) {
        let _ = match index {
            Some(index) => self
                .slot
                .query_selector(&tag_selector(index))
                .and_then(|button| button.focus()),
            None => self.input.focus(),
        };
    }

    /// Whether the focus is on tag `index`'s button.
    fn holds(self, index: usize) -> bool {
        self.slot
            .query_selector(&tag_selector(index))
            .is_ok_and(|button| button.is_focused())
    }
}

fn tag_selector(index: usize) -> String {
    format!("[data-tag-index='{index}'] button")
}

/// Where the cursor goes once tag `index` of `count` is removed: the next tag,
/// which slides into its place, else the one before, else the input.
fn after_removal(index: usize, count: usize) -> Option<usize> {
    match index + 1 < count {
        true => Some(index),
        false => index.checked_sub(1),
    }
}

/// The chips, one per held tag. The input is the one tab stop; the chip cursor
/// moves the focus onto a tag's button, and a removal moves it on before the
/// tag goes ([[principles/focus-after-removal]]).
fn tags_field_chips<F: Fn(Vec<String>) + Clone + 'static>(
    held: &[String],
    onchange: &Option<F>,
    draw_tag: Option<Callback<SelectionArgs<String>, Element>>,
    size: Size,
    locked: bool,
    cursor: TagCursor,
) -> Element {
    let count = held.len();
    let chips = held.iter().cloned().enumerate().map(|(index, value)| {
        let list = held.to_vec();
        let onchange = onchange.clone();
        // Guarded here as well as by disabling the x: a caller's own `tag`
        // gets `remove` too, and Backspace refuses a locked field.
        let remove = Callback::new(move |_: ()| {
            let (false, Some(onchange)) = (locked, &onchange) else {
                return;
            };
            // A mouse removal never focused the tag, so only the keyboard's
            // focus is moved.
            if cursor.holds(index) {
                cursor.focus(after_removal(index, count));
            }
            let mut next = list.clone();
            next.remove(index);
            onchange(next);
        });
        let chip = match &draw_tag {
            Some(tag) => tag.call(SelectionArgs { value, remove }),
            None => removable_chip(value, remove, size, locked),
        };
        (chip, remove)
    });

    rsx! {
        for (index , (chip , remove)) in chips.enumerate() {
            span {
                key: "{index}",
                "data-slot": "tag",
                "data-tag-index": "{index}",
                // The default chip's own guard, applied here so a caller's
                // `tag` has it too: a press on its x must not take the focus
                // off the input, or the removal strands it on the body.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                onkeydown: move |event: KeyboardEvent| {
                    let key = event.key();
                    // Tab leaves, Escape closes the list and chords are the
                    // browser's; every other key is the cursor's, not the
                    // combobox's around it.
                    if matches!(key, Key::Tab | Key::Escape) || !event.modifiers().is_empty() {
                        return;
                    }
                    event.stop_propagation();
                    match key {
                        Key::ArrowLeft => cursor.focus(Some(index.saturating_sub(1))),
                        Key::ArrowRight => cursor.focus(Some(index + 1).filter(|next| *next < count)),
                        Key::Delete | Key::Backspace if !locked => remove.call(()),
                        _ => return,
                    }
                    event.prevent_default();
                },
                {chip}
            }
        }
    }
}

/// The suggestion rows: what is already held is never offered again, and the
/// draft narrows what is left.
///
/// Drawn eagerly and handed down as values: a `Callback` would let the rows
/// memoize and a narrowed list would leave stale ones on screen.
fn tags_field_rows(
    suggestions: Option<Vec<String>>,
    held: &[String],
    draft: &str,
    add: &Add,
    mut text: Signal<String>,
    state: ComboboxState,
) -> Vec<Element> {
    let query = draft.trim().to_lowercase();
    let picked: Vec<String> = held.iter().map(|tag| tag.trim().to_lowercase()).collect();

    suggestions
        .unwrap_or_default()
        .into_iter()
        .filter(|suggestion| {
            let folded = suggestion.trim().to_lowercase();
            !picked.contains(&folded) && folded.contains(&query)
        })
        .map(|suggestion| {
            let label = suggestion.clone();
            let pick = add.clone();
            rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        // The draft was only a filter here, so it goes either way.
                        let _ = pick(vec![suggestion.clone()]);
                        text.set(String::new());
                        state.set_active(None);
                    },
                    {row_label(label)}
                }
            }
        })
        .collect()
}

/// A hidden input per tag is how the list posts: the visible one holds the
/// draft, so it carries no `name` at all. The same shape `MultiSelect` uses,
/// and the same one a native `<select multiple>` sends.
fn tags_field_hidden(name: Option<String>, held: &[String], disabled: bool) -> Option<Element> {
    let held = held.to_vec();
    name.map(|name| {
        rsx! {
            for (index , tag) in held.iter().cloned().enumerate() {
                input {
                    key: "{index}",
                    r#type: "hidden",
                    name: name.clone(),
                    value: tag,
                    disabled: disabled.then_some(true),
                }
            }
        }
    })
}

/// What the draft input reads and writes. `add` is the one path that can make
/// a tag; `onchange` is what Backspace uses to take one back.
struct Draft {
    text: Signal<String>,
    state: ComboboxState,
    held: Vec<String>,
    add: Add,
    split_chars: Vec<String>,
    has_suggestions: bool,
    disabled: bool,
    readonly: bool,
    required: bool,
    row_count: usize,
    placeholder: Option<String>,
    cursor: TagCursor,
}

/// The draft input: the field's id, label and `aria-describedby` land here,
/// because this is what the caller types into and what `<label for>` may name.
fn tags_field_input<F: Fn(Vec<String>) + Clone + 'static>(
    control: BoxStyle,
    draft: Draft,
    onchange: Option<F>,
    extra: Vec<Attribute>,
) -> Element {
    let Draft {
        mut text,
        state,
        held,
        add,
        split_chars,
        has_suggestions,
        disabled,
        readonly,
        required,
        row_count,
        placeholder,
        cursor,
    } = draft;
    // One `Rc` per path that can add a tag: typing a splitter, Enter and blur
    // all merge through the same closure.
    let (typing, entering, blurring) = (add.clone(), add.clone(), add);

    let mut attributes = match has_suggestions {
        true => state.a11y_attributes(),
        // Without a dropdown there is no listbox for `aria-controls` to name
        // and nothing for the arrows to move, so the input is what it looks
        // like: a text input.
        false => Vec::new(),
    };
    attributes.extend(extra);
    control
        .attr_default("type", "text")
        .attr("aria-autocomplete", has_suggestions.then_some("list"))
        .attr("value", text())
        .attr("data-controlled", true)
        .attr(
            "placeholder",
            held.is_empty().then_some(placeholder).flatten(),
        )
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", (required && held.is_empty()).then_some(true))
        .attr("autocomplete", "off")
        .event("oninput", move |event: FormEvent| {
            let raw = event.value();
            let pieces: Vec<String> = split(&raw, &split_chars);
            match pieces.is_empty() {
                // Nothing to commit - the text is only separators. Keeping what
                // arrived is what keeps the signal and the DOM agreeing: a
                // value the vdom already rendered is not written back, so
                // normalising `","` to `""` would leave the comma on screen.
                true if raw != text() => text.set(raw),
                true => {}
                // A refused tag stays in the draft, to fix or drop.
                false => text.set(typing(pieces).unwrap_or_default()),
            }
            // The list changes under the highlight, so typing disarms it: the
            // next Enter belongs to what was typed, not to a row that happens
            // to sit where the old one did.
            state.set_active(None);
            if has_suggestions {
                state.open();
            }
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            if disabled || readonly {
                return;
            }
            match event.key() {
                // A highlighted row is the core's Enter, and it must never mean
                // two things. Nothing highlighted and nothing typed is nobody's,
                // so it bubbles and a form still submits.
                Key::Enter
                    if !(state.is_open() && state.active().is_some() && row_count > 0)
                        && !text().trim().is_empty() =>
                {
                    event.prevent_default();
                    if entering(vec![text()]).is_none() {
                        text.set(String::new());
                    }
                    state.set_active(None);
                }
                // The input is the value's own editor here, not a filter over a
                // list - which is why this reverses the call `MultiSelect`'s
                // search box made. With text in it, Backspace only ever edits.
                Key::Backspace if text().is_empty() && !held.is_empty() => {
                    let Some(onchange) = &onchange else {
                        return;
                    };
                    event.prevent_default();
                    let mut next = held.clone();
                    next.pop();
                    onchange(next);
                }
                // Only on an empty draft: the caret's position is not readable
                // here, and leaving would commit the draft on blur.
                Key::ArrowLeft
                    if text().is_empty() && !held.is_empty() && event.modifiers().is_empty() =>
                {
                    event.prevent_default();
                    event.stop_propagation();
                    cursor.focus(Some(held.len() - 1));
                }
                _ => {}
            }
        })
        // What was typed and not committed is still what the user meant, so it
        // becomes a tag rather than being thrown away. Not a prop: nobody turns
        // it off, and one for "discard my typing" is not worth the line.
        .event("onblur", move |_: FocusEvent| {
            let draft = text();
            if !readonly && !draft.trim().is_empty() && blurring(vec![draft]).is_none() {
                text.set(String::new());
            }
            state.close();
        })
        .element(&cursor.input)
        .render(HtmlTag::Input, attributes, ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn rules(allow_duplicates: bool, max_tags: Option<usize>) -> TagRules {
        TagRules {
            allow_duplicates,
            max_tags,
            rule: None,
        }
    }

    /// No splitter in the text means "still a draft", which is how `oninput`
    /// tells a keystroke from a commit.
    #[test]
    fn splitting_only_reports_pieces_once_a_separator_is_there() {
        assert!(split("rus", &tags(&[","])).is_empty());
        assert_eq!(split("rust,", &tags(&[","])), tags(&["rust"]));
        assert_eq!(split("a,b;c", &tags(&[",", ";"])), tags(&["a", "b", "c"]));
        // Only separators: nothing to commit, and nothing to clear either.
        assert!(split(",,", &tags(&[","])).is_empty());
    }

    /// A paste and the same tags typed one after another have to land the same
    /// way, which is what folding one at a time buys.
    #[test]
    fn max_tags_fills_the_room_that_is_left_rather_than_refusing_the_batch() {
        let merged = merge(
            &tags(&["rust"]),
            tags(&["dioxus", "wasm", "css"]),
            &rules(false, Some(3)),
            &None,
        );
        assert_eq!(merged.next, Some(tags(&["rust", "dioxus", "wasm"])));
        assert_eq!(merged.refused, [("css".to_string(), Refusal::Full)]);
    }

    /// Todo 545: a refusal says why, one sentence per reason.
    #[test]
    fn a_refusal_names_the_tags_and_the_reason() {
        let merged = merge(
            &tags(&["rust", "css"]),
            tags(&["Rust", "wasm", "CSS"]),
            &rules(false, Some(3)),
            &None,
        );
        assert_eq!(
            refusal_message(&merged.refused, &TagsFieldLabels::ENGLISH),
            Some("Already added: Rust, CSS".to_string())
        );
        let full = [
            ("a".to_string(), Refusal::Full),
            ("b".to_string(), Refusal::Duplicate),
        ];
        assert_eq!(
            refusal_message(&full, &TagsFieldLabels::ENGLISH),
            Some("Already added: b. Tag limit reached, not added: a".to_string())
        );
        assert_eq!(refusal_message(&[], &TagsFieldLabels::ENGLISH), None);
    }

    /// Trimmed and lowercased on both sides - and a batch is compared against
    /// what it has already added, not only against what was held.
    #[test]
    fn duplicates_are_refused_unless_they_are_allowed() {
        assert_eq!(
            merge(
                &tags(&["Rust"]),
                tags(&[" rust "]),
                &rules(false, None),
                &None
            )
            .next,
            None
        );
        assert_eq!(
            merge(&tags(&[]), tags(&["a", "A"]), &rules(false, None), &None).next,
            Some(tags(&["a"]))
        );
        assert_eq!(
            merge(&tags(&["Rust"]), tags(&["rust"]), &rules(true, None), &None).next,
            Some(tags(&["Rust", "rust"]))
        );
    }

    /// The cursor stays in the chips while any are left, and prefers the tag
    /// that slides into the removed one's place.
    #[test]
    fn the_cursor_lands_on_the_next_tag_then_the_previous_then_the_input() {
        assert_eq!(after_removal(0, 3), Some(0));
        assert_eq!(after_removal(1, 3), Some(1));
        assert_eq!(after_removal(2, 3), Some(1));
        assert_eq!(after_removal(0, 1), None);
    }

    /// Nothing added means nothing changed, so a refused edit never emits an
    /// `onchange` the caller would have to recognise as a no-op.
    #[test]
    fn an_edit_that_adds_nothing_reports_no_change() {
        assert_eq!(
            merge(&tags(&["a"]), tags(&["  "]), &rules(false, None), &None).next,
            None
        );
        assert_eq!(
            merge(&tags(&["a"]), tags(&["b"]), &rules(false, Some(1)), &None).next,
            None
        );
    }
}
