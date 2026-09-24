# Tldr

Crate: `libero`
Import: `use libero::components::{SummaryProvider, Tldr, summary_url};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/tldr.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A menu of links that ask an AI assistant (ChatGPT, Google AI, Claude, Perplexity or your own) to summarize a page.

A menu of links that hand a page to an AI assistant to summarize. Each link
opens the assistant's new chat with a prompt naming `url`. No script and no API
key: the prompt rides in the link's query.

This site's pages use it beside "View as markdown", with a prompt of their own.
The words come from the [localization](localization.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Tldr;

#[component]
fn Demo() -> Element {
    rsx! {
        Tldr { url: "https://libero-ui.dev/md/tldr.md" }
        Tldr { url: "https://libero-ui.dev/md/tldr.md", icon_only: true }
    }
}
```

## Your own providers and prompt

`SummaryProvider::defaults()` is a plain `Vec`: filter it to drop a provider,
push one to add it. A provider is a name, an address the encoded prompt is
appended to, and an optional mark drawn in the text colour. `{url}` in `prompt`
is filled with `url`; `summary_url` builds one link on its own.

```rust
use dioxus::prelude::*;
use libero::components::{SummaryProvider, Tldr};

#[component]
fn Demo() -> Element {
    let mut providers = SummaryProvider::defaults();
    providers.retain(|provider| provider.name != "Google AI");
    providers.push(SummaryProvider::new("Mistral", "https://chat.mistral.ai/chat?q="));
    rsx! {
        Tldr {
            url: "https://example.com/guide",
            providers,
            // Doubled braces: rsx would fill a bare `{url}` itself.
            prompt: "Summarize {{url}} in five bullet points.",
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `url` | `String` | - | Required. The page's absolute address, e.g. its markdown mirror. The assistant is asked to read it. |
| `providers` | `Vec<SummaryProvider>` | `SummaryProvider::defaults()` | The menu's links, in order: ChatGPT, Google AI, Claude and Perplexity. Filter the vec to drop one, push a `SummaryProvider` to add one. |
| `prompt` | `String` | - | What the assistant is asked. `{url}` is filled with `url`; in an `rsx!` string literal write `{{url}}`. Unset, `TldrLabels::prompt` from the localization. |
| `label` | `String` | - | The trigger's text. Unset, `TldrLabels::label` ("TLDR"). |
| `icon_only` | `bool` | `false` | Draws only the sparkles, named by `aria_label`. |
| `aria_label` | `String` | - | Names the icon-only trigger. Unset, `TldrLabels::icon_only` ("Summarize with AI"). |
| `variant` | `Variant` | `outlined` | The trigger's visual style. Unset, `theme.tldr.variant`. |
| `size` | `Size` | `md` | The trigger's size step, the icon-only one too. |
| `radius` | `Size` | - | Corner radius, independent of `size`. Unset, the trigger's own: `xl` on the labelled chip, `sm` icon-only. |
| `color` | `ThemeAwareValue` | `neutral` | The trigger's accent color. A theme color name or any CSS color. Unset, `theme.tldr.color`. |

Like every component, `Tldr` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the trigger.

## SummaryProvider

| Item | Description |
|---|---|
| `name: String` | The link's text and accessible name. |
| `url: String` | A "new chat with this prompt" address; the encoded prompt is appended. |
| `mark: Option<Element>` | Drawn before the name, in the text colour. |
| `new(name, url)` | A provider without a mark; `.mark(rsx! { .. })` adds one. |
| `chatgpt()`, `google_ai()`, `claude()`, `perplexity()` | The built-ins, with their marks. |
| `defaults()` | The four built-ins, in that order. |

`summary_url(&provider, prompt, url)` returns one link: `prompt` with `{url}`
filled, percent-encoded, appended to the provider's address.

The words are `TldrLabels` in the localization: `label`, `icon_only`, `group`
and `prompt`. Provider names are brand names and are not translated.

The marks are Lobe Icons' monochrome ones (MIT, Copyright (c) 2023 LobeHub),
from `pictogram-icons-lobe`. They are trademarks of their owners, shown only to name the service a
link opens.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Enter` or `Space` or `ArrowDown` | On the trigger: opens the menu on its first link. |
| `ArrowUp` | On the trigger: opens the menu on its last link. |
| `ArrowDown` or `ArrowUp` | Moves between the links. |
| `Enter` or `Space` | On a link: follows it in a new tab and closes the menu. |
| `Escape` | Closes the menu and returns focus to the trigger. |

### Libero handles

- It is `Menu`'s menu button: the trigger has `aria-haspopup` and
  `aria-expanded`, the links sit in a group named "Summarize with".
- Each link is an `<a role="menuitem">` with a real `href`, so middle-click and
  the context menu work. It opens in a new tab, `rel="noopener noreferrer"`.
- The provider marks are hidden from assistive technology; the provider's name
  is the link's name.
- The icon-only trigger is named "Summarize with AI". Every word comes from
  `TldrLabels` in the localization.

### You must

- Give a custom provider a name that says which service it opens.

## Theme defaults

`TldrDefaults` on the theme, as `tldr`.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`outlined`). |
| `color` | `Color` | Default `color` when the prop is omitted (`neutral`). |

## CSS variables

None of its own; it renders a `Chip` (or an `ActionIcon` when `icon_only`) and
a `Menu`, whose variables apply.

## Data attributes

The `Chip`'s or `ActionIcon`'s on the trigger, the `Menu`'s on the menu.
