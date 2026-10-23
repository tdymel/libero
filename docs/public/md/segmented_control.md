# SegmentedControl

Crate: `libero`
Import: `use libero::components::{Options, SegmentedControl};`
Feature: `segmented-control` (off by default, turns on `button`, `use-id`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/segmented_control>
Index: [index.md](index.md) lists every other page
Description: A connected strip of segments over an enum, exactly one of them selected, with the field slots.

A connected strip of segments over an enum, exactly one of them selected. The
segments are the enum's variants, so a misspelled one does not compile. Pass
`value` with `onchange`, or bind it to a [Form](form.md) through `name`. A
runtime set of `String`s goes in `options`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    #[option(label = "Right edge")]
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        SegmentedControl {
            value: alignment(),
            onchange: move |next| alignment.set(next),
            label: "Alignment",
        }
    }
}
```

## Props

### SegmentedControl

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | - | The selected segment. Pair it with `onchange`. A control bound to a `Form` through `name` leaves it out. |
| `onchange` | `EventHandler<T>` | - | Called with the segment to select next. |
| `name` | `FieldName<T>` | - | What the control posts as. A path such as `Settings::FIELDS.align()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<T>` | - | Rules over the selection, shown once the control loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the strip. A runtime set of `String`s goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single segments. Named groups are drawn flat, without headings. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Renames a segment, or draws it with `OptionLabel::rich`. Runs during render, so it can read a locale from context. `#[option(label = "..")]` renames one at the type. |
| `orientation` | `Orientation` | `horizontal` | A row or a column. |
| `variant` | `Variant` | `filled` | The unselected look, shared by every segment: `filled`, `tonal`, `elevated`, `outlined`, `standard`, or `gradient`, which takes the theme's gradient. |
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color; `theme.segmented_control.color` when unset. |
| `size` | `Size` | `md` | Size of the segments and the captions. |
| `radius` | `ThemeAwareValue` | `md` | Radius of the control's outer corners. Inner corners are square. Or any CSS, e.g. `radius: "0"`. |
| `gap` | `ThemeAwareValue` | - | Space between the segments, or any CSS, e.g. `gap: "0"`. Set, each segment gets its own border and radius. Unset, a wrapped second line keeps the joined look, so set it where the row may wrap. |
| `full_width` | `bool` | `false` | Segments share the width evenly instead of sizing to their label. A label too long for its segment ends in an ellipsis either way. |
| `focusable` | `bool` | `true` | `false` keeps the segments out of the tab order, and a click leaves focus where it is. For a control inside a field's dropdown. |
| `label` | `Caption` | - | The question, and the group's name. |
| `description` | `Caption` | - | Between the label and the segments. How to choose. |
| `helper` | `Caption` | - | Under the segments. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` on the group and marks the label. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Disables every segment and dims the captions. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the control from the tab order and the post instead. |

### OptionLabel

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The segment's accessible name, and its text when there is no `content`. |
| `content` | `Element` | - | Drawn in place of the name, such as an icon and text. `name` still names the segment for screen readers. A segment is a `<label>`, so phrasing content only: an `Icon` works, a `Flex` does not. |

`OptionLabel` is a value, not a component, so it takes no shared props. A bare
string converts (`"Konto".into()`). [Tabs](tabs.md) uses it too.

Like every component, `SegmentedControl` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes. `sx`, `class` and
`states` land on the field's wrapper, and the attributes on the
`radiogroup`.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SegmentedControlPart::Label` | `label` | The label above the control. |
| `SegmentedControlPart::Required` | `required` | The required asterisk, in the label. |
| `SegmentedControlPart::Description` | `description` | The caption between the label and the control. |
| `SegmentedControlPart::Control` | `control` | The connected strip. |
| `SegmentedControlPart::Segment` | `segment` | One segment's visible label. |
| `SegmentedControlPart::Helper` | `helper` | The caption under the control. |
| `SegmentedControlPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters and leaves the whole control. |
| `Left`, `Right`, `Up` or `Down` | Move the selection. |
| `Space` | Picks the focused segment. |
| `Enter` | Outside a `Form`: picks the focused segment. Inside one: submits the form, as on a native radio. |

### You must

- Without a visible label, spread `"aria-label"`, since the segments name the
  options, not the question.

### Limits

- A row too long for its container wraps (WCAG 1.4.10). Without `gap`, the
  second line keeps the joined look: its first segment overlaps by 1px and its
  ends stay square. Set `gap` or a vertical `orientation` where it may wrap.
- With `selected_check`, picking a segment widens it by the check, so the strip
  reflows; theme `selected_check: false` keeps its width.

### Example

An alignment strip with no visible label, `"aria-label": "Alignment"`: Tab
enters the whole strip once, the arrows move the selection, and the next Tab
leaves it.

## Theme defaults

`SegmentedControlDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). Separate from `ButtonDefaults::variant`. |
| `selected_check` | `bool` | Draws a check before the selected segment's label, so the selection is not a colour change alone (`true`). The picked segment grows by the check, so the strip reflows; `false` keeps its width. |

For `size` and `radius` the control reads [Button](button.md)'s
`ButtonDefaults`. `gap` resolves against the theme's spacing scale.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-button-color` | The accent, resolved from `color`. Set on the root, and the segments inherit it. |
| `--lsx-button-contrast` | Label color on a `filled` segment. |
| `--lsx-button-hover` | Hover background. |
| `--lsx-button-selected` | Background of the selected segment, and of a segment while pressed. |
| `--lsx-spacing-<size>` | Read for `gap` when it is set. |
| `--lsx-spacing-xs` | The gap inside a segment, between a rich label's icon and its text. |

## Data attributes

The field's wrapper carries the usual field tokens: size, radius, status,
`disabled` and `required`. State tokens on the `radiogroup`'s `data-state`,
space separated:

| Token | Condition |
|---|---|
| `horizontal` / `vertical` | The `orientation` in effect. |
| `filled` / `tonal` / `elevated` / `outlined` / `standard` / `gradient` | The `variant` in effect. |
| `full-width` | `full_width` is set. |
| `collapsed` | No `gap`. The segments share borders and square off their inner corners. |
| `size-<size>` | The `gap` step in effect, when `gap` is set. |

Each segment `<label>` carries its own `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` / `radius-<size>` | The control's `size` and `radius`. |
| `checked` | The segment is the selected one. |
| `disabled` | The segment's `OptionItem` is `disabled`, or the whole control is. |
