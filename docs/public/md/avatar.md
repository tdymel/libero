# Avatar

Crate: `libero`
Import: `use libero::components::{Avatar, AvatarGroup, AvatarSpec};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/avatar/avatar.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A person as a fixed square, with a fallback chain from a picture down to a person glyph, and a group that collapses its overflow into a +N chip.

A person as a fixed square, with a fallback chain: the picture, then `children`,
then `initials`, then a person glyph. Nothing is derived from `name` - an
initial is a first grapheme cluster rather than a first character, and which one
abbreviates a name is a property of the script, so the caller supplies it. The
root is a `<span role="img">` carrying `name`, so a screen reader announces "Ada
Lovelace" instead of spelling out the two letters; `alt: ""` marks it
decorative, for the common case of an avatar sitting beside the person's visible
name. It is not focusable and not interactive - wrap it in a `Button` or an
`Anchor` if it should be either.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Avatar;

#[component]
fn Demo() -> Element {
    rsx! {
        Avatar { name: "Ada Lovelace", src: "/ada.png" }
    }
}
```

Each link of the chain takes over when the one before it is absent. A `src`
that fails to load counts as absent, so the fallback is what a reader sees
after a broken URL - not a blank square:

```rust
use dioxus::prelude::*;
use libero::components::Avatar;

#[component]
fn Demo() -> Element {
    rsx! {
        // The picture; `initials` show only if it fails to load.
        Avatar { name: "Ada Lovelace", src: "/ada.png", initials: "AL" }
        // No picture, so the initials.
        Avatar { name: "Grace Hopper", initials: "GH" }
        // Neither, so the person glyph.
        Avatar { name: "Katherine Johnson" }
        // Beside the person's own name, so decorative.
        Avatar { name: "Radia Perlman", alt: "", initials: "RP" }
    }
}
```

`AvatarGroup` owns its members rather than taking them as children, which is
what lets it count them. `max` is the number of circles, the chip included, so
the chip always stands for at least two people - a `+1` is unrepresentable by
construction. The chip is focusable and tooltipped, and its `aria-label` lists
the same names.

Members overlap by `spacing`, the first drawn on top, each separated from the
next by a ring in the page colour (`--lsx-paper-background`).

```rust
use dioxus::prelude::*;
use libero::components::{AvatarGroup, AvatarSpec};

#[component]
fn Demo() -> Element {
    rsx! {
        AvatarGroup {
            max: 4,
            people: vec![
                AvatarSpec {
                    name: "Ada Lovelace".into(),
                    src: Some("/ada.png".into()),
                    ..Default::default()
                },
                AvatarSpec {
                    name: "Grace Hopper".into(),
                    initials: Some("GH".into()),
                    ..Default::default()
                },
                AvatarSpec {
                    name: "Katherine Johnson".into(),
                    initials: Some("KJ".into()),
                    color: Some("secondary".into()),
                    ..Default::default()
                },
                AvatarSpec::from("Radia Perlman"),
                AvatarSpec::from("Barbara Liskov"),
                AvatarSpec::from("Margaret Hamilton"),
            ],
        }
    }
}
```

That renders three avatars and a `+3` chip labelled "3 more: Radia Perlman,
Barbara Liskov, Margaret Hamilton".

## Accessibility

`name` is the avatar's accessible name. Pass `alt: ""` wherever the person's
name is already visible beside the avatar, or it is announced twice.

