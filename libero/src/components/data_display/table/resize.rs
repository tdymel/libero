use std::collections::BTreeMap;

use dioxus::prelude::*;

use super::use_table::StateSlice;
use crate::{
    components::{
        accessibility::Announcer,
        overlay::{MenuEntry, MenuItem},
    },
    hooks::{DragMove, DragOptions, DragStart, use_drag, use_element, use_resize_fallback},
    localization::TableLabels,
    platform::ElementApi,
};

/// Resized column widths in CSS px, by header text. An entry wins over the
/// column's own `width`.
///
/// ```rust
/// # use libero::components::ColumnWidths;
/// let widths = ColumnWidths::from([("Name".to_string(), 240.0)]);
/// ```
pub type ColumnWidths = BTreeMap<String, f64>;

/// How far the column menu's Widen and Narrow step, in CSS px.
pub(super) const MENU_STEP: f64 = 50.0;

/// The widths a resize keeps a column within, in CSS px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResizeLimits {
    pub min: f64,
    pub max: f64,
}

impl ResizeLimits {
    pub fn new(min: f64, max: f64) -> Self {
        let min = min.max(0.0);
        Self {
            min,
            max: max.max(min),
        }
    }

    /// `width` within the limits, in whole px.
    pub fn clamp(self, width: f64) -> f64 {
        width.clamp(self.min, self.max).round()
    }
}

impl Default for ResizeLimits {
    fn default() -> Self {
        Self::new(50.0, f64::INFINITY)
    }
}

/// `widths` with `header` at `width`, or without an entry for `None`.
pub(super) fn with_width(widths: &ColumnWidths, header: &str, width: Option<f64>) -> ColumnWidths {
    let mut next = widths.clone();
    match width {
        Some(width) => next.insert(header.to_string(), width),
        None => next.remove(header),
    };
    next
}

/// A table's resize state, shared by the handles and the column menus.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ColumnResize {
    pub widths: StateSlice<ColumnWidths>,
    /// The width a drag shows before it ends, by header index.
    pub preview: Signal<Option<(usize, f64)>>,
    /// Each header cell's last rendered width, by header text.
    pub measured: CopyValue<BTreeMap<String, f64>>,
    pub labels: TableLabels,
}

impl ColumnResize {
    /// The column's width now: resized, else as last rendered.
    pub fn width(&self, header: &str) -> Option<f64> {
        self.widths
            .peek()
            .get(header)
            .copied()
            .or_else(|| self.measured.peek().get(header).copied())
    }

    pub fn set(&self, header: &str, width: Option<f64>) {
        self.widths
            .set(with_width(&self.widths.peek(), header, width));
    }
}

/// A resizable column's entries in its column menu.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuWidth {
    pub resize: ColumnResize,
    pub limits: ResizeLimits,
    /// The resized width, `None` while the column has its own.
    pub width: Option<f64>,
    /// Says each step's new width: the open menu shows no change (todo 1415).
    pub announcer: Announcer,
}

impl MenuWidth {
    /// Widen and Narrow, which keep the menu open to step again, and Reset width.
    pub fn items(self, header: &str, labels: TableLabels) -> Vec<MenuEntry> {
        let Self {
            resize,
            limits,
            width,
            announcer,
        } = self;
        let step = |by: f64| {
            let header = header.to_string();
            move |_| {
                if let Some(width) = resize.width(&header) {
                    let next = limits.clamp(width + by);
                    resize.set(&header, Some(next));
                    announcer.say((labels.column_width)(&header, next));
                }
            }
        };
        let reset = header.to_string();
        vec![
            MenuItem::new(labels.widen_column)
                .disabled(width.is_some_and(|width| width >= limits.max))
                .keep_open()
                .onselect(step(MENU_STEP))
                .into(),
            MenuItem::new(labels.narrow_column)
                .disabled(width.is_some_and(|width| width <= limits.min))
                .keep_open()
                .onselect(step(-MENU_STEP))
                .into(),
            MenuItem::new(labels.reset_column_width)
                .disabled(width.is_none())
                .onselect(move |_| {
                    resize.set(&reset, None);
                    announcer.say((labels.column_width_reset)(&reset));
                })
                .into(),
        ]
    }
}

/// How far an arrow key moves a grip, in CSS px; Shift moves it `MENU_STEP`.
const KEY_STEP: f64 = 10.0;

/// What a key does to a grip's column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum KeyResize {
    By(f64),
    Min,
    Max,
}

/// An arrow moves the grip the way it points, so under RTL, where the grip is
/// the column's left edge, Left widens.
pub(super) fn key_resize(key: &Key, shift: bool, rtl: bool) -> Option<KeyResize> {
    let step = if shift { MENU_STEP } else { KEY_STEP };
    let outward = if rtl { -step } else { step };
    match key {
        Key::ArrowRight => Some(KeyResize::By(outward)),
        Key::ArrowLeft => Some(KeyResize::By(-outward)),
        Key::Home => Some(KeyResize::Min),
        Key::End => Some(KeyResize::Max),
        _ => None,
    }
}

