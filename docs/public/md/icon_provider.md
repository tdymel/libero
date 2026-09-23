# IconProvider

Crate: `libero`
Import: `use libero::{IconProvider, IconSet, IconSlot};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/context/icons.rs>
Index: [index.md](index.md) lists every other page
Description: Swaps the glyphs libero draws itself (chevrons, close, checks, ...), slot by slot, for everything below it; lucide by default.

Need icons? [pictogram](https://github.com/tdymel/pictogram) ships lucide, Tabler, Material and more.

Swaps the glyphs libero draws itself, slot by slot, for everything below it.
Each `IconSlot` names one glyph by what it means; an `IconSet` maps slots to
`SvgData`. Slots you leave empty keep lucide, libero's default. Nested
providers merge: the inner one wins per slot.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot,
    components::{Checkbox, Options, Select},
};
use pictogram_icons_lucide as lucide;

#[derive(Clone, PartialEq, Options)]
enum Fruit {
    Apple,
    Pear,
}

const ICONS: IconSet = IconSet::new()
    .with(IconSlot::ChevronDown, lucide::chevrons_down::outlined)
    .with(IconSlot::CheckboxCheck, lucide::check_check::outlined);

#[component]
fn Demo() -> Element {
    rsx! {
        IconProvider { icons: ICONS,
            Select { label: "Fruit", value: Fruit::Apple, onchange: |_: Option<Fruit>| {} }
            Checkbox { label: "Ripe", checked: true, onchange: |_| {} }
        }
    }
}
```

One slot: `IconSet::new().with(slot, icon)` for just that slot, as above.

The whole set: fill every slot once, from another pictogram icon crate
(`pictogram-icons-tabler`, `pictogram-icons-material`, ...), at the root of
your app. There is no mapping between sets, since their icon names differ; a
slot you skip keeps lucide.

```rust
use dioxus::prelude::*;
use libero::{IconProvider, IconSet, IconSlot, LiberoProvider};
use pictogram_icons_lucide as lucide;

// Lucide's circled variants; a whole set lists every slot under "Slots".
const MY_ICONS: IconSet = IconSet::new()
    .with(IconSlot::Close, lucide::circle_x::outlined)
    .with(IconSlot::Check, lucide::circle_check::outlined)
    .with(IconSlot::Plus, lucide::circle_plus::outlined)
    .with(IconSlot::Minus, lucide::circle_minus::outlined)
    .with(IconSlot::ChevronDown, lucide::circle_chevron_down::outlined)
    .with(IconSlot::ChevronUp, lucide::circle_chevron_up::outlined)
    .with(IconSlot::ChevronLeft, lucide::circle_chevron_left::outlined)
    .with(IconSlot::ChevronRight, lucide::circle_chevron_right::outlined);

#[component]
fn App() -> Element {
    rsx! {
        LiberoProvider {
            IconProvider { icons: MY_ICONS,
                "The rest of your app."
            }
        }
    }
}
```

`use_icon(slot, default)` reads a slot in your own component, `default` when no
provider sets it:

```rust
use dioxus::prelude::*;
use libero::{IconSlot, components::Pictogram, hooks::use_icon};

#[component]
fn CloseMark() -> Element {
    let icon = use_icon(IconSlot::Close, pictogram_icons_lucide::x::outlined);
    rsx! { Pictogram { icon, width: "16px", height: "16px" } }
}
```

What a slot cannot change:

- Brand marks (`RepoButton`'s GitHub and GitLab, `Tldr`'s providers) are not
  slots: they name a service.
- A component keeps its own tweaks on a swapped glyph: `Checkbox` draws its
  marks at `stroke-width="3"`.
- Right to left, CSS mirrors `ChevronLeft` and `ChevronRight` where the
  component does today (calendar, pagination, carousel); swap the physical
  glyph only.

## Slots

`Close`, `ChevronDown`, `ChevronUp`, `ChevronLeft`, `ChevronRight`,
`ChevronFirst`, `ChevronLast`, `ArrowDown`, `Check`, `CheckboxCheck`,
`CheckboxIndeterminate`, `Plus`, `Minus`, `Eye`, `EyeOff`, `Upload`,
`EyeDropper`, `Copy`, `CopyFailed`, `ExternalLink`, `Person`, `Sun`, `Moon`,
`SystemScheme`, `Play`, `Pause`, `TextDirectionLtr`, `TextDirectionRtl`,
`Sparkles`, `Grip`. `IconSlot` is `#[non_exhaustive]`: new slots may come.

## Accessibility

### Libero handles

- A swapped glyph stays decorative (`aria-hidden="true"`): the control it sits
  in keeps its own name.

### You must

- Pick a glyph that means the same as the one it replaces: a chevron for
  `ChevronDown`, a check for `CheckboxCheck`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `icons` | `IconSet` | required | The slots to swap. An empty slot keeps the outer provider's glyph, then libero's lucide default. |
