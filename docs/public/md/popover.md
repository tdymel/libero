# Popover

Crate: `libero`
Import: `use libero::hooks::{use_element, use_popover, Align, PopoverOptions, PopoverWidth, Side};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/popover/mod.rs>
Index: [index.md](index.md) lists every other page
Description: A hook that anchors a portaled box to a trigger, flipping and shifting it to stay on screen.

A popover is a hook, not a component. A dropdown, a menu and a hover card share
where the box goes, not how it looks. `use_popover` portals the box, so no
`overflow: hidden` ancestor clips it, and flips and shifts it to stay on
screen.

It owns no open state. Pass `show(None)` to take a closed box away. Popovers
nest. Anchor the inner one to a row in the outer box, and the one shown later
paints on top.

## Usage

The role, the trigger's aria and the box's focus are yours, so the example
carries them.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::{use_element, use_id, use_popover, Align, PopoverOptions, Side},
    platform::ElementApi,
    sx::sx,
    use_theme,
};

#[component]
fn Demo() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let box_id = use_id();

    let popover = use_popover(
        anchor,
        opened(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start)
            .dismiss(true),
    );
    // Escape and a press outside.
    popover.on_dismiss(move || opened.set(false));
    let floating = *popover.floating();

    // Focus the box once it is placed, once per opening.
    let mut entered = use_signal(|| false);
    use_effect(move || match (opened(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let _ = floating.focus();
        }
        (false, _) => entered.set(false),
        _ => {}
    });
    // Tab closes it and returns focus to the trigger.
    let mut close = move |event: &KeyboardEvent| {
        opened.set(false);
        let _ = anchor.focus();
        if event.modifiers().shift() {
            event.prevent_default();
        }
    };

    popover.show(opened().then(|| rsx! {
        Box {
            id: "{box_id}",
            role: "dialog",
            aria_label: "Example popover",
            tabindex: "-1",
            attributes: popover.floating_events(),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Tab {
                    close(&event);
                }
            },
            style: popover.style(),
            onmounted: floating.mount(),
            sx: sx().background("surface").padding("var(--lsx-popover-padding)"),
            "Popover content"
        }
    }));

    rsx! {
        Button {
            onmounted: anchor.mount(),
            onclick: move |_| opened.toggle(),
            attributes: popover.anchor_events(),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Tab && opened() {
                    opened.set(false);
                }
            },
            aria_haspopup: "dialog",
            aria_expanded: "{opened()}",
            aria_controls: "{box_id}",
            "Popover"
        }
    }
}
```

## Focus after placed, never after mount

```rust,ignore
// Wrong: the box is mounted but not measured, so it is still
// `visibility: hidden`. `focus()` answers Ok(()) and nothing moves.
use_effect(move || {
    if opened() {
        let _ = first_item.focus();
    }
});

// Right: wait for the measurement.
use_effect(move || {
    if !opened() || !popover.placed() {
        return;
    }
    let _ = first_item.focus();
});
```

## Context across the portal

```rust,ignore
// The box renders at the portal outlet, at the document root, so it
// inherits none of the context around the call site. Re-provide what
// the content needs, inside the content itself.
popover.show(opened().then(|| rsx! {
    MenuProvider { context: menu, {items} }
}));

// A signal a callback outside every scope writes has to outlive the
// scope that made it, and be dropped by hand.
let tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
use_drop(move || tick.manually_drop());
```

## What it cannot do

Only the web tells the box when the page scrolls. Elsewhere an open popover
drifts. No backend tracks a resize, so use `remeasure`. A press outside needs
to know where focus is. The web and Blitz can tell, a WebView cannot, so there
only Escape and your own handlers close the box.

## PopoverOptions

`PopoverOptions::new(gap, padding)` takes the theme's values, so it is not
`Default`. Every other field has a builder method of the same name.

| Prop | Type | Default | Description |
|---|---|---|---|
| `side` | `Side` | `Bottom` | The preferred side of the anchor. Flipping may override it. `Start`/`End` are logical: `Start` is the left under `dir="ltr"`, the right under `rtl`. |
| `align` | `Align` | `Start` | Where the box lines up along that side. Across `Top`/`Bottom` it follows the direction too. |
| `gap` | `f64` | `theme.popover.gap` | Pixels between the anchor and the box. |
| `padding` | `f64` | `theme.popover.padding` | How close to a viewport edge the box may come before it flips or shifts. The box is never wider than the viewport less this on both sides. |
| `flip` | `bool` | `true` | Moves to the opposite side when the preferred one has no room. |
| `shift` | `bool` | `true` | Slides along the side to stay on screen when flipping does not help. |
| `width` | `PopoverWidth` | `Auto` | `Auto` follows the content, `Match` takes the anchor's width, and `Min` is at least the anchor's width. |
| `remeasure` | `u64` | `0` | Changing it measures the box again. Use it for an anchor that resizes while the box is open. |
| `dismiss` | `bool` | `false` | Escape and a press outside close the box, through `on_dismiss`. Spread `anchor_events()` on the trigger and `floating_events()` on the box. |

## PopoverHandle

`use_popover(anchor: ElementHandle, open: bool, options: PopoverOptions) -> PopoverHandle`

| Method | Type | Description |
|---|---|---|
| `floating()` | `&ElementHandle` | Mount it on the box. Nothing is placed until it is attached. |
| `placed()` | `bool` | Whether the box has been measured. `false` on the render that opens it. |
| `placement()` | `Placement` | The side and align the box landed on, after flipping. |
| `style()` | `Option<String>` | The box's `style`, with its position and width. |
| `show(content)` | `Option<Element>` | Renders the box. `None` removes it. |
| `on_dismiss(f)` | `impl FnMut()` | What Escape and a press outside call, with `dismiss` on. Call it on every render. |
| `anchor_events()` | `Vec<Attribute>` | Spread on the trigger: `dismiss`, and on a WebView the link `Hotkey::within` follows into the box. |
| `floating_events()` | `Vec<Attribute>` | Spread on the box, as `anchor_events()` on the trigger; the WebView link needs both. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Escape` | With `dismiss(true)`: closes the box and returns focus to the trigger. |
| `Tab` | Closes the box and moves on. Focus is not trapped. |

### Libero handles

- The hook adds no role and no keys of its own. `dismiss(true)` adds the
  Escape key and closing on focus leaving.
- With `dismiss(true)`, focus leaving the trigger and the box closes it.

### You must

- Put a `role` on the box, and `aria-haspopup`, `aria-expanded` and
  `aria-controls` on the trigger, as the example does.
- Drive `aria-expanded` from the same signal the hook gets, or a screen reader
  hears the wrong state.
- Make Escape close the box (WCAG 1.4.13): turn on `dismiss(true)`, or handle
  it yourself.
- Give the box `tabindex="-1"`, or a click on its text moves focus out and
  closes it.
- If you animate the close, give the closing box `visibility: hidden` or
  `inert`. Until it unmounts, it is still tabbable and still announced.

### Limits

- Safari does not focus a button on click, so there a press outside a box
  opened by pointer does not close it.

## Theme defaults

`PopoverDefaults` on the theme holds `gap` and `padding`, both in pixels.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-popover-gap` | The theme's `gap`. |
| `--lsx-popover-padding` | The theme's `padding`, often reused as the box's inner padding. |
| `--lsx-z-index-popover` | The stacking level for a floating box. |
