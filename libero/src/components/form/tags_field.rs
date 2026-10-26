use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::use_announcer,
        common::{
            ComboboxState, HtmlTag, Input, Part, Parts, ScaleOrCss, TOOLBAR_ITEM, ToolbarItem,
            aria_label_of, ring_overlay, use_combobox, use_no_toolbar, use_toolbar_item,
        },
        form::{
            CaretKeys, ChipFocus, ComboboxCore, ComboboxOption, DropdownPart, SelectionArgs,
            clear_button, field_control_sx, field_parts_enum, field_props, removable_chip,
            row_label, use_bound, use_chip_announcer, use_chip_handles, use_field, use_field_frame,
            use_prefill,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, PopoverWidth, use_localization, use_theme},
    localization::{TagsFieldLabels, fill},
    platform::{ElementApi, caret_edges, is_tab, logical_key},
    sx::{StaticSx, sx},
    theme::Size,
    utils::warn,
};

/// The chips and the draft input on one wrapping flow. The field's id and
/// label still land on the input.
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
            format!("& > [data-slot='{}']", TagsFieldPart::Tag.slot()),
            sx().display("inline-flex").max_width("100%").min_width("0"),
        )
        // A flex line of its own, or the x hangs off the label's baseline.
        .selector(
            "& [data-slot='remove']",
            sx().display("inline-flex").align_items("center"),
        )
});

/// The draft input. `flex-basis`, not `min-width`, so it wraps onto its own
/// line instead of being squeezed beside the chips.
static TAGS_INPUT_SX: StaticSx = StaticSx::new(|| field_control_sx().flex("1 1 60px"));

field_parts_enum! {
    /// [`TagsField`]'s inner parts, for its `parts` prop: a field's, and the tags.
    pub enum TagsFieldPart framed {
        /// One tag's chip, before the draft input.
        Tag = "tag" => "& > [data-slot='frame'] > * > [data-slot='tag'], & > * > [data-slot='frame'] > * > [data-slot='tag']",
    }
}

field_props! {
    parts(TagsFieldPart);
    extends(input);
    pub struct TagsFieldProps {
        /// The tags, in order. Strictly controlled - pair it with `onchange`.
        #[props(default)]
        value: Vec<String>,
        /// Called with the whole list the caller should hold next.
        #[props(default)]
        onchange: Option<EventHandler<Vec<String>>>,
        /// Offers a dropdown of tags to pick; held tags are never offered.
        #[props(default)]
        suggestions: Option<Vec<String>>,
        /// Each one commits the text before it, typed or pasted. Defaults to `[","]`.
        #[props(default)]
        split_chars: Option<Vec<String>>,
        /// Lets the same tag be added twice. Off, the comparison is trimmed and
        /// case-insensitive.
        #[props(default)]
        allow_duplicates: Option<bool>,
        /// The most tags the field accepts; the rest are refused one by one.
        #[props(default)]
        max_tags: Option<usize>,
        /// Accepts or refuses one tag before it is added. Shows no message.
        #[props(default)]
        tag_rules: Option<Callback<String, bool>>,
        /// A tag was rejected, with why: a duplicate, past `max_tags`, or by
        /// `tag_rules`. Called after the edit's `onchange`.
        #[props(default)]
        onreject: Option<EventHandler<TagRejection>>,
        /// Rules over the whole list, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Vec<String>>,
        /// Posts one hidden input per tag. A path also binds the list to the
        /// surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Vec<String>>,
        /// Shown while there are no tags and nothing has been typed.
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows an x that empties the field.
        #[props(default)]
        clearable: Option<bool>,
        /// Draws one whole tag, remove control included (`args.remove`). Make
        /// that a `<button>` with `tabindex: "-1"`.
        #[props(default)]
        tag: Option<Callback<SelectionArgs<String>, Element>>,
        /// Styles the portaled `suggestions` dropdown and its inner parts.
        #[props(default, into)]
        dropdown_parts: Input<Parts<DropdownPart>>,
    }
}

