# PinField

Crate: `libero`
Import: `use libero::components::PinField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/pin_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A pin, one character per cell, with auto-advance, paste spreading and an `oncomplete` that fires the moment the last cell fills.

A pin, one character per cell. Typing fills a cell and moves to the next,
Backspace clears and steps back, and the arrows move without changing anything.
A code pasted into any cell spreads across the rest, and characters the field
does not take are dropped, so `"4 2-1 3"` lands as `4213`. `oncomplete` fires
the moment the last cell fills, which is usually where you submit the code.
Extra HTML attributes land on the group, not on a cell.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::PinField;

#[component]
fn Demo() -> Element {
    let mut code = use_signal(String::new);

    rsx! {
        PinField {
            label: "Verification code",
            description: "Sent to your phone.",
            length: 6,
            separator: rsx! { "-" },
            value: code(),
            oninput: move |next| code.set(next),
            oncomplete: move |code: String| submit(code),
        }
    }
}
#
# fn submit(_code: String) {}
```

## Accessibility

Each cell is a tab stop. The arrows, Home and End move within the field, and
Tab leaves it as it leaves any group of inputs. Give it a `label`, which names
the whole group. Each cell is named for its place, such as "Character 1 of 6",
from the localization's `PinFieldLabels`, and reads the helper and the error
too.

## Props

### `PinField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | The cell's square, its font size and the gap. A cell is as tall as a `TextField` of the same size. |
| `radius` | `Size` | `sm` | Corner radius of each cell, independent of `size`. |
| `length` | `usize` | `4` | How many cells. |
| `kind` | `PinKind` | `numeric` | `numeric` or `alphanumeric`. Any other character is ignored as it is typed. |
| `value` | `Option<String>` | - | The pin so far, one character per filled cell. Leave it out and the field keeps its own pin. |
| `oninput` | `EventHandler<String>` | - | Fires for every accepted character with the pin the field should hold next. |
| `validate` | `Validators<String>` | - | Rules over the pin, shown once the field loses focus or its form is submitted. |
| `oncomplete` | `EventHandler<String>` | - | Fires once when the last empty cell fills. Clearing a cell arms it again. |
| `mask` | `bool` | `false` | Hides the characters as in a password field. The value is unaffected. |
| `one_time_code` | `bool` | `true` | Lets a phone offer the code it just received. |
| `separator` | `Element` | - | Rendered between the cells, such as a dash. |
| `name` | `FieldName<String>` | - | What the pin posts as. A path such as `Login::FIELDS.code()` also binds the pin to the surrounding `Form`'s value when the field has no `oninput`. |
| `autofocus` | `bool` | `false` | Focuses the first cell on mount. |
| `label` | `Caption` | - | The field's caption, above the cells. It names the group of cells. |
| `description` | `Caption` | - | Between the label and the cells. Where the code came from. |
| `helper` | `Caption` | - | Under the cells. How long the code lasts, how to get another. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables and dims every cell. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`PinFieldDefaults` holds `length`, `size`, `radius`, `kind` and `gap`, the
space between the cells, emitted as `--lsx-pin-field-gap`. Everything else is
`FieldDefaults`, so a pin field and a text field line up in one form.

## Data attributes

State tokens on the wrapper's and each cell frame's `data-state`: size, radius,
status, `disabled`, `required` and the `kind`. Each cell input carries
`data-pin-index`.
