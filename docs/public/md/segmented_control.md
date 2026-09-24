# SegmentedControl

Crate: `libero`
Import: `use libero::components::{Options, SegmentedControl};`
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
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        SegmentedControl {
            value: alignment(),
            onchange: move |next| alignment.set(next),
        }
    }
}
```

`#[option(label = "..")]` renames a segment at the type. `option_label` renames
it during render, which is what a translated strip needs.

`OptionLabel::rich` draws a segment as rsx and names it separately, since a
screen reader reads only the name. A segment is a `<label>`, so its content must
be phrasing content. An [Icon](icon.md) is a `span` and works, a
[Flex](flex.md) is a `div` and does not.

```rust
use dioxus::prelude::*;
use libero::components::{Icon, OptionLabel, Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Center);

    rsx! {
        SegmentedControl {
            value: alignment(),
            onchange: move |next| alignment.set(next),
            option_label: |alignment: Alignment| OptionLabel::rich(
                alignment.label(),
                rsx! {
                    Icon {
                        variant: "standard",
                        size: "sm",
                        match alignment {
                            Alignment::Left => rsx! { AlignLeftIcon {} },
                            Alignment::Center => rsx! { AlignCenterIcon {} },
                            Alignment::Right => rsx! { AlignRightIcon {} },
                        }
                    }
                    "{alignment.label()}"
                },
            ),
        }
    }
}
#
# #[component] fn AlignLeftIcon() -> Element { rsx! {} }
# #[component] fn AlignCenterIcon() -> Element { rsx! {} }
# #[component] fn AlignRightIcon() -> Element { rsx! {} }
```

The `Align*Icon`s are your own icon components. `Icon` sizes and colors them.

A runtime set passes its segments in `options`, since `String` lists none of
its own:

```rust
use dioxus::prelude::*;
use libero::components::SegmentedControl;

#[component]
fn Demo(options: Vec<String>) -> Element {
    let mut selected = use_signal(|| options[0].clone());

    rsx! {
        SegmentedControl {
            value: selected(),
            options: options.clone(),
            onchange: move |next| selected.set(next),
        }
    }
}
```

Bound to a [Form](form.md) through a path, it needs neither `value` nor
`onchange`. The selection is read from the form's value and written back:

```rust
use dioxus::prelude::*;
use libero::components::{Fields, Form, Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Default, Options)]
enum Density {
    Compact,
    #[default]
    Cosy,
    Roomy,
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Layout {
    density: Density,
}

#[component]
fn Demo() -> Element {
    let layout = use_store(Layout::default);

    rsx! {
        Form {
            value: layout,
            SegmentedControl {
                name: Layout::FIELDS.density(),
                label: "Density",
                helper: "Applies to every table.",
            }
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

## Props

### SegmentedControl

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | - | The selected segment. Pair it with `onchange`. A control bound to a `Form` through `name` leaves it out. |
| `onchange` | `EventHandler<T>` | - | Called with the segment to select next. |
| `name` | `FieldName<T>` | - | What the control posts as. A path such as `Settings::FIELDS.align()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<T>` | - | Rules over the selection, shown once the control loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the strip. A runtime set of `String`s goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single segments. Named groups are drawn flat, without headings. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Renames a segment, or draws it with `OptionLabel::rich`. Runs during render, so it can read a locale from context. |
| `orientation` | `Orientation` | `horizontal` | A row or a column. |
| `variant` | `Variant` | `filled` | The unselected look, shared by every segment. |
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color. |
| `size` | `Size` | `md` | Size of the segments and the captions. |
| `radius` | `Size` | `md` | Radius of the control's outer corners. Inner corners are square. |
| `gap` | `Size` | - | Space between the segments. Set, each segment gets its own border and radius. |
| `full_width` | `bool` | `false` | Segments share the width evenly instead of sizing to their label. A label too long for its segment ends in an ellipsis either way. |
| `focusable` | `bool` | `true` | `false` keeps the segments out of the tab order, and a click leaves focus where it is. For a control inside a field's dropdown. |
| `label` | `Caption` | - | The question, and the group's name. |
| `description` | `Caption` | - | Between the label and the segments. How to choose. |
| `helper` | `Caption` | - | Under the segments. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` on the group and marks the label. |
| `disabled` | `bool` | `false` | Disables every segment and dims the captions. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the control from the tab order and the post instead. |

### OptionLabel

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The segment's accessible name, and its text when there is no `content`. |
| `content` | `Element` | - | Drawn in place of the name, such as an icon and text. `name` still names the segment for screen readers. |

`OptionLabel` is a value, not a component, so it takes no shared props. A bare
string converts (`"Konto".into()`). [Tabs](tabs.md) uses it too.

Like every component, `SegmentedControl` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes. `sx`, `class` and
`states` land on the field's wrapper, and the attributes on the
`radiogroup`.

## Theme defaults

`SegmentedControlDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). Separate from `ButtonDefaults::variant`. |

For `size` and `radius` the control reads [Button](button.md)'s
`ButtonDefaults`. `gap` resolves against the theme's spacing scale.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-button-color` | The accent, resolved from `color`. Set on the root, and the segments inherit it. |
| `--lsx-button-contrast` | Label color on a `filled` segment. |
| `--lsx-button-hover` | Hover background. |
| `--lsx-button-selected` | Background of the selected segment. |
| `--lsx-spacing-<size>` | Read for `gap` when it is set. |
| `--lsx-spacing-xs` | The gap inside a segment, between a rich label's icon and its text. |

## Data attributes

The field's wrapper carries the usual field tokens: size, radius, status,
`disabled` and `required`. State tokens on the `radiogroup`'s `data-state`,
space separated:

| Token | Condition |
|---|---|
| `horizontal` / `vertical` | The `orientation` in effect. |
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `full-width` | `full_width` is set. |
| `collapsed` | No `gap`. The segments share borders and square off their inner corners. |
| `size-<size>` | The `gap` step in effect, when `gap` is set. |

Each segment `<label>` carries its own `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` / `radius-<size>` | The control's `size` and `radius`. |
| `checked` | The segment is the selected one. |
| `disabled` | The segment's `OptionItem` is `disabled`, or the whole control is. |
