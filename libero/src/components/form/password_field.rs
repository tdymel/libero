use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input},
        form::{
            FormScope, SLOT_BUTTON_SX, TextField, field_props, slot_icon_size,
            use_fieldset_disabled,
        },
    },
    context::IconSlot,
    hooks::{listener, use_element, use_form_settles, use_localization, use_theme},
    sx::ThemeAwareValue,
    theme::Responsive,
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

/// Keeps the secret from a browser's spellcheck, even revealed; a caller's attribute wins.
fn private_attributes(mut attributes: Vec<Attribute>) -> Vec<Attribute> {
    for (name, value) in [
        ("spellcheck", "false"),
        ("autocorrect", "off"),
        ("autocapitalize", "off"),
    ] {
        if attributes.iter().all(|attribute| attribute.name != name) {
            attributes.push(Attribute::new(name, value, None, false));
        }
    }
    attributes
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
    let theme = use_theme();
    let reveal_button = props
        .reveal_button
        .unwrap_or(theme.password_field.reveal_button);
    // Revealed at a count of the form's submits and resets, so the next one hides it again
    // (todo 497); outside a libero `Form`, the raw `<form>`'s (4295, 4331).
    let form = try_use_context::<FormScope>();
    let button = use_element();
    let owner = use_form_settles(button, form.is_none() && reveal_button);
    let settled = form.map_or(0, |form| form.settled()) + owner().unwrap_or(0);
    let mut revealed_at = use_signal(|| None::<u32>);
    // Shown only while its button can hide it; once it cannot, the reveal is over (5321).
    let in_disabled_group = use_fieldset_disabled();
    let hideable = reveal_button && !props.disabled.unwrap_or(false) && !in_disabled_group;
    let revealed = move || hideable && revealed_at() == Some(settled);
    use_effect(use_reactive!(|hideable| {
        if !hideable {
            revealed_at.set(None);
        }
    }));

    // One static name plus `aria-pressed`: a name that flips is announced as a
    // different button, not as a state change (todo 498).
    let labels = use_localization().password_field;
    let aria_label = props
        .reveal_label
        .clone()
        .unwrap_or_else(|| labels.show.to_string());
    // Sized off the field's own size, theme default included, as the clear button is.
    let size: Input<Responsive<ThemeAwareValue>> = props
        .size
        .copied_or(theme.text_field.size.into())
        .map(|size| ThemeAwareValue::Size(slot_icon_size(size)))
        .into();

    rsx! {
        TextField {
            r#type: if revealed() { "text" } else { "password" },
            label: props.label,
            label_trailing: props.label_trailing,
            description: props.description,
            description_position: props.description_position,
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
            parts: props.parts,
            states: props.states,
            hidden_from: props.hidden_from,
            visible_from: props.visible_from,
            attributes: private_attributes(props.attributes),
            trailing: reveal_button.then(|| rsx! {
                ActionIcon {
                    aria_label,
                    "aria-pressed": revealed().to_string(),
                    size,
                    sx: &SLOT_BUTTON_SX,
                    // `ActionIcon` follows a disabled `Fieldset` itself.
                    disabled: props.disabled,
                    attributes: vec![
                        listener("onmounted", button.mount()),
                        // A press keeps focus in the input, so a phone keyboard stays open.
                        listener("onpointerdown", |event: Event<PointerData>| {
                            event.prevent_default()
                        }),
                    ],
                    onclick: move |_| revealed_at.set((!revealed()).then_some(settled)),
                    // A struck-through eye while the secret is readable.
                    if revealed() {
                        Glyph { slot: IconSlot::EyeOff, icon: lucide::eye_off::outlined }
                    } else {
                        Glyph { slot: IconSlot::Eye, icon: lucide::eye::outlined }
                    }
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::AttributeValue;

    use super::*;
    use crate::LiberoProvider;

    fn html(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn texts(attributes: &[Attribute], name: &str) -> Vec<String> {
        attributes
            .iter()
            .filter(|attribute| attribute.name == name)
            .map(|attribute| match &attribute.value {
                AttributeValue::Text(text) => text.clone(),
                other => panic!("{other:?}"),
            })
            .collect()
    }

    /// Todo 4294: a cloud spellcheck must not see the secret, hidden or revealed.
    #[test]
    fn the_hidden_input_turns_off_spellcheck_correction_and_capitals() {
        let page = html(|| rsx! { LiberoProvider { PasswordField { label: "Secret" } } });
        assert!(page.contains("type=\"password\""), "{page}");
        assert!(page.contains("spellcheck=\"false\""), "{page}");
        assert!(page.contains("autocorrect=\"off\""), "{page}");
        assert!(page.contains("autocapitalize=\"off\""), "{page}");
    }

    /// The reveal only flips `type`; the attributes it renders are these either way.
    #[test]
    fn the_defaults_are_set_once_and_do_not_depend_on_the_reveal() {
        let attributes = private_attributes(Vec::new());
        assert_eq!(texts(&attributes, "spellcheck"), ["false"]);
        assert_eq!(texts(&attributes, "autocorrect"), ["off"]);
        assert_eq!(texts(&attributes, "autocapitalize"), ["off"]);
    }

    /// Todo 5419: the slot sits beside the `<label>`, in a row, never inside it.
    #[test]
    fn label_trailing_renders_after_the_label_outside_it() {
        let page = html(|| {
            rsx! {
                LiberoProvider {
                    PasswordField { label: "Secret", label_trailing: rsx! { button { "Forgot" } } }
                }
            }
        });
        let row = page.find("data-slot=\"label-row\"").expect(&page);
        let label_end = page.find("</label>").expect(&page);
        let button = page.find("<button>Forgot").expect(&page);
        assert!(row < label_end && label_end < button, "{page}");
        assert!(page.contains("data-slot=\"label-trailing\""), "{page}");

        let plain = html(|| rsx! { LiberoProvider { PasswordField { label: "Secret" } } });
        assert!(!plain.contains("data-slot=\"label-row\""), "{plain}");
    }

    #[test]
    fn a_caller_attribute_replaces_its_default() {
        let caller = vec![
            Attribute::new("spellcheck", "true", None, false),
            Attribute::new("autocapitalize", "words", None, false),
        ];
        let attributes = private_attributes(caller);
        assert_eq!(texts(&attributes, "spellcheck"), ["true"]);
        assert_eq!(texts(&attributes, "autocapitalize"), ["words"]);
        assert_eq!(texts(&attributes, "autocorrect"), ["off"]);

        let page = html(|| {
            rsx! { LiberoProvider { PasswordField { label: "Secret", "spellcheck": "true" } } }
        });
        assert!(page.contains("spellcheck=\"true\""), "{page}");
        assert!(!page.contains("spellcheck=\"false\""), "{page}");
    }
}
