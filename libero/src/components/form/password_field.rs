use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Input,
        common::{EyeIcon, EyeOffIcon, field_props},
        form::{Disabled, FormScope, TextField, slot_icon_size},
    },
    hooks::use_theme,
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
        /// Rules over the secret, shown once the field loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<String>,
        /// What the field posts as. A path - `Signup::FIELDS.password()` - also
        /// binds it to the surrounding `Form`'s value when the field has no
        /// `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<String>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Offers the reveal button at all. On by default - a password nobody
        /// can read back is the field's worst papercut - but a confirmation
        /// field, or one next to a revealed twin, has nothing to add.
        #[props(default)]
        reveal_button: Option<bool>,
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
/// a state a caller should be able to ask for. A submit or a reset of the
/// surrounding `Form` hides the secret again.
#[component]
pub fn PasswordField(props: PasswordFieldProps) -> Element {
    // Revealed at a count of the form's submits and resets, so the next one
    // hides the secret again (todo 497).
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
                    size,
                    disabled,
                    onclick: move |_| revealed_at.set((!revealed()).then_some(settled)),
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