/// A list of free-typed strings, drawn as chips with the editor between them.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::TagsField;
/// # fn app() -> Element {
/// let mut topics = use_signal(Vec::<String>::new);
/// rsx! {
///     TagsField {
///         label: "Topics",
///         value: topics(),
///         onchange: move |next| topics.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/tags-field>
#[component]
pub fn TagsField(props: TagsFieldProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.tags_field.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.tags_field.radius);
    let required = crate::components::form::fieldset_required(props.required);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    // Refused on the input (native), the commit/delete keys, the chips' x and
    // the clear button.
    let readonly = props.readonly.unwrap_or(false);
    let item = use_toolbar_item();
    // The chips' x and the clear button belong to the field, not the bar.
    use_no_toolbar();
    let held = bound.value().unwrap_or_else(|| props.value.clone());
    let announcer = use_chip_announcer(held.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("TagsField: without `onchange` the tags can never change.");
    }

    // The draft is the component's own, as `SelectCore` owns its query.
    let mut text = use_signal(String::new);
    let state = use_combobox();
    // The chips' focus walks as on `MultiSelect`, with the draft input as its trigger.
    let handles = use_chip_handles();
    let input_element = handles.trigger();
    // Set while the input hands focus to a tag, so its blur keeps the draft.
    let entering = use_hook(|| CopyValue::new(false));

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
        .empty(held.is_empty())
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let onchange = bound.emit(props.onchange);

    // Every path that adds a tag merges here, so the rules cannot disagree.
    let merging = held.clone();
    let onreject = props.onreject;
    let emit = onchange.clone();
    // An `Rc`, not a `use_callback`: blur calls it ([[codebase/reentrant-handlers]]).
    // Returns the first rejected tag, which the draft keeps.
    let refusals = use_announcer();
    let localization = use_localization();
    let words = localization.tags_field;
    let add: Add = Rc::new(move |pieces: Vec<String>| {
        let merged = merge(&merging, pieces, &rules);
        if let Some(next) = merged.next
            && let Some(emit) = &emit
        {
            emit(next);
        }
        // After the change, so a handler clearing on change keeps the rejection.
        if let Some(onreject) = &onreject {
            for rejection in &merged.refused {
                onreject.call(rejection.clone());
            }
        }
        if let Some(message) = refusal_message(&merged.refused, &words) {
            refusals.say(message);
        }
        merged
            .refused
            .into_iter()
            .next()
            .map(|rejection| rejection.tag)
    });

    // Taken as a blur would: no focus came first, so the text commits, and a refused tag stays.
    let (prefill_add, prefill_splits) = (add.clone(), split_chars.clone());
    use_prefill(
        input_element,
        Some(text.peek().clone()),
        onchange.is_some().then_some(move |raw: String| {
            let mut pieces = split(&raw, &prefill_splits);
            if pieces.is_empty() {
                pieces.push(raw);
            }
            let left = prefill_add(pieces).unwrap_or_default();
            let _ = input_element.set_value(&left);
            let mut text = text;
            text.set(left);
        }),
        disabled || readonly,
    );

    let focus = ChipFocus::new(handles, format!("{}-tag", field.id()), held.len());
    let tags = tags_field_chips(
        &held,
        &onchange,
        props.tag,
        size,
        (disabled, readonly),
        &focus,
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
        Some(&field),
        move |_| {
            text.set(String::new());
            state.set_active(None);
            if let Some(onchange) = &clear_change {
                onchange(Vec::new());
            }
        },
    );
    // The x's press blurs the input before its click, and that blur must not commit the draft.
    let mut clearing = use_hook(|| CopyValue::new(false));
    let clear = clear.map(|button| {
        rsx! {
            span {
                style: "display: contents",
                onmousedown: move |_| clearing.set(true),
                {button}
            }
        }
    });

    let frame = use_field_frame()
        .trailing(&clear)
        .states(field.states())
        // As the input's own: only while no tag is held.
        .placeholder(props.placeholder.as_deref().filter(|_| held.is_empty()))
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

    let aria_label = aria_label_of(&props.attributes);
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
            input: input_element,
            focus,
            entering,
            clearing,
            item,
        },
        onchange.clone(),
        props.attributes,
    );

    let control = slot.element(&handles.slot()).render(
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
                aria_label,
                size,
                radius: radius.value(),
                disabled: disabled || readonly,
                // Stays up after a pick, as on `MultiSelect`.
                close_on_pick: false,
                width: PopoverWidth::Match,
                // A tag added or taken back resizes the frame under an open list.
                remeasure: held.len() as u64,
                parts: props.dropdown_parts,
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

/// A typed or pasted tag the field turned away, handed to `onreject`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagRejection {
    /// Trimmed, as it would have been added.
    pub tag: String,
    pub reason: TagRejectReason,
}

