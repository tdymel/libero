# Styling

Crate: `libero`
Import: `use libero::sx::{Sx, StaticSx, bp, sx};`
Index: [index.md](index.md) - every other component's markdown page
Description: The `sx` styling builder every component takes - theme values, states, selectors, responsive and container queries, cascade layers and `StaticSx`.

Every component takes the same four styling props: `sx` for CSS, `states` for
your own variants, `class` for a stylesheet you already have, and everything a
`GlobalAttributes` element accepts - `id`, `onclick`, the rest - straight
through to the rendered tag.

`sx()` builds a list of declarations, not an inline style: identical
declarations hash to one `lsx-*` class shared by every element that asked for
it, emitted once into a `<style>` tag. So a thousand rows styled alike cost one
rule, and pseudo-classes and media queries work - neither of which an inline
style can express.

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

A value is a plain string, and the ones the theme knows resolve against it. A
bare color name is shade 6; `1` through `9` are generated from that one hex
value, and `-contrast` is whichever of black or white reads on it. A size word
(`xs` to `xxl`) resolves through whichever scale the property belongs to -
spacing for `padding`/`margin`/`gap`, radius for the `border_radius` family.
Prefix it with a `-` to read the same scale in the negative direction.
Everything else is CSS text, untouched.

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

A theme color given to `background` or `background_color` also tells the focus
rings inside the element which color reads on it. A `var()` or any other CSS
text tells them nothing, so they keep the surrounding one.

## States

Your own variants are not a second class per variant: fold every one into the
same `sx` with `when`, and switch between them with the `states` prop, which
renders a `data-state` attribute. One shared class, per-instance variation - the
same mechanism every Libero component uses for its own size and variant props. A
condition can combine tokens with `&&` and `||`.

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
                .background("grey.2")
                .when("active", sx().background("primary").color("primary-contrast"))
                .when("danger", sx().background("error").color("error-contrast")),
            states: States::new().active("active"),
            "Badge"
        }
    }
}
```

## Selectors

`hover`, `focus` and `focus_visible` nest an `Sx` under that pseudo-class. They
are conveniences over `selector`, which takes any selector text and substitutes
`&` with the class this `Sx` generates - so `&` can sit anywhere in the pattern,
including after an ancestor. A pattern without one is appended, making
`":hover"` and `"&:hover"` the same thing, and a comma list expands to one rule
per part.

```rust,ignore
sx()
    .hover(sx().background("primary.7"))                  // == selector(":hover", ..)
    .selector("& svg", sx().width("18px"))                // a descendant
    .selector("&::before", sx().content("\"*\""))         // a pseudo-element
    .selector("&:not(:last-child)", sx().margin_bottom("sm"))
    .selector(".dark &", sx().background("grey.8"))       // this element, in a context
    .selector("&::before, &::after", sx().display("block"))
```

## Responsive

`breakpoint` nests a whole `Sx` under a `min-width` media query, so the
unnested declarations are the small-screen ones and each breakpoint overrides
upwards. Several blocks apply in the order they are written, so write them
smallest first; a debug build warns when a wider one comes before a narrower
one.

```rust,ignore
sx()
    .flex_direction("column")
    .gap("sm")
    .breakpoint(Size::Sm, sx().flex_direction("row").gap("lg"))
```

When only one property changes, `bp()` puts the breakpoints inside the value
instead of wrapping a block around it:

```rust,ignore
sx().width(bp().sm("480px").lg("720px"))
```

It carries no base value, and a second call to the same property replaces the
first rather than adding to it - so a property that also needs a value below the
smallest breakpoint wants the nested form for the base. The steps apply from the
smallest up, in whatever order they are written.

The six breakpoints are fixed literals rather than theme values - a `@media`
query cannot read a CSS custom property:

| Size | `min-width` |
|---|---|
| `xs` | `36rem` |
| `sm` | `48rem` |
| `md` | `62rem` |
| `lg` | `75rem` |
| `xl` | `88rem` |
| `xxl` | `101rem` |

## Media queries

`media` nests an `Sx` under any `@media` query, passed through verbatim. It is
the general form `breakpoint` is the shorthand of, and the reason it exists is
`prefers-reduced-motion`:

```rust,ignore
sx()
    .transition("transform 200ms ease")
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))
```

Nothing validates the query, so a typo silently matches nothing - the same
tradeoff `when` makes. Nested `media` modifiers fold into a single `and` query,
exactly like nested `breakpoint`s.

**Nest `media` inside the condition, never the condition inside `media`.** A
media query adds no specificity: the rule inside it carries the same selector as
the one outside. A condition appends `[data-state~="open"]`, so it is one class
and one attribute (0-2-0) against a bare class (0-1-0), and the flat form loses
no matter where it sits in the file:

```rust,ignore
// Wrong - the transition still plays under reduced motion.
sx().when("open", sx().transition("transform 200ms ease"))
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))

// Right - both rules are 0-2-0, and the media one comes later.
sx().when(
    "open",
    sx().transition("transform 200ms ease")
        .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
)
```

The same applies to any modifier that changes the selector - `hover`, `selector`
and `when` all do. `breakpoint` and `container_query` do not, so those compose
in either order.

## Container queries

`breakpoint` asks about the viewport, but "does this component have room" is
almost always a question about the box it sits in. The two disagree whenever a
component lives in a column narrower than the window - a sidebar, a grid cell, a
card.

`container` marks an ancestor as a query container and `container_query` asks
about it by name. The name is required: an anonymous container binds the query to
the nearest ancestor container, which picks the wrong one as soon as containers
nest.

```rust,ignore
// The ancestor whose width the answer depends on:
sx().container("demo-card")

