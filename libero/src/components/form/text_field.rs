use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, ScaleOrCss, TOOLBAR_ITEM, ToolbarItem, use_no_toolbar, use_toolbar_item,
        },
        form::{
            FIELD_CONTROL_SX, LiveControl, LiveSlot, field_props, use_bound, use_field,
            use_field_frame, use_live_slot,
        },
        layout::use_box,
    },
    hooks::use_theme,
    platform::fit_max_length,
};

field_props! {
    extends(input);
    pub struct TextFieldProps {
        /// The text. `None` leaves the `<input>` uncontrolled.
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
        /// Inside the frame, before the control.
        #[props(default, into)]
        leading: Option<Element>,
        /// Inside the frame, after the control.
        #[props(default, into)]
        trailing: Option<Element>,
        /// `leading` is text that describes the value (`aria-describedby`).
        #[props(default)]
        describe_leading: bool,
        /// `trailing` is text that describes the value, e.g. a unit.
        #[props(default)]
        describe_trailing: bool,
    }
}

/// A single-line text field with its label, captions and validation message.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::TextField;
/// # fn app() -> Element {
/// let mut name = use_signal(String::new);
/// rsx! {
///     TextField {
///         label: "Name",
///         value: name(),
///         oninput: move |text| name.set(text),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/text-field>
#[component]
pub fn TextField(props: TextFieldProps) -> Element {
    let mut props = props;
    let value = props.value.take();
    let mut live = use_signal(|| value.clone());
    if *live.peek() != value {
        live.set(value);
    }
    let leading = use_live_slot(props.leading.take());
    let trailing = use_live_slot(props.trailing.take());
    rsx! { TextFieldShell { live, leading, trailing, field: props } }
}

/// Everything but the text and the slots' content, so a keystroke or a
/// caller's new slot `Element` skips it.
#[component]
fn TextFieldShell(
    live: Signal<Option<String>>,
    leading: Option<Signal<Option<Element>>>,
    trailing: Option<Signal<Option<Element>>>,
    field: TextFieldProps,
) -> Element {
    let props = field;
    let leading = leading.map(|content| rsx! { LiveSlot { content } });
    let trailing = trailing.map(|content| rsx! { LiveSlot { content } });
    let theme = use_theme();

    let size = props.size.copied_or(theme.text_field.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.text_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let item = use_toolbar_item();
    // Buttons in the slots belong to the field, not the bar.
    use_no_toolbar();
    let bound_value = bound.value();
    // Rules read the text here, so only a validated field redraws per keystroke.
    let rules = (!props.validate.is_empty())
        .then(|| {
            bound.check(
                &props.validate,
                bound_value.clone().or_else(|| live.cloned()),
            )
        })
        .flatten();
    let empty = bound.is_empty(
        required,
        || bound_value.clone().or_else(|| live.cloned()),
        String::is_empty,
    );

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(rules)
        .bound(&bound)
        .required(required)
        .empty(empty)
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .text_slots(
            props.describe_leading && leading.is_some(),
            props.describe_trailing && trailing.is_some(),
        )
        .prepare();

    let frame = use_field_frame()
        .leading(&leading)
        .trailing(&trailing)
        .states(field.states())
        .ids(field.slot_ids())
        .placeholder(props.placeholder.as_deref())
        .prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let oninput = bound.emit(props.oninput);
    let input = field
        .aria(control)
        .attr_default("type", "text")
        .attr("name", bound.name().map(str::to_string))
        .attr("placeholder", props.placeholder)
        // A toolbar keeps a disabled item focusable, in its arrow order: read-only instead.
        .attr("disabled", disabled && item.is_none())
        .attr(
            "aria-disabled",
            (disabled && item.is_some()).then_some("true"),
        )
        .attr("readonly", readonly || (disabled && item.is_some()))
        .attr("required", required)
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("tabindex", item.map(ToolbarItem::tabindex))
        .event(
            "oninput",
            oninput.map(|emit| move |event: FormEvent| emit(fit_max_length(event.value()))),
        );
    let attributes = props.attributes;
    let draw = Rc::new(move || {
        let value = bound_value.clone().or_else(|| live.cloned());
        input
            .clone()
            // A form reset leaves the text alone when this component sets it.
            .attr("data-controlled", value.is_some())
            .attr("value", value)
            // Void element - `()` costs no dynamic node.
            .render(HtmlTag::Input, attributes.clone(), ())
    });

    field.render(frame.render(rsx! { LiveControl { draw } }))
}
