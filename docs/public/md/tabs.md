# Tabs

Crate: `libero`
Import: `use libero::components::{OptionLabel, Options, Tabs};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/tabs>
Index: [index.md](index.md) lists every other page
Description: One strip of tabs over an enum, with only the selected tab's panel built.

A strip of tabs over an enum, with the selected tab's panel below it.
`#[derive(Options)]` lists and names the tabs. `panel` matches on the same enum,
so a missing tab does not compile. Only the selected panel is built.

## Usage

The enum is the tab strip. `#[option(label = ..)]` renames a variant.

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
            aria_label: "Settings",
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

You own `value`, and `onchange` asks for the next tab. Without `onchange` the
selection never changes, and without `panel` nothing shows below the strip.
`Tabs` warns about both.

`option_label` renames the whole strip during render.

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
            aria_label: "Settings",
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

`OptionLabel::rich` draws a tab as rsx and names it separately.

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
            aria_label: "Settings",
            value: section(),
            onchange: move |next| section.set(next),
            option_label: |section: Section| OptionLabel::rich(
                section.label(),
                rsx! {
                    Icon { variant: "standard", size: "sm", FileIcon {} }
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

`FileIcon` is your own icon component. Any `svg` works, and `Icon` sizes it.

## Labels

`option_label` renames tabs during render, so the strip follows a locale
signal. `OptionLabel::rich` draws a tab as rsx, such as an icon, and takes a
text name for screen readers.

## Props

### Tabs

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | required | The selected tab. Pair it with `onchange`. |
| `onchange` | `EventHandler<T>` | - | Called with the tab that should become selected. |
| `panel` | `Callback<T, Element>` | - | The body of the selected tab. Only the selected panel is built. |
| `options` | `OptionSource<T>` | `T::options()` | The tabs to show. A `Vec<T>` converts, and an `OptionList<T>` can disable single tabs. Groups draw flattened. A source still loading draws no tabs. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Overrides a tab's label. Runs during render, so it can read a locale. |
| `size` | `Size` | `md` | Tab strip size. |
| `color` | `ThemeAwareValue` | `primary` | Indicator and selected-label color. |
| `full_width` | `bool` | `false` | Tabs grow to fill the row, never below their label. A crowded strip still scrolls. |
| `activation` | `TabsActivation` | `Automatic` | `Automatic` selects as the arrows move. `Manual` moves only the focus, and Enter or Space selects. Use it for slow panels. |
| `parts` | `Parts<TabsPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

`Tabs` also takes the shared props `sx`, `class`, `states`, and any extra HTML
attributes.

### OptionLabel

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The tab's visible text and accessible name. |
| `content` | `Element` | - | Drawn in place of the name, such as an icon, via `OptionLabel::rich`. `name` still names the tab. Keep it inline: an `Icon` fits, a `Flex` does not. |

`OptionLabel` is a value, not a component, and takes no shared props. A string
converts into one (`"Konto".into()`).

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `TabsPart::List` | `list` | The `tablist` strip. |
| `TabsPart::Tab` | `tab` | One tab. The selected one has `aria-selected="true"`. |
| `TabsPart::Panel` | `panel` | The selected tab's panel. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters the strip at the selected tab, the only one in the tab order. |
| `Left` or `Right` | Moves to the previous or next tab and selects it, skipping disabled ones. |
| `Home` or `End` | Jumps to the first or last tab. |
| `Enter` or `Space` | With `activation: TabsActivation::Manual`, where the arrows move only the focus: selects the focused tab. |

### Libero handles

- Only the selected tab is in the tab order.
- `aria_label` and `aria_labelledby` land on the tablist, not the root.
- In a strip too wide for its box, the selected tab scrolls into view, also
  when `value` changes from outside.

### You must

- Name the strip with `aria_label` or `aria_labelledby`. Without either it
  warns in debug builds.
- If you remove the focused tab from `options`, move the focus back to the
  strip yourself.

### Example

A settings strip, `Tabs { aria_label: "Settings" }`: Tab lands on the selected
tab, the arrows move and select, and Home and End jump to the first and last
tab.

## Theme defaults

`TabsDefaults` on the theme. Per-size values live in its `sizes` scale.

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

The tabs carry no `data-state`. Style them through `[aria-selected="true"]`
and `[aria-disabled="true"]`.