/// A header's resize grip at its inline end: a pointer drag, or a focusable
/// separator the arrow keys move. The column menu is the drag-free way too (WCAG 2.5.7).
#[component]
pub(super) fn ResizeHandle(
    index: usize,
    header: String,
    limits: ResizeLimits,
    resize: ColumnResize,
) -> Element {
    // Fills the header cell, to measure it.
    let cell = use_element();
    let grip = use_element();
    let mut base = use_hook(|| CopyValue::new(None::<f64>));
    let mut rtl = use_hook(|| CopyValue::new(false));
    let mut preview = resize.preview;
    let mut measured = resize.measured;
    // The rendered width, so `aria-valuenow` follows it.
    let mut shown = use_signal(|| None::<f64>);
    let onresize = {
        let header = header.clone();
        move |event: Event<ResizeData>| {
            if let Ok(size) = event.get_border_box_size() {
                measured.write().insert(header.clone(), size.width);
                if *shown.peek() != Some(size.width.round()) {
                    shown.set(Some(size.width.round()));
                }
            }
        }
    };
    let onkeydown = {
        let header = header.clone();
        move |event: Event<KeyboardData>| {
            let Some(step) = key_resize(&event.key(), event.modifiers().shift(), cell.is_rtl())
            else {
                return;
            };
            let Some(width) = resize.width(&header) else {
                return;
            };
            let next = match step {
                KeyResize::By(by) => limits.clamp(width + by),
                KeyResize::Min => limits.min,
                KeyResize::Max if limits.max.is_finite() => limits.max,
                KeyResize::Max => return,
            };
            // Not the scroll area's arrow keys too.
            event.prevent_default();
            event.stop_propagation();
            resize.set(&header, Some(next));
        }
    };
    use_resize_fallback(cell, onresize.clone());
    let start_header = header.clone();
    let end_header = header.clone();
    // At once, not past a slop: the grip is too narrow to keep an uncaptured pointer.
    let drag = use_drag(DragOptions {
        capture: grip,
        onstart: Callback::new(move |start: DragStart| {
            rtl.set(cell.is_rtl());
            base.set(resize.width(&start_header));
            if base.peek().is_some() {
                return;
            }
            // Not measured yet (Blitz's first frames): read it, started here.
            let read = cell.dimensions();
            spawn(async move {
                match read.await {
                    Ok(size) => base.set(Some(size.width)),
                    Err(_) => start.cancel.call(()),
                }
            });
        }),
        onmove: Callback::new(move |step: DragMove| {
            let Some(base) = *base.peek() else {
                return;
            };
            // The grip sits on the inline end: under RTL, the left.
            let delta = match *rtl.peek() {
                true => -step.delta().x,
                false => step.delta().x,
            };
            preview.set(Some((index, limits.clamp(base + delta))));
        }),
        onend: Callback::new(move |()| {
            if let Some((_, width)) = preview.take() {
                resize.set(&end_header, Some(width));
            }
        }),
    });
    let dragging = (drag.dragging)();
    // The resized width at once; the measured one lags a layout behind it.
    let now = resize.widths.read().get(&header).copied().or(shown());
    // Siblings: Blitz hits nothing under a `pointer-events: none` box.
    rsx! {
        div {
            "data-resize-box": true,
            aria_hidden: "true",
            onmounted: cell.mount(),
            onresize,
        }
        div {
            "data-resize-handle": true,
            "data-dragging": dragging.then_some(true),
            role: "separator",
            tabindex: "0",
            aria_orientation: "vertical",
            aria_label: (resize.labels.resize_column)(&header),
            aria_valuenow: now.map(|width| width.round().to_string()),
            aria_valuemin: limits.min.to_string(),
            aria_valuemax: limits.max.is_finite().then(|| limits.max.to_string()),
            onmounted: grip.mount(),
            onkeydown,
            onpointerdown: move |event| drag.onpointerdown.call(event),
            onpointermove: move |event| drag.onpointermove.call(event),
            onpointerup: move |event| drag.onpointerup.call(event),
            onpointercancel: move |event| drag.onpointercancel.call(event),
            // Back to the column's own width.
            ondoubleclick: move |_| resize.set(&header, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_clamp_to_whole_px_and_never_cross() {
        let limits = ResizeLimits::new(80.0, 200.0);

        assert_eq!(limits.clamp(10.0), 80.0);
        assert_eq!(limits.clamp(120.4), 120.0);
        assert_eq!(limits.clamp(999.0), 200.0);
        assert_eq!(ResizeLimits::new(100.0, 40.0).clamp(10.0), 100.0);
        assert_eq!(ResizeLimits::default().clamp(5000.0), 5000.0);
    }

    #[test]
    fn arrows_move_the_grip_the_way_they_point() {
        assert_eq!(
            key_resize(&Key::ArrowRight, false, false),
            Some(KeyResize::By(10.0))
        );
        assert_eq!(
            key_resize(&Key::ArrowLeft, true, false),
            Some(KeyResize::By(-50.0))
        );
        // Under RTL the grip is the left edge: Left widens.
        assert_eq!(
            key_resize(&Key::ArrowLeft, false, true),
            Some(KeyResize::By(10.0))
        );
        assert_eq!(key_resize(&Key::Home, false, true), Some(KeyResize::Min));
        assert_eq!(key_resize(&Key::Enter, false, false), None);
    }

    #[test]
    fn a_width_sets_or_drops_its_entry() {
        let widths = ColumnWidths::from([("A".to_string(), 100.0)]);

        assert_eq!(with_width(&widths, "B", Some(60.0)).len(), 2);
        assert_eq!(with_width(&widths, "A", Some(60.0))["A"], 60.0);
        assert!(with_width(&widths, "A", None).is_empty());
    }
}
