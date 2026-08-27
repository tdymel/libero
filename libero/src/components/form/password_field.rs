use dioxus::prelude::*;

use crate::{
    components::{ActionIcon, Input, common::field_props, form::TextField},
    sx::ThemeAwareValue,
};

field_props! {
    extends(input);
    pub struct PasswordFieldProps {
        /// The secret. `None` leaves the `<input>` uncontrolled - it keeps its
        /// own text and needs no handler.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the text the field should hold next.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Offers the reveal button at all. On by default - a password nobody
        /// can read back is the field's worst papercut - but a confirmation
        /// field, or one next to a revealed twin, has nothing to add.
        #[props(default = true)]
        reveal_button: bool,
        /// Announced on the reveal button while the secret is hidden.
        #[props(default, into)]
        reveal_label: Option<String>,
        /// Announced on the reveal button while the secret is shown.
        #[props(default, into)]
        hide_label: Option<String>,
    }
}

/// A password field: a [`TextField`] whose `type` flips between `password` and
/// `text`, with the reveal toggle in its trailing slot.
///
/// It is a `TextField` rather than its own field, which is the composition the
/// field foundation is built for - a domain field is a text field with a
/// narrower contract. Everything `TextField` grows, this grows too.
///
/// Reveal state is the component's own: a password that starts visible is not
/// a state a caller should be able to ask for.
#[component]
pub fn PasswordField(props: PasswordFieldProps) -> Element {
    let mut revealed = use_signal(|| false);

    let disabled = props.disabled.unwrap_or(false);
    let hidden_label = props
        .reveal_label
        .clone()
        .unwrap_or_else(|| "Show password".to_string());
    let shown_label = props
        .hide_label
        .clone()
        .unwrap_or_else(|| "Hide password".to_string());

    // The icon shows what the click does, so it is the *opposite* of the
    // current state: a struck-through eye while the secret is readable.
    let aria_label = match revealed() {
        true => shown_label,
        false => hidden_label,
    };
    let size: Input<ThemeAwareValue> = props
        .size
        .as_ref()
        .map(|size| ThemeAwareValue::Size(*size))
        .into();

    rsx! {
        TextField {
            r#type: if revealed() { "text" } else { "password" },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size,
            radius: props.radius,
            disabled: props.disabled,
            required: props.required,
            value: props.value,
            oninput: props.oninput,
            placeholder: props.placeholder,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
            trailing: props.reveal_button.then(|| rsx! {
                ActionIcon {
                    aria_label,
                    size,
                    disabled,
                    onclick: move |_| revealed.toggle(),
                    if revealed() {
                        EyeOffIcon {}
                    } else {
                        EyeIcon {}
                    }
                }
            }),
        }
    }
}

/// libero ships no icon set, so the two icons this component cannot do without
/// live here rather than in a caller's hands - a reveal button with no default
/// glyph is a blank button.
#[component]
fn EyeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

#[component]
fn EyeOffIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M2 12s3.5-7 10-7c2 0 3.8.7 5.2 1.6" }
            path { d: "M21.5 10.4c.3.6.5 1.1.5 1.6 0 0-3.5 7-10 7-1.3 0-2.5-.3-3.5-.7" }
            path { d: "M9.9 9.9a3 3 0 0 0 4.2 4.2" }
            path { d: "M3 3l18 18" }
        }
    }
}
