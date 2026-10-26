# Share

Crate: `libero`
Import: `use libero::components::Share;`
Feature: `share` (off by default, turns on `action-icon`, `icon`, `button`, `use-share`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/share.rs>
Index: [index.md](index.md) lists every other page
Description: A button that opens the system share sheet, or copies the link where there is none, and says "Copied".

A button that opens the system share sheet: the Web Share API on the web and in
desktop and iOS WebViews. Where there is none (Blitz, most Linux browsers), a
press copies the link, the icon turns into a check and a screen reader hears
"Copied". Icon-only by default; `children` make it a labelled `Button`.

For a share control of your own, build on `use_share()`: `share(ShareData)`
opens the sheet (call it in the press handler: the browser allows it only
inside a user action), `outcome()`, `shared()`, `copied()` and `failed()`
follow once it settled, `pending()` says one is still open (a press meanwhile
is ignored), and `reset()` clears them. `libero::platform::share` is the same
without the copy fallback.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Share;

#[component]
fn Demo() -> Element {
    rsx! {
        Share {
            url: "https://libero-ui.dev",
            title: "Libero",
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `title` | `String` | - | A title, used as the subject where the target has one. |
| `text` | `String` | - | A message. |
| `url` | `String` | - | The link. Where no share sheet exists, a press copies it, else `text`, else `title`. |
| `files` | `Vec<ShareFile>` | - | Files (name, MIME type, bytes), shared where the target takes them: on the web where `canShare` takes them, on Android by name from the app's cache (kept a day). Elsewhere the title, text and link still go. |
| `variant` | `Variant` | - | Visual style, as on `ActionIcon` or `Button`. Icon-only with `color` also unset, the button draws no chrome of its own. |
| `color` | `ThemeAwareValue` | - | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `md` | Button size. With `children`, only a size word applies. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `aria_label` | `String` | - | The button's name, such as "Share this article". Unset, the name is the localization's `share`, or the `children` text. |
| `label` | `String` | - | Describes what a press shares, read after the name, such as "Link to this page". |
| `disabled` | `bool` | `false` | Disables and dims the button. |
| `onshare` | `EventHandler<ShareOutcome>` | - | Called with the outcome once the share settled: `Shared`, `Copied`, `Cancelled`, `Unsupported` or `Failed(reason)`. `Cancelled` also covers a sheet with no target to pick: the browser reports both the same way. Android opens the system share sheet. |
| `children` | `Element` | - | A visible label: renders a `Button` with the share glyph instead of an icon-only button. |

Like every component, `Share` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the button.

The words are `ShareLabels` in the [localization](localization.md): `share`
(the icon-only name), `copied` and `share_failed` (what the status says). The
glyph is the `IconSlot::Share` slot of an [IconProvider](icon_provider.md).

## Accessibility

### Libero handles

- The button keeps its focus through a share. The sheet is the system's own
  dialog and announces itself, so the status stays silent.
- Where no share sheet exists, an always-mounted `role="status"` region says
  "Copied" once the link is on the clipboard, or that sharing failed, and the
  icon turns into a check or an alert. A cancelled sheet says nothing.
- The icon and the status reset when the pointer or the focus leaves.
- Unset, the icon-only button's `aria_label` is the localization's `share`.
  The words are `ShareLabels`.
- `label` becomes the button's description through a hidden element, so the
  name stays short: "Share, Link to this page".

### You must

- Name an icon-only button after what it shares with `aria_label`, such as
  "Share article", or keep the short name and say what it shares with `label`.
- For a share control of your own on `use_share()`, call `share` in the press
  handler and say the outcome in a status region that is already mounted.

### Example

An article's share button, `Share { url, label: "Link to this article"
}`: the button reads "Share, Link to this article". On a phone the share sheet
opens; on a desktop without one, the status region says "Copied".

## CSS variables

None of its own; it renders an `ActionIcon`, or a `Button` with `children`,
whose variables apply.

## Data attributes

`ActionIcon`'s or `Button`'s, on the button's `data-state`.
