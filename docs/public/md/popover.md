# Popover

Crate: `libero`
Import: `use libero::hooks::{use_element, use_popover, Align, PopoverOptions, PopoverWidth, Side};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/popover/mod.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A popover is a hook, not a component - `use_popover` portals a box to the document root and anchors it, flipping and shifting to stay on screen.

There is no `Popover` component. A popover is a hook - `use_popover` - because
every consumer themes its own box: a dropdown, a menu and a hover card share
when and where, never what it looks like. The hook portals the box out to the
document root, so it escapes an `overflow: hidden` ancestor, and places it in
viewport coordinates, flipping and shifting to stay on screen.

It owns no open state. `show(None)` is how a closed popover stops rendering,
and deciding when that happens is the caller's.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::{use_element, use_popover, Align, PopoverOptions, Side},
    sx::sx,
    use_theme,
};

#[component]
fn Demo() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();

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
            "Open popover"
        }
    }
}
```

## Call order

Call `use_popover` before the `use_box()` whose `style()` takes its placement.
Hooks are positional, so the order is not a style preference: the box reads the
placement the hook produced on this render, and the other way round it styles
itself with the previous one.

```rust
// use_popover first: the style it returns is what the box renders with.
let anchor = use_element();
let popover = use_popover(anchor, opened(), PopoverOptions::new(gap, padding));

// Then the box, taking that style.
let dropdown = use_box()
    .framework_sx(&DROPDOWN_SX)
    .style(popover.style())
    .prepare();

popover.show(opened().then(|| dropdown
    .element(popover.floating())
    .render(HtmlTag::Div, vec![], rsx! { .. })));
```

## Focus after placed, never after mount

A box is mounted one render before it is measured, and until then it is
`visibility: hidden` - laid out, so it can be measured, but not shown.
`focus()` on a hidden element returns `Ok(())` and moves nothing, with no error
to catch. That is why this fails silently rather than loudly: wait for
`placed()`.

```rust
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

## Dismissal

Closing is not the placement hook's business, so it lives beside it in
`use_dismiss`: Escape, focus leaving the box, and handing focus back to
whatever opened it. It renders nothing - it hands back two attributes to spread
on the box you drew yourself.

`use_dismiss` is internal while `Menu` is still finding its contract; the shape
below is what it will be when it goes public. Until then a downstream dropdown
writes the two handlers itself.

```rust
let dismiss = use_dismiss(
    anchor,
    *popover.floating(),
    opened(),
    popover.placed(),
    Some(close),
    DismissOptions {
        initial_focus: Some(first_item),
        ..Default::default()
    },
);

// On the trigger, synchronously - that is where the active element
// still is the one the user acted on. For a trigger the application
// may delete while the box is open, name where focus should land
// instead:
onclick: move |_| {
    dismiss.focus_return().remember_active();
    dismiss.focus_return().fallback(list);
    opened.toggle();
}

// On the floating box: Escape and the focus-leaves check.
popover.show(opened().then(|| dropdown
    .element(popover.floating())
    .render(HtmlTag::Div, dismiss.floating_events(), rsx! { .. })));
```

`placed` sits beside `open` rather than inside `DismissOptions` because it is
reactive per-render state, not configuration. A reactive value hidden in an
options struct eventually gets read once and goes stale, and the failure is
silent: the box never receives focus and nothing errors.

Escape is arbitrated rather than claimed. Every open dismissible layer - a
popover, and a [Modal](modal.md) too - is on one stack ordered by open time,
and a handler acts only if its own layer is on top. So a popover open inside a
modal closes on Escape and the modal under it stays up, without either of them
stopping the event: stopping propagation on a document-level listener would
kill every other handler for that press in the whole document.

A layer joins that stack only where it can hear Escape from outside its own
subtree, which today means only where the platform can report a document-level
key press. A pointer-opened box leaves focus where it was, so its own
`onkeydown` never fires; if it took the top of the stack anyway, the modal that
did hear the press would decline and Escape would do nothing at all. Where there
is no document-level listener the box keeps its own handler, the modal stays
top, and Escape behaves exactly as it did before.

A held Escape is one intent, so dismissal ignores an auto-repeat - otherwise a
press-and-hold walks down the stack, closing the menu and then the modal behind
it. Focus returns to the trigger on a deliberate close, meaning Escape or an
item being chosen, and not when focus simply left the box: a click has already
put it somewhere the user meant.

## Context across the portal

The box does not render where it is written. It renders at the portal outlet,
at the document root, so it inherits no context from around the call site - a
`use_context` inside the content finds the outlet's ancestors, not yours.
Re-provide what the content needs, inside the content.

```rust
popover.show(opened().then(|| rsx! {
    MenuProvider { context: menu, {items} }
}));

// A signal a callback outside every scope writes has to outlive the
// scope that made it, and be dropped by hand.
let tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
use_drop(move || tick.manually_drop());
```

## Nested popovers

A popover inside a popover needs nothing from the geometry: both boxes are
placed in viewport coordinates, and the one that enters the portal later paints
over the earlier one. What it does need is the containment check - the inner
box is not a descendant of the outer one, so focus moving into it looks exactly
like focus leaving.

```rust
// Register the submenu's box as counting *inside* the parent, and keep
// the guard for as long as the submenu is open. It is a call rather than
// a field because the submenu only exists after the parent's hook ran.
let _inside = use_hook(move || parent.register_inside(submenu_box));
```

## What it cannot do

Scroll tracking needs a document-level scroll notification, which only the web
answers today - natively an open popover drifts when the page scrolls. Nothing
tracks a resize on any backend; `remeasure` is the only answer there, on every
platform. The focus-leaves check needs to wait for the platform's next task to
see where focus landed, and that wait is a real one only on the web.

Off the web that check is wrong rather than merely inert. Where the platform
cannot report which element has focus or search a subtree, nothing counts as
inside, so every focusout closes the box - including focus moving from the
trigger into the list. A consumer that has to work on those backends passes
`outside: false` and closes on its own signal instead.

## Accessibility

The hook contributes no role and no keyboard of its own - a popover has no
semantics. The consumer supplies `role="menu"`, `"listbox"` or `"dialog"`, and
the `aria-haspopup`, `aria-expanded` and `aria-controls` that name the trigger,
because only the consumer knows which popup type it is and owns the id scheme.

Escape must close a popover (WCAG 2.1 SC 1.4.13), which is what `use_dismiss`
answers, and only the top layer acts. Focus is deliberately **not** trapped: a
menu's Tab closes it and moves on, which is what the ARIA Authoring Practices
ask for.

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
