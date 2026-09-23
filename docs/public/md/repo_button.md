# RepoButton

Crate: `libero`
Import: `use libero::components::{RepoButton, RepoHost};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/repo_button.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A link to a GitHub or GitLab repository with its star count beside the host's icon.

A link to a repository with its star count beside the host's icon. Give it the
repository's name; it asks the host's public API once per mount and keeps the
count for the session (the web's `sessionStorage`, memory elsewhere). Until the
count arrives, when it is 0, or when the host does not answer, the icon stands
alone.

It opens in a new tab. This site's header uses
`RepoButton { repo: "tdymel/libero" }`. The button's name comes from the
[localization](localization.md).

Native builds fetch through dioxus-native's network provider, which needs its
`net` feature. `net` is on by default, so only an app that turns dioxus-native's
default features off has to add it back.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{RepoButton, RepoHost};

#[component]
fn Demo() -> Element {
    rsx! {
        RepoButton { repo: "tdymel/libero" }
        RepoButton { repo: "gitlab-org/gitlab", host: RepoHost::GitLab }
    }
}
```

## Accessibility

### Libero handles

- The link's name is the host and repository ("GitHub tdymel/libero"), the star
  count once it arrives, and the new-tab cue, so two buttons on one page read
  apart. The words come from `RepoButtonLabels::stars` and
  `AnchorLabels::new_tab` in the localization.
- The drawn count is not read twice: the name replaces the link's content.
- A count drawn in the accent color, which could miss 4.5:1, takes the `ink`
  color instead.
- Set, `aria_label` replaces that whole name, count and new-tab cue included.

### You must

- With your own `aria_label`, name the repository and say that it opens in a
  new tab.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `repo` | `String` | - | Required. `owner/repo`, as in the repository's URL. GitLab takes nested groups too. |
| `host` | `RepoHost` | `GitHub` | Where the repository lives: `RepoHost::GitHub` or `RepoHost::GitLab`. |
| `variant` | `Variant` | `outlined` | Visual style, as on `ActionIcon`. |
| `color` | `ThemeAwareValue` | `muted` | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `md` | Button size. The icon takes half of it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `aria_label` | `String` | - | Replaces the whole built name: host, repository, star count and new-tab cue. Unset, the built name. |

Like every component, `RepoButton` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the link.

## Theme defaults

`RepoButtonDefaults` on the theme, as `repo_button`.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`outlined`). |
| `color` | `Color` | Default `color` when the prop is omitted (`muted`). |

The count's words are `RepoButtonLabels::stars` in the localization, a function
of the count for plural forms. The new-tab cue is `AnchorLabels::new_tab`.

## CSS variables

None of its own; it renders an `ActionIcon`, whose variables apply.

## Data attributes

`ActionIcon`'s, on the link's `data-state`. A `display: contents` wrapper around
the link carries `stars` while a count shows.
