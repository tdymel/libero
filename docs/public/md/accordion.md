# Accordion

Crate: `libero`
Import: `use libero::components::{Accordion, AccordionOpen, OptionLabel, Options};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/accordion>
Index: [index.md](index.md) lists every other page
Description: Sections over an enum, each a heading whose button opens its panel, with one or many open.

Sections over an enum, each a heading with a button that opens its panel.
`#[derive(Options)]` lists the variants in order and names each one. `panel` is
a match over the same enum, so a forgotten section is a compile error. A closed
panel is not mounted, so what it held, such as a half-typed form, is gone when
it closes.

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

## Open sections

`open` is controlled, and its variant is the mode. `AccordionOpen::One` holds at
most one section, so opening another closes the first. `AccordionOpen::Many`
toggles each on its own. `onchange` hands back the whole new set.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `open` | `AccordionOpen<T>` | `One(None)` | Which sections are open. `One(Option<T>)` holds at most one, `Many(Vec<T>)` any number. Controlled, so pair it with `onchange`. An `Option<T>` or a `Vec<T>` converts into one. |
| `onchange` | `EventHandler<AccordionOpen<T>>` | `None` | Called with the whole new open set, in the same mode, ready to store. |
| `panel` | `Callback<T, Element>` | `None` | A section's body. A closed panel is not mounted, so it keeps no state. |
| `options` | `OptionSource<T>` | `T::options()` | The sections to show. A `Vec<T>` converts. An `OptionList<T>` can disable a section, which renders, cannot be toggled and stays a tab stop. Groups are drawn flat. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Renames a section. `OptionLabel::rich` draws the trigger as rsx and still names it. |
| `heading` | `HtmlTag` | `h3` | The heading around each trigger, `h1` to `h6`. |
| `size` | `Size` | `md` | Type and padding of the triggers and panels. |

Like every component, `Accordion` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes. An `id` seeds the ids of its parts.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Moves between the triggers: each one is a tab stop. |
| `Enter` or `Space` | Toggles the focused section. |
| `Up` or `Down` | Moves to the previous or next trigger, without toggling. |
| `Home` or `End` | Jumps to the first or last trigger, without toggling. |

### Libero handles

- Every open panel is a region named by its trigger.

### You must

- Pick the heading level the page outline needs, then the size. `h3` assumes a
  section title above the accordion.
- With `OptionLabel::rich`, make the name contain the visible text, since it
  replaces the drawn label (WCAG 2.5.3).

### Limits

- A `Many` accordion with a dozen open sections makes a long landmark list.

## Theme defaults

`theme.accordion` holds `size`, `sizes` (font size, padding and chevron per
step), `border_color` (the line between sections), `hover_color` and
`chevron_duration`. Each panel is a `Collapse`, so its height animation is
`theme.collapse.duration`.
