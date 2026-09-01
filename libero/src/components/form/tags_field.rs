use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Chip, ComboboxCore, ComboboxOption, HtmlTag, Input, SelectionArgs,
        common::field_props,
        form::{field_control_sx, glyphs::CloseIcon, use_bound, use_field, use_field_frame},
        layout::use_box,
        use_combobox,
    },
    hooks::{PopoverWidth, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHIP_HEIGHT, Size},
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
        .selector(
            "& > [data-slot='tag']",
            sx().display("inline-flex").max_width("100%").min_width("0"),
        )
        // A tag is free text, so one can be arbitrarily long. The label is the
        // only part that may shrink: without `min-width: 0` a flex item's floor
        // is its min-content size, which for an unbroken 240-character tag is
        // the whole string - and the x was then pushed clean out of its own
        // chip. Seen in a browser; the box model alone looked correct.
        .selector(
            "& [data-slot='label']",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        // A flex line of its own, or the x hangs off the label's baseline. It
        // never shrinks - an x too narrow to hit is worse than a clipped label.
        .selector(
            "& [data-slot='remove']",
            sx().display("inline-flex")
                .align_items("center")
                .flex("0 0 auto"),
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
        #[props(default)]
        tag: Option<Callback<SelectionArgs<String>, Element>>,
    }
}

