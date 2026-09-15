# Alert

Crate: `libero`
Import: `use libero::components::Alert;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/alert.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A tinted surface for something the reader has to know - a title that names it, an optional icon and close button, and a role picked by its color that you can replace.

A tinted surface for something the reader has to know: an error summary, a
warning, a note. An `error` or `warning` color renders `role="alert"`, every
other color the polite `role="status"`; your own `role` replaces either. It takes no focus and does not close on Escape, because it is not an
overlay.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Alert;

#[component]
fn Demo() -> Element {
    rsx! {
        Alert { title: "Card expiring",
            "Your card ends 09/26. Update it before the next invoice."
        }
    }
}
```

Severity is yours to state. The default color is `info`; pass `error`,
`warning` or `success` when the message is one.

```rust
use dioxus::prelude::*;
use libero::components::Alert;

#[component]
fn Demo() -> Element {
    let mut dismissed = use_signal(|| false);

    rsx! {
        if !dismissed() {
            Alert {
                color: "warning",
                title: "Card expiring",
                icon: rsx! { WarningGlyph {} },
                onclose: move |_| dismissed.set(true),
                "Your card ends 09/26. Update it before the next invoice."
            }
        }
    }
}
#
# #[component] fn WarningGlyph() -> Element { rsx! {} }
```

`onclose` is what shows the close button. There is no separate
`with_close_button`: a close button that does nothing is not something you can
ask for. Closing is yours - the alert does not hide itself, you unmount it.

The library ships no icon set, so `icon` takes your own glyph. An empty `icon`
takes no room.

### In a form

`Form`'s error summary is an `Alert` with `color: "error"`. You do not render it
yourself - a blocked submit shows it, moves focus to it, and each line focuses
its field. See [Form](form.md).

## Accessibility

- **The role follows `color`.** `error` and `warning` render `role="alert"`,
  which interrupts a screen reader; `info`, `success`, the other palette colors
  and a literal CSS color render `role="status"`, which waits its turn. Your
  own `role` wins over either.
- **The icon is `aria-hidden`**, so the severity has to be in the title or the
  message too.
- **An alert never takes focus on its own.** Focus it from outside where that
  matters, as `Form` does with its summary. `Escape` does not close it.
- **`outlined` carries no tint**, only a border, which is quiet for an error.
  Prefer `tonal` or `filled` where the severity must be seen. On `filled` the
  close button's focus ring takes the text colour on the fill as its halo:
  white on `info` is about 4.5:1.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `title` | `String` | - | The heading, and the accessible name through `aria-labelledby`. |
| `icon` | `Element` | - | A leading glyph, rendered `aria-hidden`. |
| `color` | `ThemeAwareValue` | `info` | The tint; a theme color name or a literal CSS color. `error` and `warning` make the role `alert`, the rest `status`. |
| `variant` | `Variant` | `tonal` | Chrome, shared with `Button` and `Badge`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. No hover response - an alert is not a target. |
| `radius` | `Size` | `md` | A step on the radius scale. Anything else goes through `sx`. |
| `onclose` | `EventHandler<()>` | - | Shows the close button, and fires when it is pressed. |
| `close_label` | `String` | `Close` | The close button's accessible name. |
| `children` | `Element` | - | The message, and the description through `aria-describedby`. |

Like every component, `Alert` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`AlertDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | `Tonal`. |
| `color` | `&'static str` | `info`. A palette colour name, read to default the `color` prop. |
| `radius` | `Size` | `Md`, the same step as `paper.radius`. |
| `padding` | `Size` | `Md`, on the spacing scale. |
| `gap` | `Size` | `Md` - icon to text to close button. |
| `body_gap` | `Size` | `Xs` - title to message. |
| `icon_size` | `&'static str` | `20px`, the icon slot's width and height. |
| `close_label` | `&'static str` | `Close`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-alert-radius` | The theme's corner radius. |
| `--lsx-alert-radius-override` | Set by the `radius` prop; wins over the theme's. |
| `--lsx-alert-padding` | From `AlertDefaults::padding`. |
| `--lsx-alert-gap` | From `AlertDefaults::gap`. |
| `--lsx-alert-body-gap` | From `AlertDefaults::body_gap`. |
| `--lsx-alert-icon-size` | From `AlertDefaults::icon_size`. |
| `--lsx-alert-color` | Resolved `color`. |
| `--lsx-alert-contrast` | Text color on that color, for `filled`. Unset for a literal CSS color. |
| `--lsx-alert-container` | The tint of `tonal`. |
| `--lsx-alert-on-container` | Text color on that tint. |

## Data attributes

| Attribute | On |
|---|---|
| `data-state="filled"` / `tonal` / `elevated` / `outlined` / `standard` | The root, for the `variant` in effect. |
| `data-slot="icon"` | The icon wrapper. |
| `data-slot="body"` | The column holding the title and the message. |
| `data-slot="title"` | The title. |
| `data-slot="message"` | The message. |
| `data-slot="close"` | The close button. |
