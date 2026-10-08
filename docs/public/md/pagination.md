# Pagination

Crate: `libero`
Import: `use libero::components::Pagination;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/pagination/pagination.rs>
Index: [index.md](index.md) lists every other page
Description: A row of page buttons in a named nav landmark, with an ellipsis that keeps the row the same width.

A row of page buttons in a named `<nav>` landmark. The ellipsis keeps the row
the same width as you click through it. You own `page`, and `onchange` asks for
a new one.

It renders buttons, not links, so open in a new tab and crawlable page URLs
are not available. A listing that wants shareable URLs navigates in
`onchange`.

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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `total` | `u32` | required | Page count. `0` renders nothing. |
| `page` | `u32` | required | The current page, 1-based and clamped into range. |
| `onchange` | `EventHandler<u32>` | - | Asks for a new page. Without it the page never changes, and the component warns. |
| `aria_label` | `String` | required | Names the `<nav>` landmark, so two paginations on one page can be told apart. |
| `siblings` | `u8` | `1` | Pages on each side of the current one. The row is always `2·siblings + 2·boundaries + 3` items wide, and an ellipsis never stands for a single page. |
| `boundaries` | `u8` | `1` | Pages pinned at each end. `0` counts as 1. |
| `size` | `Size` | `md` | Control box and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `color` | `ThemeAwareValue` | `primary` | Fill of the current page. A theme color gives its text the matching `-contrast` shade, a literal color black or white. |
| `disabled` | `bool` | `false` | Disables every control. |
| `with_controls` | `bool` | `true` | Shows the previous and next controls. |
| `with_edges` | `bool` | `false` | Shows the first and last controls. |
| `label` | `Callback<PaginationLabel, String>` | - | Overrides every accessible name. Runs during render, so it can read a locale. |
| `parts` | `Parts<PaginationPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Pagination` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the `<nav>`.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `PaginationPart::List` | `list` | The list of controls. |
| `PaginationPart::Page` | `page` | A page button. The current one has `aria-current="page"`. |
| `PaginationPart::Arrow` | `arrow` | The first, previous, next and last buttons. |
| `PaginationPart::Ellipsis` | `ellipsis` | The `…` between page ranges. |

## Accessibility

### Libero handles

- Every control is a button and a tab stop, except an arrow at its end, which is
  a natively disabled button. Under `disabled` every control is `aria-disabled`
  and stays a tab stop, so focus is not lost.
- An arrow that disables itself on click, such as next on the last page, hands
  focus to the current page.
- The page names come from the [localization](localization.md)'s
  `PaginationLabels`, and `label` overrides them.

### You must

- Keep `theme.pagination.gap` above zero: at `xs` the controls are 22px and
  meet the 24px target size only through the gap.

### Example

A results pager: each page number and enabled arrow is a button and a tab stop, and
the next arrow reads "Go to next page" from `PaginationLabels` instead of a
bare arrow.

## Theme defaults

`PaginationDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size`, `radius` | `Size` | Defaults for the matching props. |
| `color` | `Color` | Current-page fill. Its text takes the `-contrast` color. |
| `siblings`, `boundaries` | `u8` | Defaults for the matching props. |
| `gap` | `Size` | Space between controls. |
| `control_sizes` | `Sizes<u16>` | Control box per size step, in px. |
| `font_sizes` | `Sizes<u16>` | Font size per size step, in px. |
| `border` | `&'static str` | Control border colour. |

`PaginationLabels` on `Localization::pagination`, English by default. The
`<nav>`'s own name comes from `aria_label`, so you localise it yourself.

| Field | Default |
|---|---|
| `page` | `Go to page {n}` |
| `current_page` | `Page {n}` |
| `previous` | `Go to previous page` |
| `next` | `Go to next page` |
| `first` | `Go to first page` |
| `last` | `Go to last page` |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-pagination-control-size-{xs..xxl}` | Control box, per size step. |
| `--lsx-pagination-font-size-{xs..xxl}` | Font size, per size step. |
| `--lsx-pagination-gap` | Space between controls. |
| `--lsx-pagination-border` | Control border colour. |
| `--lsx-pagination-active-background` | Current-page fill. |
| `--lsx-pagination-active-color` | Text on the current page. |

## Data attributes

| `data-state` token | When |
|---|---|
| `size-{xs..xxl}` | The `size` in effect, on every control. |
| `current` | The control for the current page. |
| `disabled` | A control that cannot be activated. |