// A descendant, asking about it:
sx().width("100%")
    .container_query("demo-card", "(min-width: 640px)", sx().width("388px"))

// Or at a Size's breakpoint:
sx().container_breakpoint("demo-card", Size::Md, sx().width("388px"))
```

`container` emits `container-type: inline-size`, which is `contain: layout style
inline-size`. The element stops being sized by its own contents in the inline
axis, becomes a stacking context, and becomes the containing block for
absolutely positioned descendants - so never mark a shrink-to-fit box as a
container. Like `@media`, the condition cannot read a CSS custom property.

## class and attributes

`sx` is additive, not exclusive: `class` puts your own class names on the same
element for CSS you already have, and any attribute or event a
`GlobalAttributes` element accepts is forwarded to the rendered tag.

```rust,ignore
Box {
    class: "prose",
    id: "intro",
    onclick: move |_| open.set(true),
    "Every component takes these"
}
```

## Cascade layers

Nothing that styles an element is settled by specificity or source order.
Libero emits one `@layer` statement up front, and every rule it writes that
styles an element goes into one of those layers - a later layer beats an earlier
one however weak its selector is, and specificity only decides ties *within* a
layer. The theme's custom properties on `:root` and its `@keyframes` stay
outside it: neither is a cascaded rule, so a layer would only make them harder
to override.

```css
@layer lsx-base, lsx-framework, lsx-user-static, lsx-user-custom;
```

| Layer | What it holds |
|---|---|
| `lsx-base` | The theme's reset and its `body` rules. First, so everything else outranks them. |
| `lsx-framework` | Each component's own styling, and its focus ring. |
| `lsx-user-static` | Your `sx` prop. Beats the component's own styling, always. |
| `lsx-user-custom` | A stylesheet you registered yourself with `use_stylesheet()`. |
| unlayered | Your own CSS files, reached through the `class` prop. Unlayered CSS outranks every layer, so these win outright. |

That last row is the one to remember: a plain stylesheet of your own is not in
the cascade layers at all, and CSS puts unlayered rules above every layered one.
So `class` is the heaviest hammer on the page - reach for `sx` first, and keep
`class` for stylesheets you already own.

Layers rank by first mention, and Libero declares its own when
`LiberoProvider` mounts - after anything already in `<head>`. So a stylesheet
of yours that declares layers has them ranked *below* every `lsx-*` layer, and
a layered `body { margin: 2rem }` of yours still loses to `lsx-base`. Two ways
out, both measured in Chromium:

- Leave the rule unlayered. Unlayered beats every layer, including `lsx-base` -
  which is what moving the reset into a layer bought you.
- Declare the whole order yourself, first in a stylesheet the browser sees
  before Libero's statement. Then you decide where your layers sit:

```css
@layer lsx-base, app-base, lsx-framework, lsx-user-static, lsx-user-custom, app-overrides;
```

That is what `lsx-base` is for: a rank you can sit above without having to
outrank a component's own styling.

A utility framework like Tailwind composes with Libero once you declare the
order. Tailwind v3's utilities are unlayered, so a `class: "mt-4"` outranks
every Libero layer. Tailwind v4 layers everything - `theme`, `base` (its
reset), `components`, `utilities` - and left alone those rank below Libero's:
its reset leaves Libero's components alone, but a utility class on one loses.
Put this first in your CSS, before `@import "tailwindcss"`, so the reset stays
under Libero's components and the utilities win over them:

```css
@layer theme, base, lsx-base, lsx-framework, lsx-user-static, lsx-user-custom, components, utilities;
```

Measured in Chromium on the Button page: with that line every Button kept its
fill and `bg-red-500` won on all 15; without it, `bg-red-500` won on none.
Using Tailwind alongside Libero is unsupported, not forbidden.

## Static sx

An `Sx` written inline is rebuilt, re-hashed and (the first time that exact
content appears) rendered to CSS on every render. When the styling does not
depend on props, hoist it into a `StaticSx` and pass a reference: the builder
runs once per process, the CSS text is rendered once and cached by address, and
the per-render check becomes a pointer comparison instead of a content hash.

```rust
use dioxus::prelude::*;
use libero::components::{Box, States};
use libero::sx::{StaticSx, sx};

static CARD_SX: StaticSx = StaticSx::new(|| {
    sx().padding("md")
        .border_radius("md")
        .background("grey.1")
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

Measured at roughly 210 ns per call site per render - too small to matter in a
page, worth having in a component that a hundred rows mount. Every constant `sx`
inside Libero itself is a `StaticSx` for that reason. If you are building
components on top of Libero, that is the pattern to copy: a static per
component, with the parts that genuinely vary carried by `states` rather than by
a fresh `Sx` per render.

## Props

The four styling props every component takes.

| Prop | Type | Default | Description |
|---|---|---|---|
| `sx` | `Input<Sx>` | - | Declarations compiled to one shared `lsx-*` class. Takes an `Sx` or a `&'static StaticSx`. |
| `states` | `Input<States>` | - | Active state tokens, rendered as `data-state`, matched by `Sx::when`. |
| `class` | `Input<ClassList>` | - | Your own class names, on the same element. Unlayered CSS outranks every Libero layer. |
| `attributes` | `Vec<Attribute>` | - | Any `GlobalAttributes` attribute or event - `id`, `style`, `onclick` - forwarded to the rendered tag. |
