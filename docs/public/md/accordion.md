# Accordion

Crate: `libero`
Import: `use libero::components::{Accordion, AccordionOpen, OptionLabel, Options};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/accordion>
Index: [index.md](index.md) - every other component's markdown page
Description: Sections over an enum, each a heading whose button expands its panel; one or many open.

A list of sections over an enum, each a heading with a button that expands its
panel. The sections are the enum's variants - `#[derive(Options)]` lists them in
declaration order and names each one - and `panel` is a match over the same type,
so a forgotten section is a compile error. A closed panel's content is not
mounted: what it held (a half-typed form) is gone when it closes.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Accordion, AccordionOpen, Options, Text};

#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    #[option(label = "Payment method")]
    Payment,
    Review,
}

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));

    rsx! {
        Accordion {
            open: open(),
            onchange: move |next| open.set(next),
            panel: |step: Step| match step {
                Step::Shipping => rsx! { Text { "Where should the parcel go?" } },
                Step::Payment => rsx! { Text { "Card, invoice or bank transfer." } },
                Step::Review => rsx! { Text { "Check the order, then place it." } },
            },
        }
    }
}
```

## One or many: the variant is the mode

`open: AccordionOpen<T>` is strictly controlled, and there is no `multiple` flag:

- `AccordionOpen::One(Option<T>)` holds at most one section. Opening another
  closes the first, by construction - "single mode with two open" cannot be built.
- `AccordionOpen::Many(Vec<T>)` toggles each section on its own.

`onchange` hands back the whole new set in the same mode, ready to `set`.
`From<Option<T>>` and `From<Vec<T>>` build either; `.one()` and `.values()` read
them back.

## Props

| Prop | Type | Default | What |
|---|---|---|---|
| `open` | `AccordionOpen<T>` | `One(None)` | Which sections are expanded; the variant is the mode |
| `onchange` | `EventHandler<AccordionOpen<T>>` | - | The new open set |
| `panel` | `Callback<T, Element>` | - | A section's body. Closed content is never mounted |
| `options` | `OptionSource<T>` | `T::options()` | The sections to show. A `Vec<T>` converts; an `OptionList<T>` adds per-option `disabled` - a section that renders, cannot toggle, and stays a tab stop. Named groups are drawn flattened |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Rename a section, or `OptionLabel::rich` to draw it as rsx |
| `heading` | `HtmlTag` | `h3` | The heading around each trigger, `h1`..`h6` |
| `size` | `Size` | `theme.accordion.size` (`md`) | Type and padding |

Plus `class`, `sx`, `states` and any global attribute (`id` seeds the ids below).

## Accessibility

- Every trigger is a tab stop. Enter and Space toggle. Up and Down move focus to
  the next or previous trigger (wrapping, skipping disabled ones); Home and End
  jump to the ends. Arrows never toggle.
- Heading level: decide the level the page outline needs first, then the size.
  `h3` assumes a section title above the accordion.
- Every open panel is a region landmark, so a `Many` accordion with a dozen
  open sections produces a long landmark list.

## Theme

`theme.accordion`: `size`, `sizes` (font size, padding, chevron per step),
`border_color` (the line between sections), `hover_color`, `chevron_duration`.
The panel's height animation is `theme.collapse.duration` - each panel is a
`Collapse`.
