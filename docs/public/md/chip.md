# Chip

> Part of the libero docs. [index.md](index.md) lists every component's
> markdown page - read it first to find the rest.

A compact token. With `onchange` it is a real checkbox - a visually hidden
`input` plus a `label` - so it gets checked semantics, Space-to-toggle and
focus for free. With `onclick` or `to` it is a button or a link instead.
Without any of them it is a plain `span`.

## Properties

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `variant` | `ButtonVariant` | `outlined` | The unselected look; a checked chip is always filled. |
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `xl` | Corner radius, independent of `size`. |
| `checked` | `bool` | | Strictly controlled selection state - pair it with `onchange`. |
| `disabled` | `bool` | `false` | Disables interaction and dims the chip. |
| `onchange` | `EventHandler<bool>` | | Called with the value `checked` should take next. Its presence makes the chip a real checkbox. |
| `onclick` | `EventHandler<MouseEvent>` | | A plain action; its presence makes the chip a `button`. |
| `to` | `NavigationTarget` | | Renders a router-aware link instead. Takes precedence over `onclick`. |
| `target` | `String` | | Link target, e.g. `_blank`. Only with `to`. |
| `children` | `Element` | | Text and `Icon` only - a `label` hijacks clicks on nested controls. |

## Selectable

Strictly controlled: `checked` drives the look, `onchange` reports the value it
should take next. A checked chip is filled whatever its `variant`, so `variant`
describes the unselected state.

```rust
Chip {
    checked: selected(),
    onchange: move |next| selected.set(next),
    "rust"
}
```

## Actions and links

`onclick` makes the chip a `button`, `to` a router-aware link. Neither combines
with `onchange`.

```rust
Chip { onclick: move |_| clicks += 1, "Clicked {clicks}x" }
Chip { to: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
```
