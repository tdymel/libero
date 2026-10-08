use dioxus::prelude::*;

use super::options::WindowRect;
use crate::{
    components::{common::has_shortcut_modifier, layout::Placement},
    hooks::{Drag, DragMove, DragOptions, DragStart, ElementHandle, use_drag},
    platform::ElementApi,
};

/// How a pinned window's size follows the grip's logical travel: it grows about
/// its anchor, so twice when centred and inverted from an end or bottom edge.
pub(super) fn pinned_growth(placement: Placement) -> (f64, f64) {
    use Placement::*;
    let inline = match placement {
        TopStart | CenterStart | BottomStart => 1.0,
        TopCenter | CenterCenter | BottomCenter => 2.0,
        TopEnd | CenterEnd | BottomEnd => -1.0,
    };
    let block = match placement {
        TopStart | TopCenter | TopEnd => 1.0,
        CenterStart | CenterCenter | CenterEnd => 2.0,
        BottomStart | BottomCenter | BottomEnd => -1.0,
    };
    (inline, block)
}

fn arrow_delta(key: &Key, step: f64) -> Option<(f64, f64)> {
    match key {
        Key::ArrowLeft => Some((-step, 0.0)),
        Key::ArrowRight => Some((step, 0.0)),
        Key::ArrowUp => Some((0.0, -step)),
        Key::ArrowDown => Some((0.0, step)),
        _ => None,
    }
}

/// The pixel range CSS clamps a requested size into.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowBounds {
    pub(super) min: (f64, f64),
    pub(super) max: (f64, f64),
}

impl WindowBounds {
    /// What CSS draws for a requested size: `min-*` wins over `max-*`.
    pub(super) fn fit(self, (width, height): (f64, f64)) -> (f64, f64) {
        (
            width.min(self.max.0).max(self.min.0),
            height.min(self.max.1).max(self.min.1),
        )
    }
}

/// Reads computed `min-*`/`max-*` and the viewport into `bounds`, if available.
pub(super) fn read_bounds(root: ElementHandle, mut bounds: Signal<Option<WindowBounds>>) {
    let reads = ["min-width", "min-height", "max-width", "max-height"]
        .map(|property| root.computed_px(property));
    let viewport = crate::platform::document().map(|document| document.viewport());
    spawn(async move {
        let mut px = [None; 4];
        for (slot, read) in px.iter_mut().zip(reads) {
            let Ok(value) = read.await else { return };
            *slot = value;
        }
        let viewport = match viewport {
            Some(read) => read.await.ok(),
            None => None,
        };
        let cap = |max: Option<f64>, screen: Option<f64>| {
            max.unwrap_or(f64::INFINITY)
                .min(screen.unwrap_or(f64::INFINITY))
        };
        let next = WindowBounds {
            min: (px[0].unwrap_or(0.0), px[1].unwrap_or(0.0)),
            max: (
                cap(px[2], viewport.map(|v| v.width)),
                cap(px[3], viewport.map(|v| v.height)),
            ),
        };
        if *bounds.peek() != Some(next) {
            bounds.set(Some(next));
        }
    });
}

/// Hands the window's rect to `callback`. The reads start in the handler, awaited
/// in the task (`platform::Read`).
fn report(root: crate::hooks::ElementHandle, callback: Option<Callback<WindowRect>>) {
    let Some(callback) = callback else { return };
    let (offset, dimensions) = (root.client_offset(), root.dimensions());
    spawn(async move {
        if let (Ok((x, y)), Ok(dimensions)) = (offset.await, dimensions.await) {
            callback.call(WindowRect {
                x,
                y,
                width: dimensions.width,
                height: dimensions.height,
            });
        }
    });
}

/// Position, size and the two drags, as one `Copy` value every part shares.
#[derive(Clone, Copy)]
pub(super) struct WindowGeometry {
    pub(super) root: ElementHandle,
    /// Top-left in viewport pixels once moved; `None` while `placement`
    /// decides.
    pub(super) position: Signal<Option<(f64, f64)>>,
    /// What the resize handle asked for; `None` is the content's own size.
    pub(super) size: Signal<Option<(f64, f64)>>,
    /// The rendered border box, for the clamp. Kept current by `onresize`.
    pub(super) measured: Signal<Option<(f64, f64)>>,
    /// The caller's size bounds in pixels; `None` until read, and where there
    /// is no computed style.
    pub(super) bounds: Signal<Option<WindowBounds>>,
    /// A keyboard or button move or resize reports once the new geometry has
    /// rendered.
    pub(super) owed: Signal<Vec<Callback<WindowRect>>>,
    pub(super) move_drag: Drag,
    pub(super) resize_drag: Drag,
    pub(super) onmove: Option<Callback<WindowRect>>,
    pub(super) onresize: Option<Callback<WindowRect>>,
    pub(super) move_step: f64,
    pub(super) resize_step: f64,
    /// Grows about its anchor instead of from the top-left (the `pinned` option).
    pinned: Option<Placement>,
}

