# IconProvider

Crate: `libero`
Import: `use libero::{IconProvider, IconSet, IconSlot};`
Feature: none, always compiled ([features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/context/icons.rs>
Index: [index.md](index.md) lists every other page
Description: Swaps the glyphs libero draws itself (chevrons, close, checks, ...), slot by slot, for everything below it; lucide by default.

Need icons? [pictogram](https://github.com/tdymel/pictogram) ships lucide, Tabler, Material and more.

Swaps the glyphs libero draws itself, slot by slot, for everything below it.
Each `IconSlot` names one glyph by what it means; an `IconSet` maps slots to
`SvgData`. Slots you leave empty keep lucide, libero's default. Nested
providers merge: the inner one wins per slot. More on [providers](providers.md)
in one page.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet,
    components::{Checkbox, Flex, NumberField, Options, Pagination, PasswordField, Select},
};

#[derive(Clone, PartialEq, Options)]
enum Fruit {
    Apple,
    Pear,
}

#[component]
fn Inputs() -> Element {
    let mut fruit = use_signal(|| Some(Fruit::Apple));
    let mut ripe = use_signal(|| true);
    let mut quantity = use_signal(|| Some(1));
    let mut page = use_signal(|| 2);
    rsx! {
        Flex { direction: "column", gap: "md",
            Select { label: "Fruit", value: fruit(), onchange: move |next| fruit.set(next) }
            Checkbox { label: "Ripe", checked: ripe(), onchange: move |next| ripe.set(next) }
            PasswordField { label: "Password" }
            NumberField { label: "Quantity", steppers: true, value: quantity(), onchange: move |next: Option<i32>| quantity.set(next) }
            Pagination { total: 3, page: page(), onchange: move |next| page.set(next), aria_label: "Pages", with_edges: true }
        }
    }
}

#[component]
fn Demo() -> Element {
    rsx! {
        IconProvider { icons: IconSet::new(), Inputs {} }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `icons` | `IconSet` | required | The slots to swap: `IconSet::new().with(IconSlot::Close, icon)` for one, or a whole set from its constructor, such as `IconSet::material_rounded()`, after turning on its libero feature (`icons-material`, `icons-tabler`, `icons-bootstrap` or `icons-phosphor`); only the set you call is compiled in. An empty slot keeps the outer provider's glyph, then libero's lucide default. Brand marks are no slots: they name a service. `IconSet::lucide_outlined()` resets a subtree. Bootstrap and Phosphor have no text direction glyph, so `TextDirectionLtr` and `TextDirectionRtl` keep lucide. A component keeps its own tweaks on a swapped glyph, such as `Checkbox`'s `stroke-width="3"`. `use_icon(slot, default)` reads a slot in your own component. |

## Accessibility

### Libero handles

- A swapped glyph stays decorative (`aria-hidden="true"`): the control it sits
  in keeps its own name.

### You must

- Pick a glyph that means the same as the one it replaces: a chevron for
  `ChevronDown`, a check for `CheckboxCheck`.

### Example

An `IconProvider` that swaps `ChevronDown` for your own chevron: a `Select`
keeps its own name, and the new glyph stays hidden from screen readers like
the one it replaces.
