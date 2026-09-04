# PhoneField

Crate: `libero`
Import: `use libero::components::PhoneField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/phone_field/field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A phone field - a country picker in front of a `tel` input, whose value is an E.164 string.

A phone number field. The picker in the leading slot carries the country and its
dial code, the `<input type="tel">` beside it holds the national number, and the
value is one E.164 string - `"+12133734253"`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::PhoneField;

#[component]
fn Demo() -> Element {
    let mut phone = use_signal(String::new);

    rsx! {
        PhoneField {
            label: "Mobile",
            country: "DE",
            value: phone(),
            oninput: move |next| phone.set(next),
        }
    }
}
```

`value` is always E.164 and never the text on screen. `oninput` fires per
keystroke with the E.164 the field should hold next, and with an empty string
once nothing is typed - an empty field is worth no value at all, not a bare dial
code. `value: None` leaves the field uncontrolled.

## Two values, which is why this is a component

The text being edited and the value are different strings: the user types
`213 373 4253` and the caller holds `+12133734253`. That is
[NumberField](number_field.md)'s edit buffer in another shape, and it is why
"a `TextField` with a `leading` the caller fills" is not the answer.

The text on screen is a rendering of the value for as long as the two still
agree; the moment they do not, the caller's value wins. So a `value` that
arrives under another country's dial code moves the picker to that country and
re-renders the text as its national part.

## It ships no validation

The library ships the country list and the dial codes and nothing else. Nothing
here knows that `+1 555` is too short, and there is no numbering-plan data in
the bundle - `phonenumber` would drag a regex engine and about a megabyte of
metadata into wasm, and a length check does not catch a wrong number of the
right length anyway.

A rule over the number is an ordinary `validate` line over the E.164:

```rust
PhoneField {
    label: "Mobile",
    validate: Rule::required().error("Enter a phone number."),
    value: phone(),
    oninput: move |next| phone.set(next),
}
```

Formatting is the one thing the field does with the digits, and it happens on
blur rather than as the number is typed: regrouping on every keystroke moves the
caret, and the platform layer has no way to put it back. The grouping itself is
deliberately sparse - it applies where a numbering plan has one fixed shape
(the NANP, Russia and Kazakhstan, France) and leaves the digits alone
everywhere else.

## The country

`country` is the country the field *starts* on, ISO 3166-1 alpha-2, and defaults
to `PhoneFieldDefaults::country`. The picker wins over it from then on - until
the prop itself changes, which moves the field and re-emits the number under the
new dial code, exactly as a pick does. `oncountrychange` reports a *pick* only,
since a caller changing the prop already knows; `oninput` fires either way,
carrying the same digits under the new dial code.

`country_select: false` pins the country and draws a static `+49` where the
picker was, which is also one tab stop fewer - the same kind of escape hatch
`reveal_button: false` is on [PasswordField](password_field.md).

`countries` narrows the list, in the order given. `country_label` overrides the
English name of one country during render, so it can read a locale out of
context; the library bundles no translations.

No flags ship with it. Emoji flags render as two regional-indicator letters on
Windows, and an inline SVG sprite of every flag costs about 44 KB gzipped, so
`flag` is a closure a caller fills:

```rust
PhoneField {
    label: "Mobile",
    countries: vec!["DE".into(), "FR".into()],
    flag: move |iso: String| rsx! { MyFlag { iso } },
}
```

## Inside a `Form`

The visible input holds the text being edited, so it carries no `name` - it
would post what is on screen. A hidden input of that name posts the E.164
instead, the shape [Slider](slider.md) and [PinField](pin_field.md) already use.
A `FieldName` built from a path also binds the number to the form's own value,
so a bound field needs no `oninput`:

```rust
Form {
    value: signup,
    PhoneField { label: "Mobile", country: "DE", name: Signup::FIELDS.phone() }
}
```

## Accessibility

The country picker is a second tab stop. Enter, Space and ArrowDown open the
list, typing filters it, the arrows move the highlight, Enter picks and Escape
closes; both hand focus back to the picker. Everything in the text input is
native.

## Props

### `PhoneField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The number, in E.164. `None` leaves the field uncontrolled. |
| `oninput` | `EventHandler<String>` | - | Fires per keystroke with the E.164 the field should hold next. |
| `country` | `String` | `US` | The country the field starts on, ISO 3166-1 alpha-2. Themed. |
| `oncountrychange` | `EventHandler<String>` | - | The user picked another country. |
| `country_select` | `bool` | `true` | Offers the picker at all. Off pins the country and draws a static dial code. |
| `country_label` | `Callback<String, String>` | - | Overrides a country's English name, during render. |
| `countries` | `Vec<String>` | - | Narrows the list to these ISO codes, in the order given. |
| `flag` | `Callback<String, Element>` | - | Draws a flag beside a country. The library ships none. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `name` | `FieldName<String>` | - | What the field posts as, through a hidden input. |
| `validate` | `Validators<String>` | - | Rules over the E.164, shown after a blur or a submit. |
| `label` | `Caption` | - | The field's caption, above the control. Names the field through a `for`/`id` pair. |
| `description` | `Caption` | - | Between the label and the control: what to enter. |
| `helper` | `Caption` | - | Under the control: the format, or an example. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes, since the props extend `input`'s own.

## Theme defaults

`PhoneFieldDefaults { size, radius, country }`. The frame's numbers live on
`FieldDefaults` and the picker's list on `ComboboxDefaults`, so a phone field
lines up with a [TextField](text_field.md) above it and with a
[Select](select.md)'s list below it by construction.

`country` is a real theme knob: a German app sets its starting country once
instead of on every field. The country *names* deliberately stay out of the
theme - 240 of them would dwarf it - and come from the component's own table,
with `country_label` as the live override.

## Data attributes

The same as [TextField](text_field.md): state tokens on the wrapper's and the
frame's `data-state`, plus `data-slot="leading"` on the picker's slot.
