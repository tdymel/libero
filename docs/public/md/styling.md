# Styling

Crate: `libero`
Import: `use libero::sx::{Sx, StaticSx, bp, sx};`
Index: [index.md](index.md) lists every other page
Description: The `sx` styling builder every component takes: theme values, states, selectors, responsive, media and container queries, cascade layers and the `parts` Style API.

Every component takes the same styling props. `sx()` is not an inline style.
Identical declarations share one `lsx-*` class, emitted once, so a thousand rows
styled alike cost one rule, and hover and media queries work.

| Prop | For |
|---|---|
| `sx` | CSS, compiled to one shared class. |
| `states` | Your own variants, matched by `when`. |
| `class` | A stylesheet you already have. |
| `id`, `onclick`, ... | Any global attribute or event, passed to the rendered tag. |

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Box;
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx()
                .background("primary")
                .color("primary-contrast")
                .padding("md")
                .border_radius("md"),
            "Styled with sx"
        }
    }
}
```

## Theme values

A value is a plain string. The ones the theme knows resolve against it.

| Value | Resolves to |
|---|---|
| `"primary"` | The role's base color, shade 6. |
| `"primary.1"` to `"primary.9"` | Shades generated from it. |
| `"primary-contrast"` | Black or white, whichever reads on it. |
| `"xs"` to `"xxl"` | A step of the property's scale: spacing for `padding`, `margin` and `gap`, radius for the `border_radius` family. |
| `"-sm"` | The same step, negated. |
| anything else | CSS text, untouched. |

```rust,ignore
sx()
    .background("primary")        // shade 6, the base hex
    .border_color("primary.2")    // shades 1-9, generated from it
    .color("primary-contrast")    // black or white, whichever reads on it
    .padding("md")                // the theme's spacing scale
    .border_radius("lg")          // the theme's radius scale
    .margin_top("-sm")            // the same scale, negated
    .width("240px")               // anything else is CSS text, untouched
```

A theme color given to `background` also tells the focus rings inside the
element which color reads on it. A `var()` tells them nothing.

CSS text reaches the stylesheet unescaped, and so do selectors and media
queries. Never build them from user text: a `}` or `</style>` in it ends the
rule or the style element.

## States

Fold every variant into one `sx` with `when`, and pick one with the `states`
prop, which renders a `data-state` attribute. Every libero component handles its
own size and variant this way. A condition can combine tokens with `&&` and
`||`.

```rust
use dioxus::prelude::*;
use libero::components::{Box, States};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx()
                .padding("sm")
                .border_radius("xl")
                .background("muted.2")
                .when("active", sx().background("primary").color("primary-contrast"))
                .when("danger", sx().background("error").color("error-contrast")),
            states: States::new().active("active"),
            "Badge"
        }
    }
}
```

## Selectors

`selector` takes any selector text, with `&` standing for the element. A pattern
without one is appended, and a comma list expands to one rule per part.
`hover`, `focus` and `focus_visible` are shorthands for it.

```rust,ignore
sx()
    .hover(sx().background("primary.7"))                  // == selector(":hover", ..)
    .selector("& svg", sx().width("18px"))                // a descendant
    .selector("&::before", sx().content("\"*\""))         // a pseudo-element
    .selector("&:not(:last-child)", sx().margin_bottom("sm"))
    .selector(".dark &", sx().background("muted.8"))       // this element, in a context
    .selector("&::before, &::after", sx().display("block"))
```

## Responsive

Write the small screen first, then override upwards with `breakpoint`, smallest
first; a debug build warns about the wrong order. For a single property, `bp()`
puts the steps inside the value. It carries no base value, so set that outside
`bp()`.

```rust,ignore
sx()
    .flex_direction("column")
    .gap("sm")
    .breakpoint(Size::Sm, sx().flex_direction("row").gap("lg"))
```

```rust,ignore
sx().width(bp().sm("480px").lg("720px"))
```

The breakpoints are fixed, because a `@media` query cannot read a CSS custom
property:

| Size | `min-width` |
|---|---|
| `xs` | `36rem` |
| `sm` | `48rem` |
| `md` | `62rem` |
| `lg` | `75rem` |
| `xl` | `88rem` |
| `xxl` | `101rem` |

## Media and container queries

`media` nests styles under any `@media` query, passed through verbatim, mostly
for reduced motion. Nothing validates the query.

```rust,ignore
sx()
    .transition("transform 200ms ease")
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))
```

Nest `media` inside a `when`, never the other way round. A media query adds no
specificity and a state adds some, so the flat form loses. The same goes for
`hover` and `selector`.

```rust,ignore
// Wrong: the transition still plays under reduced motion.
sx().when("open", sx().transition("transform 200ms ease"))
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))

// Right: both rules are 0-2-0, and the media one comes later.
sx().when(
    "open",
    sx().transition("transform 200ms ease")
        .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
)
```

`container_query` asks about a named ancestor instead of the window, for a
component in a sidebar, a grid cell or a card. The name is required, because an
anonymous query binds to the nearest container. `container` emits
`container-type: inline-size`: the element no longer takes its width from its
content, so never make a shrink-to-fit box one.

```rust,ignore
// The ancestor whose width the answer depends on:
sx().container("demo-card")