The chip's text is English by default. To translate it, set the
[localization](theming.md#localization)'s `avatar`: `{n}` is the hidden count
and `{names}` their names, so a language can put the count wherever it belongs,
e.g. `AvatarLabels { count: "+{n}", more: "{n} weitere: {names}" }`.

## Props

### `Avatar`

| Prop | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The person this avatar stands for, announced as its accessible name. |
| `src` | `String` | - | The picture. Falls through to the rest of the chain once it fails to load. |
| `initials` | `String` | - | Drawn when there is no picture. Nothing is derived from `name`. |
| `alt` | `String` | follows `name` | Overrides the announced name. `alt: ""` marks the avatar decorative. |
| `size` | `Size` | `theme.avatar.size` | The square's side, which also sets the placeholder's font size. |
| `radius` | `Size` | `theme.avatar.radius` | A step on the avatar's own radius scale, `2px` to `32px`. The default, `xxl`, is `9999px`: a circle. |
| `variant` | `Variant` | `tonal` | Placeholder chrome; invisible once a picture loads. |
| `color` | `ThemeAwareValue` | `primary` | Placeholder tint. |
| `children` | `Element` | - | Anything at all in place of the initials - an icon, a glyph. |

### `AvatarGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `people` | `Vec<AvatarSpec>` | required | The members, in paint order: the first is drawn on top. |
| `max` | `usize` | - | How many circles in total. Past that, the rest collapse into a `+N` chip. |
| `spacing` | `Size` | `theme.avatar_group.spacing` | How far each circle is pulled over the one before it. |
| `size` | `Size` | `theme.avatar.size` | Applied to every member, the chip included. |
| `radius` | `Size` | `theme.avatar.radius` | Applied to every member, the chip included. |
| `variant` | `Variant` | `tonal` | Applied to every member, the chip included. |
| `color` | `ThemeAwareValue` | `primary` | The tint a member without a `color` of its own takes. |

### `AvatarSpec`

A plain struct with `Default`, so `AvatarSpec { name: .., ..Default::default() }`
is the long form and `"Ada Lovelace".into()` the short one. It holds no
`Element`, so a `Vec<AvatarSpec>` compares equal and the group's members
memoize.

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The accessible name, and what the overflow chip lists. |
| `src` | `Option<String>` | `None` | The picture. |
| `initials` | `Option<String>` | `None` | Drawn when there is no picture. |
| `color` | `Option<ThemeAwareValue>` | `None` | This member's own tint, overriding the group's. |

`Variant` takes `filled`, `tonal`, `elevated`, `outlined` or `standard`.

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`AvatarDefaults` and `AvatarGroupDefaults` on the theme. The chip's words are
`AvatarLabels` in the localization: `count` (`+{n}`, the visible text) and
`more` (`{n} more: {names}`, the accessible name).

| Field | Type | Default | Description |
|---|---|---|---|
| `avatar.variant` | `Variant` | `tonal` | Default `variant` when the prop is omitted, for an `Avatar` and for an `AvatarGroup`. |
| `avatar.size` | `Size` | `md` | Default `size` when the prop is omitted. |
| `avatar.radius` | `Size` | `xxl` | The step of `avatar.radii` used when the prop is omitted - a circle. |
| `avatar.sizes` | `Sizes<u16>` | `20, 28, 38, 56, 84, 120` | The square's side, in px, per size step. |
| `avatar.font_sizes` | `Sizes<u16>` | `8, 11, 15, 22, 34, 48` | Placeholder font size, derived from the square at `side / 2.5`. |
| `avatar.radii` | `Sizes<&'static str>` | `2px, 4px, 8px, 16px, 32px, 9999px` | The avatar's own radius scale. `xxl` is a circle at every size. |
| `avatar_group.spacing` | `Size` | `sm` | How far each circle is pulled over the one before it. |
| `avatar_group.ring` | `&'static str` | `2px` | Width of the ring in the page colour that separates two overlapping members. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-avatar-size-{xs..xxl}` | The square's side per size step. |
| `--lsx-avatar-font-size-{xs..xxl}` | Placeholder font size per size step. |
| `--lsx-avatar-radius-<size>` | The radius for that step, from `AvatarDefaults::radii`. |
| `--lsx-avatar-radius` | The theme's default step, as `var(--lsx-avatar-radius-xxl)`. |
| `--lsx-avatar-radius-override` | Set from the `radius` prop to that step's var; wins over the theme value. |
| `--lsx-avatar-color` | The resolved `color`, which the variant chrome reads. |
| `--lsx-avatar-contrast` | What reads on `--lsx-avatar-color`; unset for a literal colour, which has no shade ramp. |
| `--lsx-avatar-container` / `--lsx-avatar-on-container` | The tonal variant's tint and its label colour. |
| `--lsx-avatar-group-spacing` | The overlap, set on the group root from its `spacing`. |
| `--lsx-avatar-group-ring` | Width of the ring around a member inside a group. |
| `--lsx-avatar-group-index` | A member's paint order, set per element by the group: the first member gets the highest. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-xs` … `size-xxl` | The `size` in effect. |
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `grouped` | On a member rendered by an `AvatarGroup`, and on the `+N` chip: it draws the ring and the paint order. |