/// A field whose value is a list of free-typed strings, drawn as chips with the
/// editor between them.
///
/// A comma - or any `split_chars` entry - and Enter both commit what was typed;
/// Backspace on an empty input takes the last tag back. `suggestions` adds a
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
    let held = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("TagsField: without `onchange` the tags can never change.");
    }

    // The draft is the component's, the way `SelectCore` owns its query: it is
    // not the value, nothing outside can act on it, and a controlled copy would
    // be one more thing for every call site to hold.
    let mut text = use_signal(String::new);
    let state = use_combobox();
    let value_element = use_element();
    let input_element = use_element();
    // Removing a chip destroys the button the keyboard was on - the chips are
    // real tab stops here, because without a chip cursor Backspace could only
    // ever reach the last one. See [[principles/focus-after-removal]].
    let mut owed = use_signal(|| None::<usize>);

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

    let field = use_field()
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
    let add: Rc<dyn Fn(Vec<String>)> = Rc::new(move |pieces: Vec<String>| {
        if let Some(next) = merge(&merging, pieces, &rules, &onrefuse)
            && let Some(emit) = &emit
        {
            emit(next);
        }
    });

    // The chips. Each carries a stable id, so the focus repair can find the one
    // that took a removed chip's place without a hook per row.
    let chip_size = Size::ALL[size.index().saturating_sub(1)];
    let remove_prefix = format!("{}-tag", field.id());
    let removing = held.clone();
    let remove_change = onchange.clone();
    let draw_tag = props.tag;
    let chips = held.iter().cloned().enumerate().map(|(index, value)| {
        let list = removing.clone();
        let onchange = remove_change.clone();
        let remove = Callback::new(move |_: ()| {
            let Some(onchange) = &onchange else {
                return;
            };
            owed.set(Some(index));
            let mut next = list.clone();
            next.remove(index);
            onchange(next);
        });
        let id = format!("{remove_prefix}-{index}");
        match &draw_tag {
            Some(tag) => tag.call(SelectionArgs { value, remove }),
            None => default_tag(&value, remove, chip_size, &id, disabled),
        }
    });
    let tags = rsx! {
        for (index, chip) in chips.enumerate() {
            span { key: "{index}", "data-slot": "tag", {chip} }
        }
    };

    // Focus goes to the chip that took the removed one's place, to the new last
    // chip when the removed one was last, and to the input when nothing is
    // left - which is also the control that adds the next tag.
    let remaining = held.len();
    use_effect(use_reactive!(|remaining| {
        let Some(index) = *owed.peek() else {
            return;
        };
        owed.set(None);
        if remaining == 0 {
            let _ = input_element.focus();
            return;
        }
        // A caller's own `tag` may carry no id, so the query simply misses -
        // the input is the fallback rather than leaving focus on the body.
        let at = index.min(remaining - 1);
        match value_element.query_selector(&format!("#{remove_prefix}-{at}-remove")) {
            Ok(button) => {
                let _ = button.focus();
            }
            Err(_) => {
                let _ = input_element.focus();
            }
        }
    }));

    // The rows: what is already held is never offered again, and the draft
    // narrows what is left.
    let query = text().trim().to_lowercase();
    let picked: Vec<String> = held.iter().map(|tag| tag.trim().to_lowercase()).collect();
    let matches: Vec<String> = suggestions
        .unwrap_or_default()
        .into_iter()
        .filter(|suggestion| {
            let folded = suggestion.trim().to_lowercase();
            !picked.contains(&folded) && folded.contains(&query)
        })
        .collect();
    let row_count = matches.len();

    // Drawn eagerly and handed down as values: a `Callback` would let the rows
    // memoize and a narrowed list would leave stale ones on screen.
    let rows: Vec<Element> = matches
        .into_iter()
        .map(|suggestion| {
            let label = suggestion.clone();
            let pick = add.clone();
            rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        pick(vec![suggestion.clone()]);
                        text.set(String::new());
                        state.set_active(None);
                    },
                    "{label}"
                }
            }
        })
        .collect();

    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(size).into();
    let clear_change = onchange.clone();
    let clearable = props.clearable.unwrap_or(false);
    let clear = (clearable && !(held.is_empty() && text().is_empty()) && !disabled).then(|| {
        rsx! {
            ActionIcon {
                aria_label: "Clear",
                size: icon_size,
                onclick: move |_| {
                    text.set(String::new());
                    state.set_active(None);
                    if let Some(onchange) = &clear_change {
                        onchange(Vec::new());
                    }
                },
                CloseIcon {}
            }
        }
    });

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

    let splitting = split_chars.clone();
    let typing = add.clone();
    let entering = add.clone();
    let blurring = add;
    let backspacing = held.clone();
    let backspace_change = onchange.clone();

    let mut attributes = match has_suggestions {
        true => state.a11y_attributes(),
        // Without a dropdown there is no listbox for `aria-controls` to name
        // and nothing for the arrows to move, so the input is what it looks
        // like: a text input.
        false => Vec::new(),
    };
    attributes.extend(props.attributes);
    let input = field
        .aria(control)
        .element(&input_element)
        .attr_default("type", "text")
        .attr("aria-autocomplete", has_suggestions.then_some("list"))
        .attr("value", text())
        .attr("data-controlled", true)
        .attr(
            "placeholder",
            (held.is_empty()).then_some(props.placeholder).flatten(),
        )
        .attr("disabled", disabled)
        .attr("required", (required && held.is_empty()).then_some(true))
        .attr("autocomplete", "off")
        .event("oninput", move |event: FormEvent| {
            let raw = event.value();
            let pieces: Vec<String> = split(&raw, &splitting);
            match pieces.is_empty() {
                // Nothing to commit - the text is only separators. Keeping what
                // arrived is what keeps the signal and the DOM agreeing: a
                // value the vdom already rendered is not written back, so
                // normalising `","` to `""` would leave the comma on screen.
                true if raw != text() => text.set(raw),
                true => {}
                false => {
                    typing(pieces);
                    text.set(String::new());
                }
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
            if disabled {
                return;
            }
            match event.key() {
                // A highlighted row is the core's Enter, and it must never mean
                // two things. Nothing highlighted and nothing typed is nobody's,
                // so it bubbles and a form still submits.
                Key::Enter
                    if !(state.opened() && state.active().is_some() && row_count > 0)
                        && !text().trim().is_empty() =>
                {
                    event.prevent_default();
                    entering(vec![text()]);
                    text.set(String::new());
                    state.set_active(None);
                }
                // The input is the value's own editor here, not a filter over a
                // list - which is why this reverses the call `MultiSelect`'s
                // search box made. With text in it, Backspace only ever edits.
                Key::Backspace if text().is_empty() && !backspacing.is_empty() => {
                    let Some(onchange) = &backspace_change else {
                        return;
                    };
                    event.prevent_default();
                    let mut next = backspacing.clone();
                    next.pop();
                    onchange(next);
                }
                _ => {}
            }
        })
        // What was typed and not committed is still what the user meant, so it
        // becomes a tag rather than being thrown away. Not a prop: nobody turns
        // it off, and one for "discard my typing" is not worth the line.
        .event("onblur", move |_: FocusEvent| {
            let draft = text();
            if !draft.trim().is_empty() {
                blurring(vec![draft]);
                text.set(String::new());
            }
            state.close();
        })
        .render(HtmlTag::Input, attributes, ());

    let control = slot.element(&value_element).render(
        HtmlTag::Div,
        Vec::new(),
        rsx! {
            {tags}
            {input}
        },
    );

    // A hidden input per tag is how the list posts: the visible one holds the
    // draft, so it carries no `name` at all. The same shape `MultiSelect` uses,
    // and the same one a native `<select multiple>` sends.
    let hidden = bound.name().map(str::to_string).map(|name| {
        rsx! {
            for (index, tag) in held.iter().cloned().enumerate() {
                input {
                    key: "{index}",
                    r#type: "hidden",
                    name: name.clone(),
                    value: tag,
                    disabled: disabled.then_some(true),
                }
            }
        }
    });

    let framed = frame.render(control);
    let body = match has_suggestions {
        true => rsx! {
            ComboboxCore {
                rows,
                active: state.active(),
                onactive: move |row| state.set_active(Some(row)),
                opened: state.opened() && !disabled,
                onopened: move |opened| state.set_opened(opened),
                id: state.id(),
                size,
                radius,
                disabled,
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
    })
}

/// What one tag has to satisfy before it joins the list.
struct TagRules {
    allow_duplicates: bool,
    max_tags: Option<usize>,
    rule: Option<Callback<String, bool>>,
}

/// Folds every candidate into the list one at a time, so a batch behaves
/// exactly like the same tags typed one after another. `None` when nothing was
/// added, which is what keeps a refused edit from emitting a change.
fn merge(
    held: &[String],
    pieces: Vec<String>,
    rules: &TagRules,
    onrefuse: &Option<EventHandler<String>>,
) -> Option<Vec<String>> {
    let mut next = held.to_vec();
    let mut added = false;
    for piece in pieces {
        let tag = piece.trim().to_string();
        if tag.is_empty() {
            continue;
        }
        let folded = tag.to_lowercase();
        let refused = (!rules.allow_duplicates
            && next.iter().any(|held| held.trim().to_lowercase() == folded))
            || rules.max_tags.is_some_and(|max| next.len() >= max)
            || rules.rule.is_some_and(|rule| !rule.call(tag.clone()));
        if refused {
            if let Some(onrefuse) = onrefuse {
                onrefuse.call(tag);
            }
            continue;
        }
        next.push(tag);
        added = true;
    }
    added.then_some(next)
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

/// The default tag: the text, and an x that drops it.
///
/// The x is a real tab stop, unlike `MultiSelect`'s: there the trigger owns a
/// chip cursor the arrows move, and here there is none - `ElementApi` cannot
/// report a caret, so "ArrowLeft only at caret 0" is unimplementable and
/// Backspace alone would leave every chip but the last unreachable.
fn default_tag(value: &str, remove: Callback<()>, size: Size, id: &str, disabled: bool) -> Element {
    let label = value.to_string();
    // A fraction of the chip's own height, not of its font: the two do not
    // scale at the same rate, so an `em` x shrinks against its chip as the
    // field grows.
    let icon_size: Input<ThemeAwareValue> =
        ThemeAwareValue::String(format!("calc({} * 0.6)", CHIP_HEIGHT.value(size))).into();
    rsx! {
        Chip { size,
            // Its own element, so it is a flex item the value slot can let
            // shrink. A bare text node is an anonymous one, which no selector
            // reaches.
            span { "data-slot": "label", "{label}" }
            span {
                // Centred by the value slot's own sx - a bare inline span would
                // hang the button off the label's baseline.
                "data-slot": "remove",
                // The input holds the focus that keeps the list open, so a
                // press must not move it onto the button before the click.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                ActionIcon {
                    id: "{id}-remove",
                    aria_label: "Remove {label}",
                    size: icon_size,
                    disabled,
                    // A native `<button>` inherits neither `color` nor
                    // `font-size` - it takes the UA's `buttontext` and 13.3px.
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
                    onclick: move |_| remove.call(()),
                    CloseIcon {}
                }
            }
        }
    }
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
        let next = merge(
            &tags(&["rust"]),
            tags(&["dioxus", "wasm", "css"]),
            &rules(false, Some(3)),
            &None,
        );
        assert_eq!(next, Some(tags(&["rust", "dioxus", "wasm"])));
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
            ),
            None
        );
        assert_eq!(
            merge(&tags(&[]), tags(&["a", "A"]), &rules(false, None), &None),
            Some(tags(&["a"]))
        );
        assert_eq!(
            merge(&tags(&["Rust"]), tags(&["rust"]), &rules(true, None), &None),
            Some(tags(&["Rust", "rust"]))
        );
    }

    /// Nothing added means nothing changed, so a refused edit never emits an
    /// `onchange` the caller would have to recognise as a no-op.
    #[test]
    fn an_edit_that_adds_nothing_reports_no_change() {
        assert_eq!(
            merge(&tags(&["a"]), tags(&["  "]), &rules(false, None), &None),
            None
        );
        assert_eq!(
            merge(&tags(&["a"]), tags(&["b"]), &rules(false, Some(1)), &None),
            None
        );
    }
}
