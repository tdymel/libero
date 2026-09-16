# PinField

Crate: `libero`
Import: `use libero::components::PinField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/pin_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A pin, one character per cell, with auto-advance, paste spreading and an `oncomplete` that fires the moment the last cell fills.

A verification code, a PIN, a backup code: one character per cell, and one
value. The cells are ordinary `<input>`s inside a `role="group"`, so the field
carries the same label, description, helper and status slots every other field
has.

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
            value: code(),
            oninput: move |next| code.set(next),
            oncomplete: move |code: String| submit(code),
        }
    }
}
#
# fn submit(_code: String) {}
```

`value` is the pin as one string, one character per filled cell. `None` leaves
the cells to the field's own buffer - unlike a `TextField`, the field always
keeps one, because auto-advance has to know which cell just filled.

Holes are not representable. The value is the filled cells joined, so typing
into a cell past the end of the value lands in the first empty one instead - and
clearing a cell in the middle pulls the characters after it one cell to the
left, rather than leaving a gap in the code.

## `oncomplete`

`oncomplete` fires once, the moment the last empty cell fills, with the whole
pin. It is latched: typing over a full pin does not fire it again, and clearing
a cell arms it for the next fill. That is the difference between this and `N`
text fields - a code is submitted when it is complete, not when a button is
pressed.

`oninput` still fires for every accepted character, so a field that only cares
about completion can ignore it.

## Typing, and pasting

- A character fills the focused cell and moves to the next.
- Typing over a filled cell replaces its character.
- Backspace clears the cell and steps back; in an empty cell it only steps back.
  In the last cell it clears without moving, because that is where the next
  character goes.
- Delete clears the focused cell without moving.
- The arrows, Home and End move without changing anything. Space moves on.
- Retyping the character a cell already holds moves on rather than rewriting it.

Pasting a whole code into any cell spreads it across that cell and the ones
after it. Characters the field does not accept are dropped rather than
rejecting the paste, so `"4 2-1 3"` lands as `4213` - a code copied out of an
email arrives with its formatting, and that formatting is never part of the pin.

There is no `maxlength` on a cell: the spread is driven by what the input
event carries, which is the one mechanism that serves typing and pasting alike,
in a browser and under Blitz.

## `kind`

`numeric` (the default) accepts digits; `alphanumeric` accepts ASCII letters and
digits. A rejected character is dropped at the key, so it never appears in a
cell and is never taken back out again.

`numeric` also sets `inputmode="numeric"` and `type="tel"` on the cells - a
numeric keypad on a phone, and no spinner, which `type="number"` would add.

## Posting with a form

`name` emits a hidden input carrying the whole pin, so the field posts in a
plain `<form>`. The cells cannot carry the name themselves: there are several of
them and each holds one character.

```rust,ignore
PinField { label: "Code", name: "otp", length: 6 }
```

## Focus

The focused cell gets the field's own focus treatment - the frame's border turns
primary, and a keyboard focus adds the ring every component draws. There is no
`color` prop: the tint would apply to nothing but the focused cell, which is the
one place a field should look like every other field.

## Accessibility

Each cell is a tab stop; the arrows move within the field, and Tab leaves it
the way it leaves any group of inputs. Give it a `label`: it names the whole
group. Each cell is named for its place, "Character 1 of 6", from the
localization's `PinFieldLabels`, and every cell points `aria-describedby` at
the helper and the error, so a focused cell hears them too.

## Props

### `PinField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | The cell's square, its font size and the gap. A cell is as tall as a `TextField` at the same size. |
| `radius` | `Size` | `sm` | Corner radius of each cell, independent of `size`. |
| `length` | `usize` | `4` | How many cells. |
| `kind` | `PinKind` | `numeric` | `numeric` or `alphanumeric`. |
| `value` | `Option<String>` | - | The pin so far, one character per filled cell. |
| `oninput` | `EventHandler<String>` | - | Fires per accepted character with the pin the field should hold next. |
| `oncomplete` | `EventHandler<String>` | - | Fires once when the last empty cell fills. |
| `mask` | `bool` | `false` | Renders the cells as password inputs. The value is unaffected. |
| `one_time_code` | `bool` | `true` | `autocomplete="one-time-code"` on the first cell. |
| `separator` | `Element` | - | Rendered between the cells. |
| `name` | `String` | - | Emits a hidden input of that name, so the pin posts with a form. |
| `autofocus` | `bool` | `false` | Focuses the first cell on mount. |
| `label` | `Caption` | - | The field's caption, above the cells. Names the group through `aria-labelledby`. |
| `description` | `Caption` | - | Between the label and the cells. |
| `helper` | `Caption` | - | Under the cells. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` on the group and marks the label. |
| `disabled` | `bool` | `false` | Disables every cell and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. Unlike the other fields, those
attributes land on the group, not on an input - the field owns several, so
`autofocus` and `name` are props of its own instead.

## Theme defaults

`PinFieldDefaults`: `length`, `size`, `radius`, `kind`, and `gap` - the space
between the cells, emitted as `--lsx-pin-field-gap`. Everything else is
`FieldDefaults`, which is why a pin field and a text field line up in one form.

## Data attributes

State tokens on the wrapper's and each cell frame's `data-state` - size, radius,
status, `disabled`, `required`, and the `kind`. Each cell input carries
`data-pin-index`, which is how the field finds the cell to focus next.
