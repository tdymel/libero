# Philosophy

Crate: `libero`
Index: [index.md](index.md) - every other component's markdown page
Description: The four principles behind libero in order of priority (developer experience, accessibility, batteries included, simple yet modern) and where each shows in the code.

Libero is a component library for Dioxus. Four principles decide what goes into
it and how it is shaped, listed here in order of priority. Each section says what
the principle means in libero and where you can see it in the code.

## Developer experience first

AI writes a lot of code now, and that makes the API matter more, not less.
Someone still reads the result, reviews it and changes it next month. A small API
that is hard to misuse is easy for a person to review and easy for a model to get
right the first time.

So libero leans on the type system instead of conventions. `Tabs` over an enum
gets its tab strip from `#[derive(Options)]`, and its `panel` is a `match`, so
the compiler rejects a tab without a panel. A `Table` column is
`column("Age").value(|p| p.age)`, and the cell's type alone makes it sort
numerically and align right. Props are typed as well, but the common ones also
take a string: `size: "sm"` and `size: Size::Sm` are the same prop.

Composition works the same way. Every field has the same five slots: label,
description, control, helper text and validation message. A `Fieldset` groups
fields into one value, a `Form` holds the whole value, and typed paths from
`#[derive(Fields)]` tie each layer to your own structs. Styling is one builder,
`sx()`, on every component. Setup is one `LiberoProvider` at the root.

Every docs page also has a plain markdown copy with complete examples, the "View
as markdown" link at the top. Those examples are compiled by the test suite, so
the code you or your assistant copies out of them builds.

## Accessibility second

Accessibility is not only for screen reader users. Keyboard support helps anyone
who would rather not reach for the mouse. A state that does not rely on colour
still reads on a dim laptop screen in the sun. Motion that stops on request helps
anyone it makes dizzy.

Where WAI-ARIA, the APG patterns or WCAG have an opinion, libero follows it, and
a deviation is what needs a reason. Menus, trees, tabs and listboxes take the
keyboard model the APG describes for them, and each component page lists its
keys. WCAG 2.2 AA is the target: 4.5:1 contrast for text and 3:1 for borders and
icons are the numbers we measure against.

A pressed, selected or current control never differs by colour alone. It carries
a short 2px line in its own text colour. Under `prefers-reduced-motion` a
`Carousel` opens paused and an `Indicator` stops its ping. A component that needs
an accessible name and has none warns you in a debug build. Every string a
component says on its own, like "Go to page 3" on a `Pagination` button, comes
from a `Localization`, with English and German built in.

Forced colours (Windows High Contrast) are supported only in part. The
[Accessibility](accessibility.md) page says what holds and what does not.

## Batteries included

Building an app should not start with a hunt for a date picker. Libero has more
than 100 components: layout, typography, navigation, feedback, data display, and
a form stack from `TextField` to `PhoneField`, `DatePicker`, `ColorPicker` and
`FileField`, with validation and a focused error summary in `Form`.

The overlays are there too. A modal and a drawer open from a hook
(`use_modal`, `use_drawer`) and can hand a result back. `Popover`, `Tooltip`,
`HoverCard`, `Menu` and a command palette cover the rest, and
`use_notifications()` shows a message from anywhere. Around them sit the hooks
the components are built from (`use_drag`, `use_clipboard`), `Virtualize` for
long lists, and `CodeBlock` with 30 grammars that you pay for only when you turn
them on.

The same components run on the web and natively through Blitz. Anything that
reaches the machine goes through one trait in `libero::platform`, and a
capability the renderer lacks is `None`: absent, not broken. You branch once, and
the rest of your code stays the same.

## Simple yet modern

The default theme aims to look clean and stay out of the way, so it fits most
apps without a fight. When it does not, theming is plain Rust. A theme is one
struct of colours, scales and per-component defaults, and you change what you
need with struct update syntax over `Theme::DEFAULT`.

The provider emits the theme once as CSS custom properties, light and dark
together. Switching the scheme is one attribute on the root, correct on first
paint, and `ColorSchemeButton` does it for you. Styling uses current CSS without
a CSS framework: container queries in `sx`, and cascade layers, so your own
unlayered CSS beats every libero rule.
