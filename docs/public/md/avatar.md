# Avatar

Crate: `libero`
Import: `use libero::components::{Avatar, AvatarGroup, AvatarSpec};`
Feature: `avatar` (off by default, turns on `tooltip`, `use-icon`, `use-id`, `visually-hidden`; [features.md](features.md))
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
        Avatar { name: "Ada Lovelace", src: "/ada.png" }
    }
}
```

## Props

### `Avatar`

| Prop | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The person this avatar stands for, read as its accessible name. |
| `src` | `Option<String>` | `None` | The picture. Falls back to the rest of the chain when it fails to load, or for a `javascript:` or `vbscript:` URL, which also warns in debug builds. |
| `initials` | `Option<String>` | `None` | Drawn when there is no picture. Nothing is derived from `name`. |
| `alt` | `Option<String>` | `None` | Replaces the announced name. `alt: ""` marks the avatar decorative. |
| `size` | `Size` | `md` | The side of the square, which also sets the placeholder's font size. |
| `radius` | `ThemeAwareValue` | `xxl` | A step on the avatar's own radius scale, `2px` to `32px`, or any CSS, e.g. `radius: "0"`. The default `xxl` is a circle. |
| `variant` | `Variant` | `tonal` | The placeholder's look. Hidden once a picture loads. |
| `color` | `ThemeAwareValue` | `primary` | The placeholder's tint. |
| `parts` | `Parts<AvatarPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Option<Element>` | `None` | Anything in place of the initials, such as an icon. |

### `AvatarGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `people` | `Vec<AvatarSpec>` | required | The members, as data rather than children so the group can count them. The first is drawn on top. |
| `max` | `Option<usize>` | `None` | How many circles in total, the `+N` chip included, so the chip always stands for at least two people. `0` counts as `1`. |
| `total` | `Option<usize>` | `None` | The full headcount when `people` lists only some, so the `+N` chip counts the rest. A value below `people.len()` is ignored. The chip lists only the hidden people it knows, and is no tab stop when there are none. |
| `surplus` | `Callback<usize, Element>` | `None` | Draws the `+N` chip's content from the hidden count. The chip keeps its look, tooltip and accessible name. |
| `spacing` | `ThemeAwareValue` | `sm` | How far each circle overlaps the one before it, or any CSS, e.g. `spacing: "0"`. |
| `size` | `Size` | `md` | For every member and the chip. |
| `radius` | `ThemeAwareValue` | `xxl` | For every member and the chip, or any CSS, e.g. `radius: "0"`. |
| `variant` | `Variant` | `tonal` | For every member and the chip. `standard` has no edge, so it suits a single avatar: in a group its members read as loose initials. |
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

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `AvatarPart::Image` | `image` | The picture, while it shows. The fallback has no wrapper: style it through `sx`. |

## Accessibility

### Libero handles

- `name` is the accessible name, so a screen reader says "Ada Lovelace" rather
  than the initials.
- A decorative avatar is hidden whole.
- The group's `+N` chip is focusable. Its name is the count, "3 more", and its
  tooltip, which lists the hidden names, describes it. With `total` above the
  listed people, the names end "and others". Its words come from the `avatar`
  labels of the [localization](localization.md). `{n}` is the hidden count and
  `{names}` their names, for example
  `AvatarLabels { count: "+{n}", more: "{n} weitere", others: "{names} und weitere" }`.

### You must

- Pass `alt: ""` where the name shows beside the avatar, or it is read twice.
- Put nothing focusable in a decorative avatar.

### Example

A comment author, `Avatar { name: "Ada Lovelace", alt: "" }` beside the
visible name: the name is read once, from the text, not twice.

## Theme defaults

`AvatarDefaults` and `AvatarGroupDefaults` on the theme. The chip's words are
`AvatarLabels` in the localization: `count` (`+{n}`, the visible text), `more`
(`{n} more`, the accessible name) and `others` (`{names} and others`, the
tooltip when `total` counts people the list does not hold).

| Field | Type | Default | Description |
|---|---|---|---|
| `avatar.variant` | `Variant` | `tonal` | Default `variant` when the prop is omitted, for an `Avatar` and for an `AvatarGroup`. |
| `avatar.color` | `Color` | `primary` | Default `color` when the prop is omitted, for an `Avatar`, an `AvatarGroup` member and its `+N` chip. |
| `avatar.size` | `Size` | `md` | Default `size` when the prop is omitted. |
| `avatar.radius` | `Size` | `xxl` | The step of `avatar.radii` when the prop is omitted, a circle. |
| `avatar.sizes` | `Sizes<u16>` | `20, 28, 38, 56, 84, 120` | The square's side, in px, per size step. |
| `avatar.font_sizes` | `Sizes<u16>` | `10, 12, 15, 22, 34, 48` | Placeholder font size in px at a 16px root, written as `rem`; about `side / 2.5`, never under 10. |
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