/// Why a `TagsField` turned a tag away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagRejectReason {
    /// Already held, and `allow_duplicates` is off.
    Duplicate,
    /// The field holds `max_tags`.
    Full,
    /// `tag_rules` refused it.
    NotAllowed,
}

/// What one edit did: the list to emit, and the tags it turned away.
#[derive(Debug, PartialEq)]
struct Merged {
    /// `None` when nothing was added, which keeps a refused edit from
    /// emitting a change.
    next: Option<Vec<String>>,
    refused: Vec<TagRejection>,
}

/// Folds every candidate into the list one at a time, so a batch behaves
/// exactly like the same tags typed one after another.
fn merge(held: &[String], pieces: Vec<String>, rules: &TagRules) -> Merged {
    let mut next = held.to_vec();
    let mut added = false;
    let mut refused = Vec::new();
    for piece in pieces {
        let tag = piece.trim().to_string();
        if tag.is_empty() {
            continue;
        }
        let folded = tag.to_lowercase();
        let reason = if !rules.allow_duplicates
            && next.iter().any(|held| held.trim().to_lowercase() == folded)
        {
            Some(TagRejectReason::Duplicate)
        } else if rules.max_tags.is_some_and(|max| next.len() >= max) {
            Some(TagRejectReason::Full)
        } else if rules.rule.is_some_and(|rule| !rule.call(tag.clone())) {
            Some(TagRejectReason::NotAllowed)
        } else {
            None
        };
        if let Some(reason) = reason {
            refused.push(TagRejection { tag, reason });
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
fn refusal_message(refused: &[TagRejection], words: &TagsFieldLabels) -> Option<String> {
    let sentences: Vec<String> = [
        (TagRejectReason::Duplicate, words.duplicate),
        (TagRejectReason::Full, words.full),
        (TagRejectReason::NotAllowed, words.not_allowed),
    ]
    .into_iter()
    .filter_map(|(reason, template)| {
        let labels: Vec<&str> = refused
            .iter()
            .filter(|rejection| rejection.reason == reason)
            .map(|rejection| rejection.tag.as_str())
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

/// The chips, one per held tag. A keyboard removal moves focus on before the
/// tag goes ([[principles/focus-after-removal]]).
fn tags_field_chips<F: Fn(Vec<String>) + Clone + 'static>(
    held: &[String],
    onchange: &Option<F>,
    draw_tag: Option<Callback<SelectionArgs<String>, Element>>,
    size: Size,
    (disabled, readonly): (bool, bool),
    focus: &ChipFocus,
) -> Element {
    let locked = disabled || readonly;
    let chips = held.iter().cloned().enumerate().map(|(index, value)| {
        let list = held.to_vec();
        let onchange = onchange.clone();
        let focus = focus.clone();
        // Guarded here as well as by disabling the x: a caller's own `tag`
        // gets `remove` too, and Backspace refuses a locked field.
        let remove = Callback::new(move |_: ()| {
            let (false, Some(onchange)) = (locked, &onchange) else {
                return;
            };
            // Only a focused tag moves the focus: a mouse removal never focused it.
            focus.before_removal(index);
            let mut next = list.clone();
            next.remove(index);
            onchange(next);
        });
        let chip = match &draw_tag {
            Some(tag) => tag.call(SelectionArgs {
                value,
                remove,
                disabled,
                readonly,
            }),
            None => removable_chip(value, remove, size, locked),
        };
        (chip, remove)
    });

    rsx! {
        for (index , (chip , remove)) in chips.enumerate() {
            span {
                key: "{index}",
                "data-slot": TagsFieldPart::Tag.slot(),
                "data-tag-index": "{index}",
                id: focus.id(index),
                // The default chip's guard, so a caller's `tag` has it too: a
                // press on its x must not strand the focus on the body.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                onfocusin: {
                    let focus = focus.clone();
                    move |_| focus.entered(index)
                },
                onfocusout: {
                    let focus = focus.clone();
                    move |_| focus.left(index)
                },
                onkeydown: {
                    let focus = focus.clone();
                    move |event: KeyboardEvent| {
                        // Tab, Escape and chords pass; every other key is the cursor's.
                        if !(is_tab(&event)
                            || matches!(logical_key(&event), Key::Named(NamedKey::Escape))
                            || !event.modifiers().is_empty())
                        {
                            event.stop_propagation();
                        }
                        focus.key(&event, index, remove);
                    }
                },
                {chip}
            }
        }
    }
}

/// The suggestion rows not yet held, narrowed by the draft. Drawn eagerly: a
/// `Callback` would let the rows memoize and leave stale ones on screen.
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

/// A hidden input per tag, as `MultiSelect` and `<select multiple>` post.
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
    input: ElementHandle,
    focus: ChipFocus,
    /// Set while ArrowLeft hands focus to a tag: a WebView's blur arrives later.
    entering: CopyValue<bool>,
    /// Set by a press on the clear x, whose blur drops the draft rather than committing it.
    clearing: CopyValue<bool>,
    item: Option<ToolbarItem>,
}

/// The draft input, which carries the field's id, label and description.
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
        input,
        focus,
        mut entering,
        mut clearing,
        item,
    } = draft;
    let (typing, committing, blurring) = (add.clone(), add.clone(), add);

    let mut attributes = match has_suggestions {
        true => state.a11y_attributes(),
        // Without a dropdown the input is a plain text input.
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
        // A toolbar keeps a disabled item focusable, in its arrow order: read-only instead.
        .attr("disabled", disabled && item.is_none())
        .attr(
            "aria-disabled",
            (disabled && item.is_some()).then_some("true"),
        )
        .attr("readonly", readonly || (disabled && item.is_some()))
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("tabindex", item.map(ToolbarItem::tabindex))
        .attr("required", (required && held.is_empty()).then_some(true))
        // A caller's token wins; the browser's saved entries then cover the suggestions.
        .attr_default("autocomplete", "off")
        .event("oninput", move |event: FormEvent| {
            let raw = event.value();
            // macOS Blitz fires one for a handled Backspace that deleted nothing (todo 3625).
            if raw == text() {
                return;
            }
            let pieces: Vec<String> = split(&raw, &split_chars);
            match pieces.is_empty() {
                // Only separators: keep them, or an already rendered `""` is not
                // written back and the comma stays on screen.
                true if raw != text() => text.set(raw),
                true => {}
                // A refused tag stays in the draft, to fix or drop.
                false => {
                    let rest = typing(pieces).unwrap_or_default();
                    // An unchanged draft is not written back, so a paste into it would stay (4189).
                    if rest == text() {
                        let _ = input.set_value(&rest);
                    }
                    text.set(rest);
                }
            }
            // Typing disarms the highlight: the next Enter belongs to the typed
            // text, not to a row that moved under it.
            state.set_active(None);
            if has_suggestions {
                state.open();
            }
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            // A composing key picks an IME candidate, it is not a command.
            if disabled || readonly || event.is_composing() {
                return;
            }
            // A key here means the last hand-off to a tag never left the input.
            entering.set(false);
            match logical_key(&event) {
                // A highlighted row owns Enter; with nothing typed it bubbles
                // and a form still submits.
                Key::Named(NamedKey::Enter)
                    if !(state.is_open() && state.active().is_some() && row_count > 0)
                        && !text().trim().is_empty() =>
                {
                    event.prevent_default();
                    if committing(vec![text()]).is_none() {
                        text.set(String::new());
                    }
                    state.set_active(None);
                }
                // The input edits the value, unlike `MultiSelect`'s search box;
                // with text in it, or with Ctrl or Alt held, Backspace only edits.
                Key::Named(NamedKey::Backspace)
                    if text().is_empty() && !held.is_empty() && event.modifiers().is_empty() =>
                {
                    let Some(onchange) = &onchange else {
                        return;
                    };
                    event.prevent_default();
                    let mut next = held.clone();
                    next.pop();
                    onchange(next);
                }
                // From an empty draft, or from the start of one, which stays.
                Key::Named(NamedKey::ArrowLeft)
                    // `caret_edges`, as a WebView's handle reads no selection.
                    if (text().is_empty() || caret_edges(&event).is_some_and(|(start, _)| start))
                        && !held.is_empty()
                        && event.modifiers().is_empty() =>
                {
                    event.prevent_default();
                    event.stop_propagation();
                    // Cleared by the blur, which a WebView delivers after this handler.
                    entering.set(true);
                    focus.focus(Some(held.len() - 1));
                }
                _ => {}
            }
        })
        // A hand-off or a press on the x that never blurred the input leaves no blur to reset them.
        .event("onfocus", move |_: FocusEvent| {
            entering.set(false);
            clearing.set(false);
        })
        // An uncommitted draft becomes a tag rather than being thrown away.
        .event("onblur", move |_: FocusEvent| {
            let draft = text();
            let kept = entering.replace(false) | clearing.replace(false);
            if !readonly && !kept && !draft.trim().is_empty() && blurring(vec![draft]).is_none() {
                text.set(String::new());
            }
            state.close();
        })
        .element(&input)
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
        );
        assert_eq!(merged.next, Some(tags(&["rust", "dioxus", "wasm"])));
        assert_eq!(merged.refused, [rejection("css", TagRejectReason::Full)]);
    }

    fn rejection(tag: &str, reason: TagRejectReason) -> TagRejection {
        TagRejection {
            tag: tag.to_string(),
            reason,
        }
    }

    /// Todo 1947: a duplicate allowed through is no rejection; a mixed batch
    /// carries each tag's own reason.
    #[test]
    fn each_rejection_carries_its_own_reason() {
        let merged = merge(
            &tags(&["rust"]),
            tags(&["Rust", "wasm", "css"]),
            &rules(false, Some(2)),
        );
        assert_eq!(
            merged.refused,
            [
                rejection("Rust", TagRejectReason::Duplicate),
                rejection("css", TagRejectReason::Full),
            ]
        );
        let allowed = merge(&tags(&["rust"]), tags(&["rust"]), &rules(true, Some(1)));
        assert_eq!(allowed.refused, [rejection("rust", TagRejectReason::Full)]);
    }

    /// Todo 545: a refusal says why, one sentence per reason.
    #[test]
    fn a_refusal_names_the_tags_and_the_reason() {
        let merged = merge(
            &tags(&["rust", "css"]),
            tags(&["Rust", "wasm", "CSS"]),
            &rules(false, Some(3)),
        );
        assert_eq!(
            refusal_message(&merged.refused, &TagsFieldLabels::ENGLISH),
            Some("Already added: Rust, CSS".to_string())
        );
        let full = [
            rejection("a", TagRejectReason::Full),
            rejection("b", TagRejectReason::Duplicate),
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
            merge(&tags(&["Rust"]), tags(&[" rust "]), &rules(false, None)).next,
            None
        );
        assert_eq!(
            merge(&tags(&[]), tags(&["a", "A"]), &rules(false, None)).next,
            Some(tags(&["a"]))
        );
        assert_eq!(
            merge(&tags(&["Rust"]), tags(&["rust"]), &rules(true, None)).next,
            Some(tags(&["Rust", "rust"]))
        );
    }

    /// Nothing added means nothing changed, so a refused edit never emits an
    /// `onchange` the caller would have to recognise as a no-op.
    #[test]
    fn an_edit_that_adds_nothing_reports_no_change() {
        assert_eq!(
            merge(&tags(&["a"]), tags(&["  "]), &rules(false, None)).next,
            None
        );
        assert_eq!(
            merge(&tags(&["a"]), tags(&["b"]), &rules(false, Some(1))).next,
            None
        );
    }
}
