# Pagination

Crate: `libero`
Import: `use libero::components::Pagination;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/pagination/pagination.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A row of page controls in a named nav landmark, with an ellipsis range that never reflows as you click through it.

A row of page controls: a named `<nav>` landmark around a list of real buttons,
with an ellipsis range that never reflows as you click through it. Strictly
controlled - `page` is yours and `onchange` asks for a new one.

It renders buttons, not links. A pagination is state; whether a navigation
happens is not part of its contract, so a content listing that wants shareable
URLs wires its own links around this component.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Pagination;

#[component]
fn Demo() -> Element {
    let mut page = use_signal(|| 1u32);

    rsx! {
        Pagination {
            total: 42,
            page: page(),
            onchange: move |next| page.set(next),
            aria_label: "Search results",
            with_edges: true,
        }
    }
}
```

`aria_label` is required: two paginations on one page have to be
distinguishable, and a landmark with no name is not.

## The range

An ellipsis never stands for exactly one page. Hiding `9` behind a gap costs the
same width as printing it, so the gap is only drawn where it saves something.
That also fixes the rendered width at `2·siblings + 2·boundaries + 3`, which is
why the strip does not reflow while you click through it.

```text
total  siblings  boundaries  page   rendered
    7         1           1     4   1 2 3 4 5 6 7
   10         1           1     1   1 2 3 4 5 … 10
   10         1           1     5   1 … 4 5 6 … 10
   10         1           1     7   1 … 6 7 8 9 10
   20         2           1    10   1 … 8 9 10 11 12 … 20
   20         1           2    10   1 2 … 9 10 11 … 19 20
   11         0           1     6   1 … 6 … 11
```

`boundaries: 0` is clamped to 1 - leading dots with nothing outside them hide a
page in order to show a gap.

The arithmetic is public, so a caller drawing a custom strip can reuse it rather
than re-deriving the edge cases:

```rust
use libero::components::{pagination_range, PaginationItem};

#[component]
fn Demo() -> Element {
    let items = pagination_range(42, 7, 1, 1);

    rsx! {
        for item in items {
            match item {
                PaginationItem::Page(n) => rsx! { button { "{n}" } },
                PaginationItem::Ellipsis => rsx! { span { "…" } },
            }
        }
    }
}
```

## Accessible names

The current page is named `Page 4` and every other `Go to page 4`.
`aria-current="page"` already says "current", so repeating "go to" on the page
you are on would be a lie. It is never `aria-current="true"`. The ellipsis is
`aria-hidden` and not focusable: it is a gap, not a control.

Every string lives in `theme.pagination_labels`, which is English by default and
swapped whole for a locale. That struct assumes the number goes last, which is
wrong in plenty of languages, so `label` is the escape hatch. It sees the five
named controls and never the ellipsis, so an exhaustive match has no dead
branch.

```rust
use dioxus::prelude::*;
use libero::components::{Pagination, PaginationLabel};

#[component]
fn Demo() -> Element {
    let mut page = use_signal(|| 1u32);

    rsx! {
        Pagination {
            total: 42,
            page: page(),
            onchange: move |next| page.set(next),
            aria_label: "Suchergebnisse",
            label: |label: PaginationLabel| match label {
                PaginationLabel::Page { number, current: true } => format!("Seite {number}, aktuell"),
                PaginationLabel::Page { number, .. } => format!("Seite {number}"),
                PaginationLabel::First => "Erste Seite".to_string(),
                PaginationLabel::Previous => "Vorherige Seite".to_string(),
                PaginationLabel::Next => "Nächste Seite".to_string(),
                PaginationLabel::Last => "Letzte Seite".to_string(),
            },
        }
    }
}
```

## Keyboard

| Key | Does |
|---|---|
| `Tab` / `Shift+Tab` | Move between controls. Every control is a real tab stop |
| `Enter` | Activate |
| `Space` | Activate |

No roving tabindex, deliberately. `Tree` and `Tabs` rove because each is one
composite widget; this is a short row of independent buttons, and roving would
make Tab skip past the whole strip.

## Focus when a control disables under you

Clicking previous until you reach page 1 disables the button your finger is on.
Nothing is removed, so focus is simply left on a disabled control, which
browsers drop to the document. Focus goes to the current page's button instead -
which is why that button stays enabled rather than being disabled as the page
you are already on.

The repair needs the platform to find the focused element and search a subtree.
Where it cannot, on the webview floor, nothing moves and nothing else changes.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `total` | `u32` | required | Page count. `0` renders nothing at all |
| `page` | `u32` | required | Current page, 1-based and clamped into range |
| `onchange` | `EventHandler<u32>` | - | Asks for a new page. Without it the page can never change, and the component warns |
| `aria_label` | `String` | required | Names the `<nav>` landmark |
| `siblings` | `u8` | `1` | Pages either side of the current one |
| `boundaries` | `u8` | `1` | Pages pinned at each end. `0` is clamped to 1 |
| `size` | `Size` | `md` | Control box and font size |
| `radius` | `Size` | `sm` | Corner radius, independent of `size` |
| `color` | `ThemeAwareValue` | `primary` | Fill of the current page |
| `disabled` | `bool` | `false` | Disables every control at once |
| `with_controls` | `bool` | `true` | Previous and next |
| `with_edges` | `bool` | `false` | First and last |
| `label` | `Callback<PaginationLabel, String>` | - | Overrides every accessible name |

Like every component, `Pagination` also takes `sx`, `class`, `style`, `states`
and any extra HTML attributes, all of which land on the `<nav>`.

## Theme defaults

Two structs, because geometry and language are changed by different people for
different reasons - the `DateDefaults` arrangement.

`PaginationDefaults` on `theme.pagination`:

| Field | Type | Description |
|---|---|---|
| `size` / `radius` | `Size` | Defaults for the matching props |
| `color` | `Color` | Current-page fill. The readable text is its `-contrast` twin |
| `siblings` / `boundaries` | `u8` | Range shape |
| `gap` | `Size` | Space between controls |
| `control_size` | `Sizes<u16>` | Control box per size step, in px |
| `font_size` | `Sizes<u16>` | Font size per size step, in px |
| `border` | `&'static str` | Control border colour |

`PaginationLabels` on `theme.pagination_labels`, English by default and swapped
whole for a locale:

| Field | Default |
|---|---|
| `nav_label` | `Pagination` |
| `page_label` | `Go to page` |
| `current_page_label` | `Page` |
| `previous_label` / `next_label` | `Go to previous page` / `Go to next page` |
| `first_label` / `last_label` | `Go to first page` / `Go to last page` |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-pagination-control-size-{xs..xxl}` | Control box, per size step |
| `--lsx-pagination-font-size-{xs..xxl}` | Font size, per size step |
| `--lsx-pagination-gap` | Space between controls |
| `--lsx-pagination-border` | Control border colour |
| `--lsx-pagination-active-background` | Current-page fill |
| `--lsx-pagination-active-color` | Text on the current page |

## Data attributes

| `data-state` token | When |
|---|---|
| `size-{xs..xxl}` | The resolved `size`, on every control |
| `current` | The control for the current page |
| `disabled` | A control that cannot be activated |
