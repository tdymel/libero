# PhoneField

Crate: `libero`
Import: `use libero::components::PhoneField;`
Feature: `phone-field` (off by default, turns on `combobox`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/phone_field/field.rs>
Index: [index.md](index.md) lists every other page
Description: A country picker in front of a `tel` input, whose value is an E.164 string.

A country picker in front of a `tel` input. The picker holds the dial code, the
input the national number, and the value is one E.164 string such as
`"+12133734253"`. The field regroups the digits when it loses focus, for
countries with a fixed number format. Below the `sm` breakpoint the country list
opens as a sheet at the foot of the screen. It does not validate the number or
strip a trunk prefix (`0171 1234567` under Germany posts `"+4901711234567"`), so
add a rule through `validate`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::PhoneField;

#[component]
fn Demo() -> Element {
    let mut phone = use_signal(String::new);

    rsx! {
        PhoneField {
            value: phone(),
            oninput: move |next| phone.set(next),
            country: "DE",
            label: "Mobile",
            flag: move |iso: String| rsx! { Flag { iso } },
        }
    }
}
#
# #[component]
# fn Flag(iso: String) -> Element { rsx! {} }
```

## Props

### `PhoneField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. Or any CSS, e.g. `radius: "0"`. |
| `value` | `Option<String>` | - | The number in E.164, such as `"+12133734253"`. Leave it out and the field keeps its own text. |
| `oninput` | `EventHandler<String>` | - | Fires on every keystroke with the E.164 the field should hold next, or an empty string once nothing is typed. |
| `country` | `String` | `US` | The country the field starts on, ISO 3166-1 alpha-2. A pick wins over it until the prop changes. A `value` with another country's dial code wins over both. |
| `oncountrychange` | `EventHandler<String>` | - | The user picked another country, or typed another country's dial code. `oninput` fires at the same time with the number under the new dial code. |
| `country_select` | `bool` | `true` | Shows the country picker. Off pins the country and shows its dial code as plain text. A typed or pasted dial code of another country still wins over the pin, and the prefix changes. |
| `country_label` | `Callback<String, String>` | - | Overrides the name of a country. Unset, the name comes from the localization's `phone_field.country_names`, which ships in German, else English. It runs during render, so it can read a locale from context. |
| `countries` | `Vec<String>` | - | Narrows the list to these ISO codes, in the order given. It limits the picker only: a typed or pasted dial code of a country outside it still switches the field to that country. |
| `flag` | `Callback<String, Element>` | - | Draws a flag beside a country, in the picker and in the list. The library ships none. Hide it from screen readers, since the country's name is already read. |
| `validate` | `Validators<String>` | - | Rules over the E.164, shown once the field loses focus or its form is submitted. The field checks nothing on its own. |
| `name` | `FieldName<String>` | - | What the field posts as, the E.164. A path such as `Signup::FIELDS.phone()` also binds the number to the surrounding `Form`'s value when the field has no `oninput`. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `label` | `Caption` | - | The field's caption, above the control. It names the input. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. The format, or an example. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label; a field without a visible `label` shows no mark. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Disables and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. The country button stays focusable and opens nothing. |
| `dropdown_parts` | `Parts<DropdownPart>` | - | Styles the portaled country list and its inner parts. |

`PhoneField` also takes the `<input>` HTML attributes and, like every
component, the shared props `sx`, `class`, `style`, `states`, and any extra
HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `PhoneFieldPart::Label` | `label` | The label above the control. |
| `PhoneFieldPart::Required` | `required` | The required asterisk, in the label. |
| `PhoneFieldPart::Description` | `description` | The caption between the label and the control. |
| `PhoneFieldPart::Frame` | `frame` | The bordered box around the control. |
| `PhoneFieldPart::Leading` | `leading` | The slot before the control: an icon, a prefix. |
| `PhoneFieldPart::Control` | `control` | The element the label names. |
| `PhoneFieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `PhoneFieldPart::Country` | `country` | The country button in the leading slot, with `country_select`. |
| `PhoneFieldPart::Dial` | `dial` | The dial code: in the country button, or alone without `country_select`. |
| `PhoneFieldPart::Helper` | `helper` | The caption under the control. |
| `PhoneFieldPart::Status` | `status` | The validation message. |

### Dropdown

The dropdown is portaled out of the field, so its parts take the
`dropdown_parts` prop. They match from the dropdown box at any depth.

| Part | `data-slot` | Description |
|---|---|---|
| `DropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `DropdownPart::SheetHandle` | `sheet-handle` | Below `sm`: the swipe strip atop the sheet. |
| `DropdownPart::Search` | `search` | The search box above the rows. |
| `DropdownPart::Listbox` | `listbox` | The scrolling list of rows. |
| `DropdownPart::Option` | `option` | A row. |
| `DropdownPart::Empty` | `nothing-found` | The text shown when the query matches nothing. |
| `DropdownPart::CountryName` | `name` | A row's country name. |
| `DropdownPart::CountryDial` | `dial` | A row's dial code. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Enter`, `Space` or `Down` | On the country picker: opens the list. |
| `Letter` | Filters the open list. |
| `Up` or `Down` | Move the highlight. |
| `PageUp` or `PageDown` | Moves the highlight 10 rows, stopping at the first or last. |
| `Enter` | Picks the highlighted country and returns focus to the picker. |
| `Escape` | Closes the list and returns focus to the picker. |

### Libero handles

- The country picker is a second tab stop.
- Android's Back button closes the country list as Escape does, rather than
  the app.
- A typed dial code of another country moves the picker and fires
  `oncountrychange`, and a polite status says so, "Country set to Germany", from
  `PhoneFieldLabels::country_set`.

### Example

A contact number, `PhoneField { label: "Phone", .. }`: Tab stops on the
country picker, then on the number, and typing a letter in the open list
filters the countries.

## Theme defaults

`PhoneFieldDefaults` holds `size`, `radius`, `country` (`US`) and
`country_select` (`true`). Set `country` once in the theme instead of on every
field. The frame reads `FieldDefaults` and the list `ComboboxDefaults`, so a
phone field lines up with a [TextField](text_field.md) and its list with a
[Select](select.md)'s.

## Data attributes

The same as [TextField](text_field.md). The picker's slot carries
`data-slot="leading"`.
