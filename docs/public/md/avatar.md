# Avatar

Crate: `libero`
Import: `use libero::components::{Avatar, AvatarGroup, AvatarSpec};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/avatar/avatar.rs>
Index: [index.md](index.md) lists every other page
Description: A person as a fixed square, with a fallback chain from a picture down to a person glyph, and a group that collapses its overflow into a +N chip.

A person as a fixed square. It shows the picture, else `children`, else
`initials`, else a person glyph. A picture that fails to load falls back too.
Nothing is derived from `name`, since the letters that abbreviate a name depend
on the script. The avatar is not focusable. Wrap it in a `Button` or an
`Anchor` to make it interactive.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Avatar;

#[component]
fn Demo() -> Element {
    rsx! {
        // `initials` show only if the picture fails to load.
        Avatar { name: "Ada Lovelace", src: "/ada.png", initials: "AL" }
        // Beside the person's own name, so decorative.
        Avatar { name: "Radia Perlman", alt: "", initials: "RP" }
    }
}
```

`AvatarGroup` takes its members as data rather than children, so it can count
them. `max` counts circles, the chip included.

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

`name` is the accessible name, so a screen reader says "Ada Lovelace" rather
than the initials. Pass `alt: ""` where the name shows beside the avatar, or it
is read twice. A decorative avatar is hidden whole, so put nothing focusable in
it.

The group's `+N` chip is focusable, and its label lists the hidden names. Its
words come from the `avatar` labels of the [localization](localization.md).
`{n}` is the hidden count and `{names}` their names, for example
`AvatarLabels { count: "+{n}", more: "{n} weitere: {names}" }`.

## Props

### `Avatar`

| Prop | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The person this avatar stands for, read as its accessible name. |
| `src` | `Option<String>` | `None` | The picture. Falls back to the rest of the chain when it fails to load. |
| `initials` | `Option<String>` | `None` | Drawn when there is no picture. Nothing is derived from `name`. |
| `alt` | `Option<String>` | `None` | Replaces the announced name. `alt: ""` marks the avatar decorative. |
| `size` | `Size` | `md` | The side of the square, which also sets the placeholder's font size. |
| `radius` | `Size` | `xxl` | A step on the avatar's own radius scale, `2px` to `32px`. The default `xxl` is a circle. |
| `variant` | `Variant` | `tonal` | The placeholder's look. Hidden once a picture loads. |
| `color` | `ThemeAwareValue` | `primary` | The placeholder's tint. |
| `children` | `Option<Element>` | `None` | Anything in place of the initials, such as an icon. |

### `AvatarGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `people` | `Vec<AvatarSpec>` | required | The members. The first is drawn on top. |
| `max` | `Option<usize>` | `None` | How many circles in total, the `+N` chip included, so the chip always stands for at least two people. |
| `spacing` | `Size` | `sm` | How far each circle overlaps the one before it. |
| `size` | `Size` | `md` | For every member and the chip. |
| `radius` | `Size` | `xxl` | For every member and the chip. |
| `variant` | `Variant` | `tonal` | For every member and the chip. |
| `color` | `ThemeAwareValue` | `primary` | The tint of every member without a `color` of its own. |

### `AvatarSpec`

A plain struct with `Default`. Write `AvatarSpec { name: .., ..Default::default() }`
in full, or `"Ada Lovelace".into()` for a name alone.

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The accessible name, and what the chip lists. |
| `src` | `Option<String>` | `None` | The picture. |
| `initials` | `Option<String>` | `None` | Drawn when there is no picture. |
| `color` | `Option<ThemeAwareValue>` | `None` | This member's own tint. |

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
| `avatar.radius` | `Size` | `xxl` | The step of `avatar.radii` when the prop is omitted, a circle. |
| `avatar.sizes` | `Sizes<u16>` | `20, 28, 38, 56, 84, 120` | The square's side, in px, per size step. |
| `avatar.font_sizes` | `Sizes<u16>` | `8, 11, 15, 22, 34, 48` | Placeholder font size, `side / 2.5`. |
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
| `--lsx-avatar-contrast` | What reads on `--lsx-avatar-color`. Unset for a CSS color. |
| `--lsx-avatar-container` / `--lsx-avatar-on-container` | The tonal variant's tint and its label colour. |
| `--lsx-avatar-group-spacing` | The overlap, set on the group root from its `spacing`. |
| `--lsx-avatar-group-ring` | Width of the ring around a member inside a group. |
| `--lsx-avatar-group-index` | A member's paint order. The first member gets the highest. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-xs` … `size-xxl` | The `size` in effect. |
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `grouped` | On a member of an `AvatarGroup` and on the `+N` chip. It draws the ring and the paint order. |
