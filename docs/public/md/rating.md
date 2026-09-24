# Rating

Crate: `libero`
Import: `use libero::components::Rating;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/rating/rating.rs>
Index: [index.md](index.md) lists every other page
Description: A row of stars picking a value, whole or in halves, by click, sideways drag or arrow keys; read-only or display-only.

A row of stars picking a value, whole or in halves. Click a star, drag across
the row, or use the arrow keys. Pass `value` with `onchange`, or bind it with
`name` in a form. `focusable: false` shows a value only, such as an average.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Rating;

#[component]
fn Demo() -> Element {
    let mut stars = use_signal(|| 3.5);

    rsx! {
        Rating {
            label: "Your rating",
            fractions: 2,
            value: stars(),
            onchange: move |value| stars.set(value),
        }
    }
}
```

An average that only shows, with no tab stop. Screen readers hear the label
and the value, "Average 4.3 of 5":

```rust,ignore
Rating { label: "Average", value: 4.3, focusable: false }
```

`clearable` resets the rating to 0 when the current value is picked again.
`icon` draws another symbol, and `IconProvider`'s `IconSlot::Star` swaps it
for a whole subtree:

```rust,ignore
Rating {
    aria_label: "Love it",
    icon: pictogram_icons_lucide::heart::outlined,
    color: "error",
    clearable: true,
    value: hearts(),
    onchange: move |value| hearts.set(value),
}
```

## Props

### `Rating`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `f64` | `0` | The rating, from 0 (unrated) to `count`. Pair it with `onchange`, or bind it with `name` in a `Form`. A value between steps draws as it is, so an average of 4.3 shows 4.3 stars. |
| `onchange` | `EventHandler<f64>` | - | Called with the value `value` should take next: on a click or tap, along a sideways drag, and on each key. With `0` when `clearable` clears it. |
| `onhover` | `EventHandler<Option<f64>>` | - | The value under a mouse or pen, and `None` once it leaves or presses. The stars show it in place of `value` meanwhile. A touch has no hover. |
| `count` | `u8` | `5` | Number of stars. |
| `fractions` | `u8` | `1` | Steps per star: `1` picks whole stars, `2` halves. Each step is a hit zone of its own. |
| `clearable` | `bool` | `false` | Picking the current value again resets it to 0, and Home or the arrows can go below the first step. Without it, the first step is the lowest a user can pick. |
| `icon` | `SvgData` | `IconSlot::Star` | The symbol, drawn empty and filled in `currentColor`. A stroked glyph turns solid when filled; a solid one only changes colour. |
| `color` | `ThemeAwareValue` | `warning` | The filled stars' colour; `theme.rating.color` when unset. Empty stars are `muted`. |
| `size` | `Size` | `md` | Star size and the gap between stars. |
| `format` | `Callback<f64, String>` | - | What a screen reader says for the value. Runs during render, so it can translate. The localization's `rating.value` ("3.5 of 5") when unset. |
| `focusable` | `bool` | `true` | `false` only shows a value: an image named by the label and the value, with no tab stop, no pointer input and nothing posted. |
| `name` | `FieldName<f64>` | - | What the rating posts as. A path such as `Review::FIELDS.stars()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<f64>` | - | Rules over the value, shown once the rating loses focus or its form is submitted. |
| `label` | `Caption` | - | The caption above the stars, and the slider's name. |
| `description` | `Caption` | - | Under the label. |
| `helper` | `Caption` | - | Under the stars. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the label with an asterisk. No `aria-required`: ARIA does not allow it on a slider. |
| `disabled` | `bool` | `false` | Dims the stars and drops them from the tab order. |
| `readonly` | `bool` | `false` | Focusable, announced and posted, but neither pointer nor keys change it. |
| `aria_label` | `String` | - | Names the rating when it has no `label`. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `RatingPart::Label` | `label` | The label above the control. |
| `RatingPart::Required` | `required` | The required asterisk, in the label. |
| `RatingPart::Description` | `description` | The caption between the label and the control. |
| `RatingPart::Control` | `control` | The row of symbols. |
| `RatingPart::Symbol` | `symbol` | One symbol with its hit area. |
| `RatingPart::Glyph` | `glyph` | A symbol's empty glyph, under its fill. |
| `RatingPart::Fill` | `fill` | A symbol's filled share, in the rating's colour. |
| `RatingPart::Helper` | `helper` | The caption under the control. |
| `RatingPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `ArrowRight` / `ArrowUp` | One step up: a half with `fractions: 2`. Right to left, ArrowLeft goes up instead. |
| `ArrowLeft` / `ArrowDown` | One step down. |
| `Shift+Arrow` / `PageUp` / `PageDown` | A whole star up or down. |
| `Home` / `End` | The lowest step, or every star. |

### Libero handles

- One tab stop, a `slider` with `aria-valuemin` 0, `aria-valuemax` the count
  and the value spoken as "3.5 of 5".
- Hover only previews: `aria-valuenow` stays the picked value.
- Filled and empty stars differ in shape as well as colour with the default
  stroked star.
- A sideways touch drag scrubs; a vertical swipe scrolls the page.
- Display-only (`focusable: false`) is an image named by the label and the
  value.
- A debug build warns when the rating has neither a visible label nor
  `aria_label`.

### You must

- Without a visible label, set `aria_label`.
- Translate the spoken value with the localization or `format`.

### Limits

- At the default `md` size a whole star is a 28px target, a half star 14px
  wide: the row is one slider target, and a drag reaches any half.
  `size: "xl"` makes each half 24px wide.
- A solid custom icon shows the value by colour alone.

## Theme defaults

`RatingDefaults` on the theme, as `theme.rating`.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted (`md`). |
| `sizes` | `Sizes<RatingSizeLevel>` | `glyph_size` and `gap` per size: 16px and 2px at `xs`, 24px and 4px at `md`, 40px and 8px at `xxl`. |
| `color` | `Color` | Default filled colour (`Warning`). |

## Localization

`Localization.rating.value` is the spoken value, with the holes `{value}` and
`{count}`: "{value} of {count}" in English, "{value} von {count}" in German. The
number uses the formats' decimal separator.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-rating-glyph-size-<size>` | Star size for that size step. |
| `--lsx-rating-gap-size-<size>` | Gap between stars for that size step. |
| `--lsx-rating-glyph` / `--lsx-rating-gap` | The picked size step, resolved on the row. |
| `--lsx-rating-color` | The filled stars' colour, resolved on the row. |

## Data attributes

`data-state` on the row carries `size-*`, and `editable`, `dragging`,
`disabled` and `readonly` when they apply. Each star is
`data-slot="symbol"`, holding `data-slot="glyph"` (the empty star and, when
filled, `data-slot="fill"`) and, on an editable rating, `data-slot="zones"`
with one `data-slot="zone"` per step.
