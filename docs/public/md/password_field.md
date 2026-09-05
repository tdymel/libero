# PasswordField

Crate: `libero`
Import: `use libero::components::PasswordField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/password_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A password field - a `TextField` whose `type` flips between `password` and `text`, with the reveal toggle in its trailing slot.

A password field. It is a [TextField](text_field.md) with a narrower contract:
the same five slots, the same `value`/`oninput` pair, `type="password"`, and a
reveal button in the trailing slot that flips the type to `text`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::PasswordField;

#[component]
fn Demo() -> Element {
    let mut secret = use_signal(String::new);

    rsx! {
        PasswordField {
            label: "Password",
            helper: "At least 8 characters.",
            value: secret(),
            oninput: move |next| secret.set(next),
        }
    }
}
```

`value` and `oninput` behave exactly as they do on a `TextField`: `value: None`
leaves the `<input>` uncontrolled, and `oninput` fires per keystroke.

## Why it is a `TextField`, not its own field

A domain field is normally a text field with a narrower contract, so
`PasswordField` renders a `TextField` rather than rebuilding the chrome on
`use_field`. It costs one component scope and pays for it by inheriting
everything the text field grows.

That is the composition path to copy for an `EmailField`, a `SearchField` or any
other field the library does not ship.

## Revealing

The reveal state belongs to the component. There is no `revealed` prop and no
`onreveal`: a password field that starts visible is not a state a caller should
be able to ask for, and a caller that wants plain text wants a `TextField`.

`reveal_button` decides whether the button renders at all. It is on by default -
a password nobody can read back is this field's worst papercut - and worth
turning off for a confirmation field, which adds nothing beside a revealed twin:

```rust,ignore
PasswordField {
    label: "Repeat password",
    reveal_button: false,
    value: repeat(),
    oninput: move |next| repeat.set(next),
}
```

The button carries the two icons the library ships - libero has no icon set
otherwise, and a reveal button with no glyph is a blank button. It is named for
what the click does, not for the current state: `reveal_label` (default
`"Show password"`) while the secret is hidden, `hide_label` (default
`"Hide password"`) while it is shown. Both are props because neither string can
be localised from the outside otherwise.

The button is disabled with the field, so a disabled password cannot be read.

## Accessibility

Leave `label` unset only when something else already names the field.

## Props

### `PasswordField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The secret. `None` leaves the `<input>` uncontrolled. |
| `oninput` | `EventHandler<String>` | - | Fires per keystroke with the text the field should hold next. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `reveal_button` | `bool` | `true` | Offers the reveal button at all. |
| `reveal_label` | `String` | `Show password` | Announced on the reveal button while the secret is hidden. |
| `hide_label` | `String` | `Hide password` | Announced on the reveal button while the secret is shown. |
| `label` | `Caption` | - | The field's caption, above the control. Names the field through a `for`/`id` pair. |
| `description` | `Caption` | - | Between the label and the control: what to enter. |
| `helper` | `Caption` | - | Under the control: the password rules. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name` and `autocomplete` among them,
since the props extend `input`'s own. `autocomplete: "new-password"` on a
sign-up form and `"current-password"` on a sign-in form is worth setting.

## Theme defaults

None of its own. It renders a `TextField`, so it reads `FieldDefaults` and
`TextFieldDefaults` - see [TextField](text_field.md).

## Data attributes

The same as [TextField](text_field.md): state tokens on the wrapper's and the
frame's `data-state`, and `data-slot="trailing"` on the reveal button's slot
when `reveal_button` renders it.
