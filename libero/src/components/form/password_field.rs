use dioxus::prelude::*;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{EyeIcon, EyeOffIcon, Input},
        form::{Disabled, FormScope, SLOT_BUTTON_SX, TextField, field_props, slot_icon_size},
    },
    hooks::{use_localization, use_theme},
    sx::ThemeAwareValue,
};

field_props! {
    extends(input);
    pub struct PasswordFieldProps {
        /// The secret. `None` leaves the `<input>` uncontrolled.
        #[props(default, into)]
        value: Option<String>,
        /// Fires per keystroke with the text the field should hold next.
        #[props(default)]
        oninput: Option<EventHandler<String>>,
        /// Rules over the secret, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<String>,
        /// What the field posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Offers the reveal button. On by default.
        #[props(default)]
        reveal_button: Option<bool>,
        /// The reveal button's name. Defaults to `password_field.show`.
        #[props(default, into)]
        reveal_label: Option<String>,
    }
}

/// A [`TextField`] for a secret, with a reveal toggle in its trailing slot.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::PasswordField;
/// # fn app() -> Element {
/// let mut password = use_signal(String::new);
/// rsx! {
///     PasswordField {
///         label: "Password",
///         value: password(),
///         oninput: move |text| password.set(text),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/password-field>
#[component]
pub fn PasswordField(props: PasswordFieldProps) -> Element {
    // Revealed at a count of the form's submits and resets, so the next one
    // hides it again (todo 497).
    let form = try_use_context::<FormScope>();
    let settled = form.map_or(0, |form| form.settled());
    let mut revealed_at = use_signal(|| None::<u32>);
    let revealed = move || revealed_at() == Some(settled);
    let theme = use_theme();
    let reveal_button = props
        .reveal_button
        .unwrap_or(theme.password_field.reveal_button);

    // A disabled `Fieldset` disables the input, so the toggle follows it.
    let group = try_use_context::<Disabled>();
    let disabled = props.disabled.unwrap_or(false) || group.is_some_and(|Disabled(group)| group());
    // One static name plus `aria-pressed`: a name that flips is announced as a
    // different button, not as a state change (todo 498).
    let labels = use_localization().password_field;
    let aria_label = props
        .reveal_label
        .clone()
        .unwrap_or_else(|| labels.show.to_string());
    // Sized off the field's own size, theme default included, as the clear button is.
    let size: Input<ThemeAwareValue> =
        ThemeAwareValue::Size(slot_icon_size(props.size.copied_or(theme.text_field.size))).into();

    rsx! {
        TextField {
            r#type: if revealed() { "text" } else { "password" },
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            validate: props.validate,
            size: props.size,
            radius: props.radius,
            disabled: props.disabled,
            required: props.required,
            readonly: props.readonly,
            name: props.name,
            value: props.value,
            oninput: props.oninput,
            placeholder: props.placeholder,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
            trailing: reveal_button.then(|| rsx! {
                ActionIcon {
                    aria_label,
                    "aria-pressed": revealed().to_string(),
                    size,
                    sx: &SLOT_BUTTON_SX,
                    disabled,
                    onclick: move |_| revealed_at.set((!revealed()).then_some(settled)),
                    // A struck-through eye while the secret is readable.
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
