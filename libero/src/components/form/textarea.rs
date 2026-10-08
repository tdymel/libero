use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        accessibility::VisuallyHidden,
        common::{HtmlTag, Input, Part},
        form::{
            FormScope, field_control_sx, field_parts_enum, field_props, use_bound, use_field,
            use_field_frame,
        },
        layout::use_box,
    },
    hooks::{
        use_css, use_debounced_value, use_element, use_form_owner, use_localization, use_theme,
    },
    platform::fit_max_length,
    sx::{StaticSx, sx},
};

/// The control's own additions to [`field_control_sx`]: a `<textarea>` is not
/// inline, and it keeps the drag handle the browser gives it for free.
static TEXTAREA_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("block")
        .resize("vertical")
        // The frame's `min-height` is a single line's floor; `rows` is what
        // decides the box, so the control must not inherit that floor.
        .min_height("0")
        // A strip under the control for the counter, so scrolled text never runs under it.
        .selector("&[data-counter]", sx().margin_bottom("1.25rem"))
});

field_parts_enum! {
    /// [`Textarea`]'s inner parts, for its `parts` prop: a field's, and the counter.
    pub enum TextareaPart framed {
        /// The `12/200` badge in the frame's corner, with `counter`. Beside the
        /// control, so one level deeper too where the renderer draws no placeholder.
        Counter = "counter" => "& > [data-slot='frame'] > [data-slot='counter'], & > [data-slot='frame'] > * > [data-slot='counter']",
    }
}

field_props! {
    parts(TextareaPart);
    extends(textarea);
    pub struct TextareaProps {
        /// The text. `None` leaves the `<textarea>` uncontrolled.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the text the field should hold next.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the text, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<String>,
        /// What the field posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Visible lines, which set the starting height.
        #[props(default = 3)]
        rows: u32,
        /// Shows `12/200` in the control's corner while a `maxlength` attribute is set.
        #[props(default)]
        counter: bool,
    }
}

/// The counter in the frame's bottom end corner (bottom left in RTL).
static COUNTER_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .bottom("0.25rem")
        .inset_inline_end("0.5rem")
        .font_size("0.75rem")
        .line_height("1")
        .color("text-dimmed")
        .pointer_events("none")
});

/// The `maxlength` a caller passed among the extra attributes.
fn max_length(attributes: &[Attribute]) -> Option<usize> {
    let attribute = attributes
        .iter()
        .find(|attribute| attribute.name == "maxlength")?;
    match &attribute.value {
        AttributeValue::Text(text) => text.trim().parse().ok(),
        AttributeValue::Int(max) => usize::try_from(*max).ok(),
        _ => None,
    }
}

/// The length of the text an uncontrolled textarea starts with, and a reset
/// brings back.
fn initial_length(attributes: &[Attribute]) -> usize {
    attributes
        .iter()
        .find(|attribute| attribute.name == "initial_value")
        .map_or(0, |attribute| match &attribute.value {
            AttributeValue::Text(text) => length(text),
            _ => 0,
        })
}

/// Length as `maxlength` counts it: UTF-16 code units.
fn length(text: &str) -> usize {
    text.encode_utf16().count()
}

/// How long typing pauses before the counter's status speaks.
const SETTLE_MS: u64 = 1000;

/// Whether `left` of `max` is near enough the limit to announce: the last
/// tenth, rounded up.
fn near_limit(left: usize, max: usize) -> bool {
    left <= max.div_ceil(10)
}

