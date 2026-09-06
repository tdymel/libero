# Tabs

Crate: `libero`
Import: `use libero::components::{OptionLabel, Options, Tabs};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/tabs>
Index: [index.md](index.md) - every other component's markdown page
Description: One strip of tabs over an enum, with only the selected tab's panel built.

One strip of tabs over an enum, with the selected tab's panel below it. The tabs
are the enum's variants - `#[derive(Options)]` lists them in declaration order
and names each one - and `panel` is a match over the same type, so a forgotten or
misspelled tab is a compile error rather than a blank page. Only the selected
panel is built at all; the others cost nothing until they are picked.

`option_label` overrides what the derive named a tab, and it runs during render - so it
can read a locale from a signal or from context, and the strip repaints when that
changes. Return a string to rename a tab, or `OptionLabel::rich` to draw it as rsx
(an icon, a badge) - that one asks for the name as well, since the rsx is what a
screen reader cannot use.

## Usage

The enum *is* the tab strip, so it is part of every snippet. `#[option(label = ..)]`
renames a variant whose Rust name is not what a reader should see.

```rust
use dioxus::prelude::*;
use libero::components::{Options, Tabs};

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    #[option(label = "Admin area")]
    Admin,
    Billing,
}

#[component]
fn Demo() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Tabs {
            value: section(),
            onchange: move |next| section.set(next),
            panel: |section: Section| match section {
                Section::Account => rsx! { "Account settings" },
                Section::Admin => rsx! { "Admin area" },
                Section::Billing => rsx! { "Billing details" },
            },
        }
    }
}
```

Strictly controlled: `value` is the tab that is drawn as selected, and `onchange`
asks for the next one. Without `onchange` the selection can never change, and
without `panel` there is nothing below the strip - the library warns about
either.

`option_label` renames the whole strip at once. Because it runs during render, reading a
locale signal inside it is enough to make the strip follow the language:

```rust
use dioxus::prelude::*;
use libero::components::{OptionLabel, Options, Tabs};

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Admin,
    Billing,
}

#[component]
fn Demo() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Tabs {
            value: section(),
            onchange: move |next| section.set(next),
            option_label: |section: Section| -> OptionLabel {
                match section {
                    Section::Account => "Konto".into(),
                    Section::Admin => "Verwaltung".into(),
                    Section::Billing => "Rechnung".into(),
                }
            },
            panel: |section: Section| match section {
                Section::Account => rsx! { "Konto" },
                Section::Admin => rsx! { "Verwaltung" },
                Section::Billing => rsx! { "Rechnung" },
            },
        }
    }
}
```

`OptionLabel::rich` draws a tab as rsx and names it separately. A tab is a
`<button>`, so its content has to stay phrasing content: an [Icon](icon.md) is an
inline-flex `<span>`, while a [Flex](flex.md) is a `<div>` and does not belong
there.

```rust
use dioxus::prelude::*;
use libero::components::{Icon, OptionLabel, Options, Tabs};

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Admin,
    Billing,
}

#[component]
fn Demo() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Tabs {
            value: section(),
            onchange: move |next| section.set(next),
            option_label: |section: Section| OptionLabel::rich(
                section.label(),
                rsx! {
                    Icon { variant: "transparent", size: "sm", FileIcon {} }
                    "{section.label()}"
                },
            ),
            panel: |section: Section| rsx! { "{section.label()}" },
        }
    }
}
#
# #[component] fn FileIcon() -> Element { rsx! {} }
```

`FileIcon` there is your own icon component - any `svg` will do; `Icon` is what
sizes it.

`tabs` narrows the strip to a subset of the enum's variants, and `disabled_options` lists
tabs that render but cannot be picked.

## Accessibility

Only the selected tab is in the tab order. Left and Right move between tabs and
select as they go, stepping over disabled ones; Home and End jump to the ends.

`OptionLabel::rich` takes the accessible name as its first argument: the rsx it
draws is what a screen reader cannot use, and that name becomes the tab's
`aria-label`.

## Props

### Tabs

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | required | The selected tab. Strictly controlled - pair it with `onchange`. |
| `onchange` | `EventHandler<T>` | - | Called with the tab that should become selected. |
| `panel` | `Callback<T, Element>` | - | The body of the selected tab. Called for `value` only, so the other panels cost nothing. |
| `tabs` | `Vec<T>` | `T::options()` | The tabs to show. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Overrides what the derive named a tab. Runs during render, so it can read a locale from context - which is how a renamed strip stays renamed. |
| `disabled_options` | `Vec<T>` | - | Tabs that render but cannot be picked. |
| `size` | `Size` | `md` | Tab strip size. |
| `color` | `ThemeAwareValue` | `primary` | Indicator and selected-label color. |
| `full_width` | `bool` | `false` | Tabs share the row evenly instead of sizing to their label. |

Like every component, `Tabs` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

### OptionLabel

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The tab's visible text and accessible name. |
| `content` | `Element` | - | Drawn in place of the name, via `OptionLabel::rich` - an icon or a badge. `name` still names the tab, since the rsx is what a screen reader cannot use. |

`OptionLabel` is a value, not a component - it takes no shared props. A bare string
converts into one (`"Konto".into()`).

## Theme defaults

`TabsDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `sizes` | `Sizes<TabsSizeLevel>` | `font_size`, `padding_x`, `padding_y`, `indicator`, `icon_gap` per size. |
| `border_color` | `ColorValue` | The line the whole strip sits on. |
| `hover_color` | `ColorValue` | Background of an unselected tab while hovered. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-tabs-font-size-<size>` | `font-size` for that size step. |
| `--lsx-tabs-padding-x-<size>` | Horizontal tab padding for that size step. |
| `--lsx-tabs-padding-y-<size>` | Vertical tab padding for that size step. |
| `--lsx-tabs-indicator-<size>` | Thickness of the selected tab's underline. |
| `--lsx-tabs-icon-gap-<size>` | Gap between a rich label's icon and its text. |
| `--lsx-tabs-pad-x` | The picked step's horizontal padding, resolved on the strip so each tab inherits it. |
| `--lsx-tabs-pad-y` | The picked step's vertical padding. |
| `--lsx-tabs-line` | The picked step's indicator thickness. |
| `--lsx-tabs-gap` | The picked step's icon gap. |
| `--lsx-tabs-border-color` | Color of the strip's 1px line. |
| `--lsx-tabs-hover` | Hover background of an unselected tab. |
| `--lsx-tabs-color` | Indicator and selected-label color, from the `color` prop. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `full-width` | `full_width` is set. |

The tabs themselves carry no `data-state`; their styling keys off ARIA -
`[aria-selected="true"]` for the indicator, `[aria-disabled="true"]` for the
dimmed look.