/// The window's own geometry state, the two drags over it, and the effect that
/// pays back a report owed to a keyboard move.
pub(super) fn use_window_geometry(
    root: ElementHandle,
    move_step: f64,
    resize_step: f64,
    pinned: Option<Placement>,
    onmove: Option<Callback<WindowRect>>,
    onresize: Option<Callback<WindowRect>>,
) -> WindowGeometry {
    let mut position = use_signal(|| None::<(f64, f64)>);
    let mut size = use_signal(|| None::<(f64, f64)>);
    let measured = use_signal(|| None::<(f64, f64)>);
    let bounds = use_signal(|| None::<WindowBounds>);
    use_effect(move || {
        if root.is_mounted() {
            read_bounds(root, bounds);
        }
    });
    // Where a pointer drag started, read once at pointerdown.
    let mut move_origin = use_signal(|| None::<(f64, f64)>);
    let mut size_origin = use_signal(|| None::<(f64, f64)>);

    // Reports after the render commits: a read in the writing task gets the old rect.
    let mut owed = use_signal(Vec::<Callback<WindowRect>>::new);
    use_effect(move || {
        let callbacks = owed();
        if !callbacks.is_empty() {
            owed.set(Vec::new());
            for callback in callbacks {
                report(root, Some(callback));
            }
        }
    });

    // Focus the window itself on open, which is what APG asks of a non-modal
    // dialog. Once: a later re-render must not pull focus back from the page.
    let mut focused = use_signal(|| false);
    use_effect(move || {
        if root.is_mounted() && !*focused.peek() {
            focused.set(true);
            let _ = root.focus();
        }
    });

    let move_drag = use_drag(DragOptions {
        capture: root,
        onstart: use_callback(move |_: DragStart| {
            move_origin.set(None);
            let offset = root.client_offset();
            spawn(async move {
                if let Ok(offset) = offset.await {
                    move_origin.set(Some(offset));
                }
            });
        }),
        onmove: use_callback(move |event: DragMove| {
            if let Some((x, y)) = move_origin() {
                let delta = event.delta();
                position.set(Some((x + delta.x, y + delta.y)));
            }
        }),
        // Owed, not read here: a WebView would read before the last move's edit lands.
        onend: use_callback(move |()| owed.write().extend(onmove)),
    });

    let mut left_origin = use_signal(|| 0.0);
    let mut resize_rtl = use_signal(|| false);
    let resize_drag = use_drag(DragOptions {
        capture: root,
        onstart: use_callback(move |_: DragStart| {
            size_origin.set(None);
            resize_rtl.set(root.is_rtl());
            // An anchored window grows away from its anchor, not under the grip (todo 2282).
            let pin = pinned.is_none() && position.peek().is_none();
            let (dimensions, offset) = (root.dimensions(), root.client_offset());
            spawn(async move {
                if let (Ok(dimensions), Ok((x, y))) = (dimensions.await, offset.await) {
                    if pin {
                        position.set(Some((x, y)));
                    }
                    left_origin.set(x);
                    size_origin.set(Some((dimensions.width, dimensions.height)));
                }
            });
        }),
        onmove: use_callback(move |event: DragMove| {
            if let Some((width, height)) = size_origin() {
                let delta = event.delta();
                // Under RTL the grip is the bottom-left corner: a drag left widens.
                let dx = if *resize_rtl.peek() {
                    -delta.x
                } else {
                    delta.x
                };
                let (inline, block) = match pinned {
                    Some(placement) if position.peek().is_none() => pinned_growth(placement),
                    _ => (1.0, 1.0),
                };
                let next = (
                    (width + dx * inline).max(0.0),
                    (height + delta.y * block).max(0.0),
                );
                size.set(Some(next));
                if *resize_rtl.peek() {
                    keep_right_edge(position, bounds, *left_origin.peek() + width, next.0);
                }
            }
        }),
        onend: use_callback(move |()| owed.write().extend(onresize)),
    });

    WindowGeometry {
        root,
        position,
        size,
        measured,
        bounds,
        owed,
        move_drag,
        resize_drag,
        onmove,
        onresize,
        move_step,
        resize_step,
        pinned,
    }
}

impl WindowGeometry {
    /// Both drags capture on the root, so it receives every move; only the one
    /// that started reacts, because `use_drag` filters on its own pointer id.
    pub(super) fn onpointermove(self, event: Event<PointerData>) {
        match (self.move_drag.dragging)() {
            true => self.move_drag.onpointermove.call(event),
            false if (self.resize_drag.dragging)() => self.resize_drag.onpointermove.call(event),
            false => {}
        }
    }