// A descendant, asking about it:
sx().width("100%")
    .container_query("demo-card", "(min-width: 640px)", sx().width("388px"))

// Or at a Size's breakpoint:
sx().container_breakpoint("demo-card", Size::Md, sx().width("388px"))
```

## class and attributes

`class` adds your own class names beside `sx`, and every other attribute or
event goes to the rendered tag.

```rust,ignore
Box {
    class: "prose",
    id: "intro",
    onclick: move |_| open.set(true),
    "Every component takes these"
}
```

## Cascade layers

Libero writes its rules into CSS layers. A later layer wins whatever its
selectors, so your `sx` always beats a component's own styling. The theme's
custom properties and `@keyframes` stay outside the layers.

```css
@layer lsx-base, lsx-framework, lsx-user-static, lsx-user-custom;
```

| Layer | What it holds |
|---|---|
| `lsx-base` | The theme's reset and its `body` rules. |
| `lsx-framework` | Each component's own styling. |
| `lsx-user-static` | Your `sx` prop. Beats the component's own styling. |
| `lsx-user-custom` | Stylesheets registered with `use_stylesheet()`. |
| unlayered | Your own CSS files, through `class`. Beats every layer. |

A stylesheet of your own is in no layer, so `class` beats everything. Reach for
`sx` first.

Layers rank by first mention, and libero declares its own when `LiberoProvider`
mounts, so layers your stylesheet declares rank below every `lsx-*` layer. To
rank them among libero's, declare the whole order yourself, first in a
stylesheet the browser sees before libero's:

```css
@layer lsx-base, app-base, lsx-framework, lsx-user-static, lsx-user-custom, app-overrides;
```

Declaring the order also makes Tailwind v4 work: its reset under libero's
components, its utilities over them. Put this before `@import "tailwindcss"`
(measured in Chromium; unsupported, not forbidden):

```css
@layer theme, base, lsx-base, lsx-framework, lsx-user-static, lsx-user-custom, components, utilities;
```

## Static sx

Styling that doesn't depend on props can live in a `StaticSx`, built once per
process instead of on every render. Every constant `sx` inside libero is one,
and your own components should copy that.

```rust
use dioxus::prelude::*;
use libero::components::{Box, States};
use libero::sx::{StaticSx, sx};

static CARD_SX: StaticSx = StaticSx::new(|| {
    sx().padding("md")
        .border_radius("md")
        .background("muted.1")
        .when("selected", sx().background("primary.1"))
});

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: &CARD_SX,
            states: States::new().active("selected"),
            "One class, built once"
        }
    }
}
```

## Style API

`sx` styles a component's root. To reach an element inside it, a multi-part
component names its parts in an enum, such as `AlertPart`, and takes a `parts`
prop keyed by it. The component's page lists its parts in a "Style API" tab.

- Each part carries a `data-slot` attribute. The names are a stable contract,
  so your own CSS can match `[data-slot='title']` too.
- A part is matched as a child of the root, through each part above it
  (`& > [data-slot='body'] > [data-slot='title']`), never as any descendant. A
  component of the same kind nested inside keeps its own styles.
- `parts` compiles into the root's `sx` class, so it beats the component's own
  styling like `sx` does. Where the instance `sx` sets the same property on the
  same part, `sx` wins.
- `StaticParts` builds the parts once per process, as `StaticSx` does for `sx`.
- Parts a component renders in a portal (a `Dialog`, a `Menu`, a `Tooltip`)
  sit outside its root, so `parts` cannot reach them. A field's portaled
  dropdown, such as a `Select`'s list, takes its own `dropdown_parts` prop
  instead.

```rust
use dioxus::prelude::*;
use libero::components::{Alert, AlertPart, Parts, StaticParts};
use libero::sx::sx;

// Built once, shared by every alert that passes it.
static QUIET: StaticParts<AlertPart> =
    StaticParts::new(|| Parts::new().part(AlertPart::Message, sx().color("muted.7")));

#[component]
fn Demo() -> Element {
    rsx! {
        Alert {
            title: "Saved",
            parts: Parts::new()
                .part(AlertPart::Title, sx().font_weight("700"))
                .part(AlertPart::Close, sx().color("error.6")),
            "Your changes are live."
        }
        Alert { title: "Synced", parts: &QUIET, "Nothing to do." }
    }
}
```

## Props

The four styling props every component takes.

| Prop | Type | Default | Description |
|---|---|---|---|
| `sx` | `Input<Sx>` | - | Declarations compiled to one shared `lsx-*` class. Takes an `Sx` or a `&'static StaticSx`. |
| `states` | `Input<States>` | - | Active state tokens, rendered as `data-state`, matched by `Sx::when`. |
| `class` | `Input<ClassList>` | - | Your own class names, on the same element. Unlayered CSS outranks every Libero layer. |
| `attributes` | `Vec<Attribute>` | - | Any `GlobalAttributes` attribute or event (`id`, `style`, `onclick`), forwarded to the rendered tag. |