/// A multi-line text field with its label, captions and validation message.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Textarea;
/// # fn app() -> Element {
/// let mut bio = use_signal(String::new);
/// rsx! {
///     Textarea {
///         label: "Bio",
///         rows: 4,
///         value: bio(),
///         oninput: move |text| bio.set(text),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/textarea>
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.textarea.size);
    let radius = props.radius.copied_or(theme.textarea.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let value = bound.value().or_else(|| props.value.clone());

    let mut field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(bound.check(&props.validate, value.clone()))
        .bound(&bound)
        .required(required)
        .empty(bound.is_empty(required, || value.clone(), String::is_empty))
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let frame = use_field_frame()
        .states(field.states())
        .placeholder(props.placeholder.as_deref())
        .multiline()
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&TEXTAREA_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // An uncontrolled length is tracked from input events, stamped with the
    // form's reset count; a raw `<form>` reset is heard off the DOM (todo 685).
    let limit = max_length(&props.attributes).filter(|_| props.counter);
    let scope = try_use_context::<FormScope>();
    let element = use_element();
    let owner = use_form_owner(
        element,
        scope.is_none() && limit.is_some() && value.is_none(),
    );
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    let mut typed = use_signal(|| (0u32, None::<usize>));
    let counter_class = use_css(Some(&COUNTER_SX), CssLayer::Framework);
    let words = use_localization().textarea;
    let count = limit.map(|max| {
        let (at, count) = typed();
        let typed_now = match (at == resets, count) {
            (true, Some(count)) => count,
            _ => initial_length(&props.attributes),
        };
        let used = value.as_deref().map_or(typed_now, length);
        (used, max, max.saturating_sub(used))
    });
    // Only a controlled value runs past `maxlength`; "0 left" would hide it.
    let over = count
        .filter(|&(used, max, _)| used > max)
        .map(|(used, max, _)| (words.characters_over)(used - max));
    // Said once typing stops, not per keystroke (GOV.UK's character count).
    let spoken = over.clone().or_else(|| {
        count
            .filter(|&(_, max, left)| near_limit(left, max))
            .map(|(_, _, left)| (words.characters_left)(left))
    });
    let mut typing = use_signal(|| spoken.clone());
    if *typing.peek() != spoken {
        typing.set(spoken);
    }
    let settled = use_debounced_value(typing.into(), SETTLE_MS);
    let count_id = format!("{}-count", field.id());
    if count.is_some() {
        // Read with the field at focus, so the limit is known before typing.
        field.describe_also(count_id.clone());
    }
    let counter = count.map(|(used, max, left)| {
        let badge = rsx! {
            div {
                class: counter_class,
                "data-slot": TextareaPart::Counter.slot(),
                // The live region beside the frame says it in words.
                "aria-hidden": "true",
                "{used}/{max}"
            }
        };
        let status = rsx! {
            // `hidden`: a description only, not a second line in browse mode.
            span { id: count_id, hidden: true, {over.unwrap_or_else(|| (words.characters_left)(left))} }
            VisuallyHidden { role: "status", {settled()} }
        };
        (badge, status)
    });
    let (badge, status) = match counter {
        Some((badge, status)) => (Some(badge), Some(status)),
        None => (None, None),
    };

    let emit = bound.emit(props.oninput);
    let track = emit.is_some() || limit.is_some();
    let oninput = track.then_some(move |event: FormEvent| {
        let text = fit_max_length(event.value());
        typed.set((resets, Some(length(&text))));
        if let Some(emit) = &emit {
            emit(text);
        }
    });
    let textarea = field
        .aria(control)
        .element(&element)
        .attr("name", bound.name().map(str::to_string))
        // A form reset leaves the text alone when this component sets it.
        .attr("data-controlled", value.is_some())
        .attr("data-counter", limit.is_some())
        .attr("value", value)
        .attr("rows", props.rows.to_string())
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .event("oninput", oninput)
        .render(HtmlTag::Textarea, props.attributes, ());

    field.render(rsx! {
        {frame.render(rsx! { {textarea} {badge} })}
        {status}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LiberoProvider;

    #[test]
    fn the_last_tenth_is_near_the_limit() {
        assert!(!near_limit(21, 200));
        assert!(near_limit(20, 200));
        // Rounded up: 3 of 25 is near.
        assert!(near_limit(3, 25));
        assert!(!near_limit(4, 25));
    }

    #[test]
    fn length_counts_as_maxlength_does() {
        // One UTF-16 unit for an accent, two for an emoji.
        assert_eq!(length("é"), 1);
        assert_eq!(length("🙂"), 2);
    }

    #[test]
    fn an_uncontrolled_count_starts_at_the_initial_text() {
        let html = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                Textarea { counter: true, maxlength: 20, initial_value: "hi" }
            }
        });
        assert!(html.contains(">2/20<"), "{html}");
    }

    #[test]
    fn a_counter_needs_a_maxlength() {
        let with = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                Textarea { counter: true, maxlength: 20, value: "hello" }
            }
        });
        assert!(with.contains(">5/20<"), "{with}");
        let without = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                Textarea { counter: true, value: "hello" }
            }
        });
        assert!(!without.contains("data-slot=\"counter\""), "{without}");
    }

    #[test]
    fn a_value_over_the_limit_says_by_how_many() {
        let html = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                Textarea { counter: true, maxlength: 3, value: "hello" }
            }
        });
        assert!(html.contains(">5/3<"), "{html}");
        assert!(html.contains("2 characters too many"), "{html}");
        assert!(!html.contains("characters left"), "{html}");
    }

    /// Todo 2460: the count is part of the field's description, read at focus.
    #[test]
    fn the_count_describes_the_field() {
        let html = dioxus_ssr::render_element(rsx! {
            LiberoProvider {
                Textarea { id: "bio", counter: true, maxlength: 20, value: "hello" }
            }
        });
        assert!(html.contains("id=\"bio-count\""), "{html}");
        assert!(html.contains(">15 characters left<"), "{html}");
        let described = html
            .split("aria-describedby=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next());
        assert_eq!(described, Some("bio-count"), "{html}");
    }
}
