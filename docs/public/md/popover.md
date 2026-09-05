# Popover

Crate: `libero`
Import: `use libero::hooks::{use_element, use_popover, Align, PopoverOptions, PopoverWidth, Side};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/popover/mod.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A popover is a hook, not a component - `use_popover` portals a box to the document root and anchors it, flipping and shifting to stay on screen.

A popover is a hook, not a component: a dropdown, a menu and a hover card share
when and where, never what the box looks like. `use_popover` portals the box to
the document root, so it escapes an `overflow: hidden` ancestor, and places it in
viewport coordinates, flipping and shifting to stay on screen. It owns no open
state - `show(None)` is how a closed popover stops rendering.

## Usage

The trigger's aria and its Escape are the consumer's, so this snippet carries both - see
[Accessibility](#accessibility).

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::{use_element, use_id, use_popover, Align, PopoverOptions, Side},
    sx::sx,
    use_theme,
};

#[component]
fn Demo() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    // One id, owned here, pointed at from the trigger.
    let box_id = use_id();

    let popover = use_popover(
        anchor,
        opened(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start),
    );
    let floating = *popover.floating();

    popover.show(opened().then(|| rsx! {
        Box {
            id: "{box_id}",
            // Plain text, so a dialog: a listbox has to hold options.
            role: "dialog",
            aria_label: "Example popover",
            style: popover.style(),
            onmounted: floating.mount(),
            sx: sx().background("white").padding("var(--lsx-popover-padding)"),
            "Popover content"
        }
    }));

    rsx! {
        Button {
            onmounted: anchor.mount(),
            onclick: move |_| opened.toggle(),
            // Focus stays on the trigger and the box is portaled, so the
            // trigger is where Escape and Tab arrive.
            onkeydown: move |event: KeyboardEvent| match event.key() {
                Key::Escape if opened() => {
                    event.prevent_default();
                    opened.set(false);
                }
                Key::Tab if opened() => opened.set(false),
                _ => {}
            },
            aria_haspopup: "dialog",
            // The same signal the hook is given, never a second copy.
            aria_expanded: "{opened()}",
            aria_controls: "{box_id}",
            "Open popover"
        }
    }
}
```

## Call order

```rust,ignore
// use_popover first: the style it returns is what the box renders with.
let anchor = use_element();
let popover = use_popover(anchor, opened(), PopoverOptions::new(gap, padding));

// Then the box. The other way round it styles itself with last
// render's placement.
let dropdown = use_box()
    .framework_sx(&DROPDOWN_SX)
    .style(popover.style())
    .prepare();

popover.show(opened().then(|| dropdown
    .element(popover.floating())
    .render(HtmlTag::Div, vec![], rsx! { .. })));
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

## Nested popovers

```rust,ignore
// A submenu is its own popover, anchored to the row that opened it.
// Geometry needs nothing: both boxes are `position: fixed`, and the one
// portaled later paints over the earlier one. Containment does - the
// submenu is no descendant of the menu, so focus moving into it reads as
// focus leaving. Register it, and hold the guard while it is open.
let _inside = use_hook(move || parent.register_inside(submenu_box));
```

## Accessibility

The hook contributes no role and no keyboard - a popover has no semantics. The
consumer supplies `role="menu"`, `"listbox"` or `"dialog"` on the box and the
`aria-haspopup`, `aria-expanded` and `aria-controls` that name the trigger, as
the Usage snippet does. Drive `aria-expanded` from the same `open` you pass the
hook: a trigger claiming to be closed over an open box is announced as closed.

Escape must close the box (WCAG 2.1 SC 1.4.13), and closing it is the consumer's.
Off the web only the element that actually holds focus hears the press, and the
box is portaled, so a surface that leaves focus on its trigger has to listen on
the trigger as well or it cannot be dismissed from the keyboard at all. Focus is
deliberately **not** trapped -
Tab closes the surface and moves on, which is what the ARIA Authoring Practices
ask for. If you animate the close, give the closing box `visibility: hidden` or
`inert` for the duration: until it unmounts it is still tabbable and still
announced.

## What it cannot do

Scroll tracking needs a document-level scroll notification, which only the web
answers today - natively an open popover drifts when the page scrolls. Nothing
tracks a resize on any backend; `remeasure` is the only answer there. Off the
web, closing when focus leaves is unreliable: every focusout counts as leaving,
including focus moving from the trigger into the content, so close on a signal of
your own there.

## Hook API

| Item | Signature | Description |
|---|---|---|
| `use_popover` | `fn(anchor: ElementHandle, open: bool, options: PopoverOptions) -> PopoverHandle` | Anchors a portaled box to `anchor`. Re-measures per open, on an `options` change, and on every scroll. |
| `PopoverHandle::floating` | `fn(&self) -> &ElementHandle` | The handle to put on the box, so it can be measured. Nothing is placed until it is attached. |
| `PopoverHandle::placed` | `fn(&self) -> bool` | Whether the box has been measured. `false` on the render that opens it. |
| `PopoverHandle::placement` | `fn(&self) -> Placement` | The side and align it actually landed on, after flipping. |
| `PopoverHandle::style` | `fn(&self) -> Option<String>` | The box's `style`: `position: fixed`, its coordinates, its width, and `visibility` until it is measured. |
| `PopoverHandle::show` | `fn(&self, content: Option<Element>)` | Portals the box. `None` takes it away, which is how a closed popover stops rendering. |

## Options

`PopoverOptions::new(gap, padding)` rather than `Default`, because the two
defaults are the theme's and reading a theme needs the running provider. Every
other field has a builder method of the same name.

| Field | Type | Default | Description |
|---|---|---|---|
| `side` | `Side` | `Bottom` | The preferred side of the anchor. Flipping may override it. |
| `align` | `Align` | `Start` | Where the box lines up along that side's cross axis. |
| `gap` | `f64` | `theme.popover.gap` | Pixels between the anchor's edge and the box. |
| `padding` | `f64` | `theme.popover.padding` | How close to a viewport edge the box may come before it flips or shifts. |
| `flip` | `bool` | `true` | Move to the opposite side when the preferred one has no room. |
| `shift` | `bool` | `true` | Slide along the side to stay on screen, once flipping cannot help. |
| `width` | `PopoverWidth` | `Auto` | Whether the box follows its own content (`Auto`), the anchor's width exactly (`Match`), or at least the anchor's width (`Min`). |
| `remeasure` | `u64` | `0` | Not a placement input: changing it re-measures. For an anchor that resizes while the box is open - nothing else re-measures. |

## Theme defaults

`PopoverDefaults` on the theme. No new knobs here - the hook ships no element
of its own.

| Field | Type | Description |
|---|---|---|
| `gap` | `f64` | Default distance between the anchor and the box, in pixels. |
| `padding` | `f64` | Default collision padding against the viewport edges, in pixels. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-popover-gap` | `PopoverDefaults::gap`, for a box that wants the theme's spacing in its own CSS. |
| `--lsx-popover-padding` | `PopoverDefaults::padding`, commonly reused as the box's own inner padding. |
| `--lsx-z-index-popover` | The stacking level a floating box should sit on. |