    pub(super) fn onpointerup(self, event: Event<PointerData>) {
        match (self.move_drag.dragging)() {
            true => self.move_drag.onpointerup.call(event),
            false if (self.resize_drag.dragging)() => self.resize_drag.onpointerup.call(event),
            false => {}
        }
    }

    pub(super) fn onpointercancel(self, event: Event<PointerData>) {
        match (self.move_drag.dragging)() {
            true => self.move_drag.onpointercancel.call(event),
            false if (self.resize_drag.dragging)() => self.resize_drag.onpointercancel.call(event),
            false => {}
        }
    }

    /// The title bar is the keyboard move handle: Arrow moves by a step,
    /// Shift+Arrow by a pixel.
    pub(super) fn handle_key(self, event: Event<KeyboardData>) {
        // Alt+ArrowLeft is Back: a chord is the browser's, not a move.
        if has_shortcut_modifier(&event) {
            return;
        }
        let step = if event.modifiers().shift() {
            1.0
        } else {
            self.move_step
        };
        let Some((dx, dy)) = arrow_delta(&event.key(), step) else {
            return;
        };
        event.prevent_default();
        event.stop_propagation();
        self.move_by(dx, dy);
    }

    /// Moves by a pixel delta from where the window is drawn, and reports it.
    pub(super) fn move_by(self, dx: f64, dy: f64) {
        let (mut position, mut owed, onmove) = (self.position, self.owed, self.onmove);
        let offset = self.root.client_offset();
        spawn(async move {
            if let Ok((x, y)) = offset.await {
                position.set(Some((x + dx, y + dy)));
                owed.write().extend(onmove);
            }
        });
    }

    /// Back to `placement` and the content's own size, reporting both.
    pub(super) fn reset(self) {
        let (mut position, mut size, mut owed) = (self.position, self.size, self.owed);
        position.set(None);
        size.set(None);
        owed.write()
            .extend(self.onmove.into_iter().chain(self.onresize));
    }

    /// Arrow resizes by a step, Shift+Arrow by a pixel, Home/End to the min/max.
    pub(super) fn resize_key(self, event: Event<KeyboardData>) {
        if has_shortcut_modifier(&event) {
            return;
        }
        let key = event.key();
        let request = match key {
            Key::Home => Some(Err((0.0, 0.0))),
            Key::End => Some(Err((f64::from(u16::MAX), f64::from(u16::MAX)))),
            _ => {
                let step = if event.modifiers().shift() {
                    1.0
                } else {
                    self.resize_step
                };
                // The grip's arrows move its corner, the bottom-left under RTL.
                let flip = if self.root.is_rtl() { -1.0 } else { 1.0 };
                arrow_delta(&key, step).map(|(dx, dy)| Ok((dx * flip, dy)))
            }
        };
        let Some(request) = request else { return };
        event.prevent_default();
        event.stop_propagation();
        self.resize_to(request);
    }

    /// `Ok` grows the drawn size by a delta, `Err` asks for an absolute size;
    /// the window's min/max constraints clamp either. Reports it.
    pub(super) fn resize_to(self, request: Result<(f64, f64), (f64, f64)>) {
        let (mut size, mut owed, onresize) = (self.size, self.owed, self.onresize);
        let (mut position, bounds, rtl) = (self.position, self.bounds, self.root.is_rtl());
        // An unmoved window grows from its top-left, as the pointer resize pins it (todo 2353).
        let pin = self.pinned.is_none() && position.peek().is_none();
        let (dimensions, offset) = (self.root.dimensions(), self.root.client_offset());
        spawn(async move {
            let dimensions = dimensions.await.ok();
            let offset = offset.await.ok();
            if let (true, Some(drawn)) = (pin, offset) {
                position.set(Some(drawn));
            }
            let next = match (request, dimensions) {
                (Err(absolute), _) => absolute,
                (Ok((dx, dy)), Some(dimensions)) => (
                    (dimensions.width + dx).max(0.0),
                    (dimensions.height + dy).max(0.0),
                ),
                (Ok(_), None) => return,
            };
            size.set(Some(next));
            if rtl && let (Some(dimensions), Some((x, _))) = (dimensions, offset) {
                keep_right_edge(position, bounds, x + dimensions.width, next.0);
            }
            owed.write().extend(onresize);
        });
    }
}

/// Under RTL a moved window resizes from its bottom-left corner, so its right
/// edge stays at `right` while the width becomes `width`, as clamped.
fn keep_right_edge(
    mut position: Signal<Option<(f64, f64)>>,
    bounds: Signal<Option<WindowBounds>>,
    right: f64,
    width: f64,
) {
    let Some((_, y)) = *position.peek() else {
        return;
    };
    let drawn = bounds.peek().map_or(width, |b| b.fit((width, 0.0)).0);
    position.set(Some((right - drawn, y)));
}
