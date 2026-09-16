use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, VisuallyHidden,
        common::field_props,
        form::{FormScope, field_control_sx, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::{use_css, use_localization, use_theme},
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
});

field_props! {
    extends(textarea);
    pub struct TextareaProps {
        /// The text to render. `None` leaves the `<textarea>` uncontrolled -
        /// it keeps its own text and needs no handler.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the text the field should hold next.
        /// Native name, native timing.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the text, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<String>,
        /// What the field posts as. A path - `Signup::FIELDS.bio()` - also
        /// binds it to the surrounding `Form`'s value when the field has no
        /// `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Visible lines, which is what sets the starting height. The user can
        /// still drag it taller.
        #[props(default = 3)]
        rows: u32,
        /// Shows `12/200` under the control while a `maxlength` attribute is
        /// set, and politely announces the characters left once a tenth of
        /// the limit remains. Without `maxlength` it draws nothing.
        #[props(default)]
        counter: bool,
    }
}

/// The counter under the control, at its end.
static COUNTER_SX: StaticSx = StaticSx::new(|| sx().text_align("end"));

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

/// Whether `left` of `max` is near enough the limit to announce: the last
/// tenth, rounded up.
fn near_limit(left: usize, max: usize) -> bool {
    left <= max.div_ceil(10)
}

/// A multi-line text field, with a label, a description, helper text and a
/// validation message stacked around it.
///
/// `rows` sets the starting height and the browser's own drag handle takes it
/// from there. Controlled through `value` + `oninput`; omit `value` and the
/// `<textarea>` owns its own text.
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

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(bound.check(&props.validate, value.clone()))
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

    let frame = use_field_frame().states(field.states()).prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&TEXTAREA_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // Only a caller's text is known without the input events: an
    // uncontrolled textarea's length is tracked from them, stamped with the
    // form's reset count: a reset brings back the initial text, no input event.
    let limit = max_length(&props.attributes).filter(|_| props.counter);
    let resets = try_use_context::<FormScope>().map_or(0, |form| form.resets());
    let mut typed = use_signal(|| (0u32, None::<usize>));
    let counter_class = use_css(Some(&COUNTER_SX), CssLayer::Framework);
    let words = use_localization().textarea;
    let counter = limit.map(|max| {
        let (at, count) = typed();
        let typed_now = match (at == resets, count) {
            (true, Some(count)) => count,
            _ => initial_length(&props.attributes),
        };
        let used = value.as_deref().map_or(typed_now, length);
        let left = max.saturating_sub(used);
        let spoken = near_limit(left, max).then(|| (words.characters_left)(left));
        rsx! {
            div {
                class: counter_class,
                "data-slot": "counter",
                // The live region below says it in words.
                "aria-hidden": "true",
                "{used}/{max}"
            }
            VisuallyHidden { role: "status", {spoken} }
        }
    });

    let emit = bound.emit(props.oninput);
    let track = emit.is_some() || limit.is_some();
    let oninput = track.then_some(move |event: FormEvent| {
        let text = event.value();
        typed.set((resets, Some(length(&text))));
        if let Some(emit) = &emit {
            emit(text);
        }
    });
    let textarea = field
        .aria(control)
        .attr("name", bound.name().map(str::to_string))
        // A form reset leaves the text alone when this component sets it.
        .attr("data-controlled", value.is_some())
        .attr("value", value)
        .attr("rows", props.rows.to_string())
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .event("oninput", oninput)
        .render(HtmlTag::Textarea, props.attributes, ());

    field.render(rsx! {
        {frame.render(textarea)}
        {counter}
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
}
