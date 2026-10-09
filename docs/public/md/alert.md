# Alert

Crate: `libero`
Import: `use libero::components::Alert;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/alert.rs>
Index: [index.md](index.md) lists every other page
Description: A tinted surface for something the reader has to know, with a title, an optional icon and close button, and a role that follows its color.

A tinted surface for something the reader has to know, such as an error, a
warning or a note. It takes no focus and does not close on Escape, because it
is not an overlay. `Form`'s error summary is an `Alert` with `color: "error"`
(see [Form](form.md)).

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

With `onclose` set, a close button shows. The alert does not hide itself, so
unmount it:

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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `title` | `String` | - | The heading and the alert's accessible name. Text only. |
| `icon` | `Option<Element>` | - | A leading icon of your own, hidden from screen readers. |
| `color` | `ThemeAwareValue` | `info` | The tint. A theme color name or any CSS color. `error` and `warning` make the role `alert`, the rest `status`. |
| `variant` | `Variant` | `tonal` | Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. |
| `radius` | `ThemeAwareValue` | `md` | Corner radius, a size step from `xs` to `xxl`, or any CSS, e.g. `radius: "0"`. |
| `onclose` | `EventHandler<()>` | - | Shows the close button and fires when it is pressed. Unmount the alert to close it. |
| `close_label` | `String` | `common.close` | The close button's accessible name. Unset, the localization's `common.close`, "Close" in English. |
| `parts` | `Parts<AlertPart>` | - | Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(AlertPart::Title, sx().font_weight("700"))`. |
| `children` | `Element` | required | The message, read as the alert's description. |

Like every component, `Alert` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `AlertPart::Icon` | `icon` | The icon wrapper. |
| `AlertPart::Body` | `body` | The column holding the title and the message. |
| `AlertPart::Title` | `title` | The title. |
| `AlertPart::Message` | `message` | The message. |
| `AlertPart::Close` | `close` | The close button. |

## Accessibility

### Libero handles

- An `error` or `warning` color renders `role="alert"`, which interrupts a
  screen reader. Every other color renders the polite `role="status"`. Your
  own `role` replaces either.
- The icon is hidden from screen readers.

### You must

- Say the severity in the title or the message too, as the icon is not read.
- Prefer `tonal` or `filled` for an error: `outlined` has no tint.
- Move the focus somewhere sensible in `onclose`: closing removes the focused
  close button.
- Mount the Alert after the page loads to have it announced: one present at
  first paint is read only when the reader reaches it.

### Example

A failed save, `Alert { color: "error", title: "Error: not saved" }` mounted
after the click: a screen reader interrupts to read it, and the title says the
severity in words.

## Theme defaults

`AlertDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | `Tonal`. |
| `color` | `&'static str` | `info`. A palette colour name, read to default the `color` prop. |
| `radius` | `Size` | `Md`, the same step as `paper.radius`. |
| `padding` | `Size` | `Md`, on the spacing scale. |
| `gap` | `Size` | `Md`, from icon to text to close button. |
| `body_gap` | `Size` | `Xs`, from title to message. |
| `icon_size` | `&'static str` | `20px`, the icon slot's width and height. |

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
| `--lsx-alert-contrast` | Text color on that color, for `filled`. For a literal CSS color, black or white, whichever reads on it. |
| `--lsx-alert-container` | The tint of `tonal`. |
| `--lsx-alert-on-container` | Text color on that tint. |

## Data attributes

| Attribute | On |
|---|---|
| `data-state="filled"` / `tonal` / `elevated` / `outlined` / `standard` | The root, for the `variant` in effect. |
| `data-slot` | Each part, see [Style API](#style-api). |
