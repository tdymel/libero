# PasswordField

Crate: `libero`
Import: `use libero::components::PasswordField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/password_field.rs>
Index: [index.md](index.md) lists every other page
Description: A `TextField` for secrets, with a button in its trailing slot that shows the text.

A [TextField](text_field.md) for secrets, with a button that shows the text.
The field keeps the reveal state itself, so a password never starts visible.
Every submit and reset of the surrounding `Form` hides the secret again.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::PasswordField;

#[component]
fn Demo() -> Element {
    let mut secret = use_signal(String::new);
    let mut repeat = use_signal(String::new);

    rsx! {
        PasswordField {
            label: "Password",
            helper: "At least 8 characters.",
            autocomplete: "new-password",
            value: secret(),
            oninput: move |next| secret.set(next),
        }
        PasswordField {
            label: "Repeat password",
            reveal_button: false,
            autocomplete: "new-password",
            value: repeat(),
            oninput: move |next| repeat.set(next),
        }
    }
}
```

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FieldPart::Label` | `label` | The label above the control. |
| `FieldPart::Required` | `required` | The required asterisk, in the label. |
| `FieldPart::Description` | `description` | The caption between the label and the control. |
| `FieldPart::Frame` | `frame` | The bordered box around the control. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Accessibility

### Libero handles

- The reveal button is a toggle with one name, so a screen reader hears it as
  pressed or not.

### You must

- Leave `label` unset only when something else names the field.
- Set `autocomplete` so password managers can fill the field: `"new-password"`
  on a sign-up form, `"current-password"` on a sign-in form.

## Props

### `PasswordField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The secret. Leave it out and the input keeps its own text. |
| `oninput` | `EventHandler<String>` | - | Fires on every keystroke with the text the field should hold next. |
| `validate` | `Validators<String>` | - | Rules over the secret, shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<String>` | - | What the field posts as. A path such as `Signup::FIELDS.password()` also binds the secret to the surrounding `Form`'s value when the field has no `oninput`. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `reveal_button` | `bool` | `theme.password_field.reveal_button` | Shows the reveal button. Turn it off for a confirmation field, which adds nothing beside a revealed twin. |
| `reveal_label` | `String` | `password_field.show` | The reveal button's name, such as "Show PIN". Unset, the localization's `password_field.show`, "Show password" in English. |
| `label` | `Caption` | - | The field's caption, above the control. It names the field. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. The password rules. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables and dims the field, reveal button included. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |

`PasswordField` also takes the `<input>` HTML attributes (`autocomplete`,
`maxlength`, ...) and, like every component, the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`PasswordFieldDefaults` holds `reveal_button`, the prop's default (`true`).
Everything else comes from the `TextField` it renders, so it reads
`FieldDefaults` and `TextFieldDefaults`, see [TextField](text_field.md).

## Data attributes

The same as [TextField](text_field.md). The reveal button's slot carries
`data-slot="trailing"`.
