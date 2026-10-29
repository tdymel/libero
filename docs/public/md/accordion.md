# Accordion

Crate: `libero`
Import: `use libero::components::{Accordion, AccordionOpen, OptionLabel, Options};`
Feature: `accordion` (off by default, turns on `collapse`, `use-icon`, `use-id`; [features.md](features.md))
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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `open` | `AccordionOpen<T>` | `One(None)` | Which sections are open. `One(Option<T>)` holds at most one, `Many(Vec<T>)` any number. Controlled, so pair it with `onchange`. An `Option<T>` or a `Vec<T>` converts into one. |
| `onchange` | `EventHandler<AccordionOpen<T>>` | `None` | Called with the whole new open set, in the same mode, ready to store. |
| `panel` | `Callback<T, Element>` | `None` | A section's body. A closed panel is not mounted, so it keeps no state. A closure that captures a plain value redraws only when its section's value or open state changes. |
| `options` | `OptionSource<T>` | `T::options()` | The sections to show. A `Vec<T>` converts. An `OptionList<T>` can disable a section, which renders, cannot be toggled and stays a tab stop. A disabled section the host opens through `open` cannot be closed from its trigger, so open only enabled ones. Groups are drawn flat. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Renames a section. `OptionLabel::rich` draws the trigger as rsx and still names it. |
| `heading` | `HtmlTag` | `h3` | The heading around each trigger, `h1` to `h6`. |
| `size` | `Size` | `md` | Type and padding of the triggers and panels. Panel content takes the type size only where it sets none of its own; `Text` sets its own. |
| `collapsible` | `bool` | `true` | `false` keeps the open section open in `One` mode: its trigger turns `aria-disabled` and only opening another closes it. `Many` mode ignores it. |
| `chevron_position` | `AccordionChevronPosition` | `end` | `end` or `start`: the chevron after or before the label. Logical, so right-to-left flips it. |
| `rotate_chevron` | `bool` | `true` | `false` keeps the chevron pointing down when a section opens. |

Like every component, `Accordion` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes. An `id` seeds the ids of its parts.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Moves between the triggers: each one is a tab stop. |
| `Enter` or `Space` | Toggles the focused section. |
| `Up` or `Down` | Moves to the previous or next enabled trigger, skipping disabled ones and wrapping at the ends, without toggling. |
| `Home` or `End` | Jumps to the first or last enabled trigger, without toggling. On the first or last one it stays the page's. |

### Libero handles

- Every open panel is a region named by its trigger.
- Panel content wider than the panel wraps, or scrolls inside the panel,
  instead of being cut off.
- With `collapsible: false` the open trigger is `aria-disabled` and stays a tab
  stop, and no cursor or hover invites a press.

### You must

- Pick the heading level the page outline needs, then the size. `h3` assumes a
  section title above the accordion.
- With `OptionLabel::rich`, make the name contain the visible text, since it
  replaces the drawn label (WCAG 2.5.3).

### Example

A FAQ under the section title "Shipping", with the default `h3` heading: a
screen reader lists each question as a level 3 heading, and Enter opens its
answer as a region named by the question.

### Limits

- A `Many` accordion with a dozen open sections makes a long landmark list.

## Theme defaults

`theme.accordion` holds `size`, `sizes` (font size, padding and chevron per
step), `border_color` (the line between sections), `hover_color` and
`chevron_duration`. Each panel is a `Collapse`, so its height animation is
`theme.collapse.duration`.
