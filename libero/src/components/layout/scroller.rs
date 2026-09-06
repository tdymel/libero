use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::{
    components::{
        Box, HtmlTag, Input, States, Variables,
        common::{ChevronDownIcon, base_props, focus_ring_sx, input_from_str, states, variables},
        layout::{
            ScrollArea, ScrollAreaBase, ScrollAreaHandle, scroll_area_base, use_box,
            use_scroll_area,
        },
    },
    hooks::{DragMove, DragOptions, DragStart, use_drag, use_id, use_theme},
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{SCROLLER_CONTROL, SCROLLER_FADE, ScrollerDefaults, Size},
};

pub use crate::theme::ScrollerControls;

input_from_str!(ScrollerControls);

/// Mantine's tolerance. A fractional device-pixel offset makes an exact
/// comparison flicker a control on and off at rest.
const EDGE_TOLERANCE: f64 = 1.0;

/// How far a mouse has to travel before a press becomes a drag. Below it the
/// press stays a click on whatever is under it.
const DRAG_THRESHOLD: f64 = 5.0;

/// `Burger`'s and `Loader`'s spelling, deliberately identical.
const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

/// Whether the strip rests against either end. Both `true` means nothing
/// overflows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollerEdges {
    pub at_start: bool,
    pub at_end: bool,
}

impl ScrollerEdges {
    /// Where every strip starts, before anything is measured: nothing known
    /// to overflow, so no control offers to go anywhere.
    const UNMEASURED: Self = Self {
        at_start: true,
        at_end: true,
    };

    /// From the scroll offset and the two widths, in px - compared in px
    /// rather than as a percent of the range, so the tolerance is one pixel
    /// on any length of strip.
    fn measure(offset: f64, scroll_width: f64, client_width: f64) -> Self {
        let max = (scroll_width - client_width).max(0.0);
        Self {
            at_start: offset <= EDGE_TOLERANCE,
            at_end: offset >= max - EDGE_TOLERANCE,
        }
    }
}

/// Where one control press lands, clamped to the scrollable range.
fn step_target(offset: f64, amount: f64, forward: bool, max: f64) -> f64 {
    let target = match forward {
        true => offset + amount,
        false => offset - amount,
    };
    target.clamp(0.0, max.max(0.0))
}

/// Steps a [`Scroller`] from an event handler, the way its own controls do.
///
/// For a strip whose buttons are the caller's own: `controls: "never"`,
/// `onedgechange` for their state, and a handle to move it.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Scroller, use_scroller};
/// # fn app() -> Element {
/// let strip = use_scroller();
/// rsx! {
///     Scroller { handle: strip, controls: "never", aria_label: "Tags", "…" }
///     Button { onclick: move |_| strip.step_back(), "Back" }
///     Button { onclick: move |_| strip.step_forward(), "Forward" }
/// }
/// # }
/// ```
///
/// `Copy`, so any number of handlers can hold it. A call before the bound
/// `Scroller` has mounted, or with none bound at all, does nothing. Bind it
/// from the first render: the element is attached on mount, so a handle
/// passed to an already mounted `Scroller` stays unattached.
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollerHandle {
    area: ScrollAreaHandle,
    /// The bound `Scroller`'s `scroll_amount`, written on every render. Not a
    /// signal: nothing renders from it.
    amount: CopyValue<f64>,
}

/// A handle for one [`Scroller`]. Pass it as that scroller's `handle`.
pub fn use_scroller() -> ScrollerHandle {
    ScrollerHandle {
        area: use_scroll_area(),
        amount: use_hook(|| CopyValue::new(0.0)),
    }
}

impl ScrollerHandle {
    /// One step towards the end, as the forward control does.
    pub fn step_forward(&self) {
        self.step(true);
    }

    /// One step towards the start, as the backward control does.
    pub fn step_back(&self) {
        self.step(false);
    }

    /// From where the strip is now, so a touch scroll in between is honoured.
    fn step(&self, forward: bool) {
        let (area, viewport, amount) = (self.area, self.area.element, *self.amount.peek());
        if !viewport.is_mounted() {
            return;
        }
        let (offset, content, size) = (
            viewport.scroll_offset(),
            viewport.scroll_size(),
            viewport.dimensions(),
        );
        spawn(async move {
            if let (Ok((x, _)), Ok(content), Ok(size)) = (offset.await, content.await, size.await) {
                area.scroll_to(
                    step_target(x, amount, forward, content.width - size.width),
                    0.0,
                );
            }
        });
    }
}

static SCROLLER_ROOT_SX: StaticSx = StaticSx::new(|| {
    ScrollerDefaults::theme_vars()
        // The controls' containing block.
        .position("relative")
        .display("block")
        // A viewport takes its width from its container, never from its
        // content - `Carousel`'s root, same reason.
        .width("100%")
        // A stacking context of its own, so the controls' `z-index` stays
        // inside the strip rather than competing with the page's layers.
        .z_index("0")
});

/// Merged onto `ScrollArea`'s own, which scrolls the x axis and hides the
/// scrollbar: the controls are the affordance, the strip still scrolls
/// natively.
static SCROLLER_VIEWPORT_SX: StaticSx = StaticSx::new(|| {
    // `ScrollArea` fills its parent's height; a strip is as tall as its
    // content.
    scroll_area_base(
        sx().height("auto")
            // A strip inside a scrolling page should not hand the scroll on when
            // it reaches its own end.
            .overscroll_behavior_x("contain")
            .scroll_behavior("smooth")
            // Chrome and Safari do not switch smooth scrolling off under reduced
            // motion - only Firefox does - so the guard is explicit.
            .media(REDUCED_MOTION, sx().scroll_behavior("auto"))
            .when("draggable", sx().cursor("grab"))
            // A drag is the pointer's own position: animating towards it lags.
            // After the smooth declaration, which it has to beat at equal
            // specificity.
            .when(
                "dragging",
                sx().scroll_behavior("auto")
                    .cursor("grabbing")
                    .user_select("none"),
            )
            // Outset: the root does not clip, and an inset ring would run under
            // the two controls.
            .focus_visible(focus_ring_sx()),
    )
});

/// `max-content` so the wrapper is as wide as the strip it holds, which is
/// what makes its `onresize` fire when the content changes width. Never
/// narrower than the viewport, so it also reports the viewport growing.
static SCROLLER_CONTENT_SX: StaticSx =
    StaticSx::new(|| sx().width("max-content").min_width("100%"));

static SCROLLER_CONTROL_SX: StaticSx = StaticSx::new(|| {
    let fade = SCROLLER_FADE.overridable();

    sx().position("absolute")
        .top("0")
        .bottom("0")
        .z_index("1")
        .display("flex")
        .align_items("center")
        .width(SCROLLER_CONTROL.value())
        .padding("0")
        .border_width("0")
        .color("inherit")
        .cursor("pointer")
        .transition("opacity 150ms")
        .media(REDUCED_MOTION, sx().transition("none"))
        // The gradient is the button's own background: one box is both the
        // fade and the hit target.
        .when(
            "start",
            sx().left("0")
                .justify_content("flex-start")
                .background(format!(
                    "linear-gradient(to right, {fade} 40%, transparent)"
                )),
        )
        .when(
            "end",
            sx().right("0")
                .justify_content("flex-end")
                .background(format!("linear-gradient(to left, {fade} 40%, transparent)")),
        )
        // The glyph points down; a quarter turn either way points it along
        // the strip.
        .selector("& svg", sx().width("60%").height("60%").flex("none"))
        .when(
            "start",
            sx().selector("& svg", sx().transform("rotate(90deg)")),
        )
        .when(
            "end",
            sx().selector("& svg", sx().transform("rotate(-90deg)")),
        )
        // The glyph dims, not the button: the button's opacity would dim its
        // focus ring with it.
        .when(
            "disabled",
            sx().cursor("default")
                .selector("& svg", sx().opacity("0.4")),
        )
        // Under `Auto` a control at its own end is gone - unless the keyboard
        // is on it, in which case it stays until the reader tabs away, so it
        // never vanishes from under the focus.
        .when(
            "controls-auto && disabled",
            sx().selector(
                "&:not(:focus-visible)",
                sx().opacity("0").pointer_events("none"),
            ),
        )
        .focus_visible(focus_ring_sx().outline_offset("-2px"))
});

fn scroller_variables(fade: Option<&ThemeAwareValue>) -> Variables {
    variables().with(
        SCROLLER_FADE.override_var(),
        fade.and_then(|value| value.resolve(None)),
    )
}

base_props! {
    pub struct ScrollerProps {
        /// Names the scrollable region, which is a tab stop.
        #[props(into)]
        aria_label: String,
        /// Pixels one control press scrolls. Defaults to the theme's.
        #[props(default)]
        scroll_amount: Option<u32>,
        /// `"auto"` (default), `"always"` or `"never"`.
        #[props(default, into)]
        controls: Input<ScrollerControls>,
        /// The width of each control strip.
        #[props(default, into)]
        control_size: Input<Size>,
        /// What the gradient under a control fades from - the surface the
        /// strip sits on. Defaults to the paper background.
        #[props(default, into)]
        fade_color: Input<ThemeAwareValue>,
        /// Mouse drag-to-pan. Touch and trackpad already scroll natively, and
        /// this never applies `touch-action: none`, which would stop them.
        #[props(default)]
        draggable: Option<bool>,
        /// Fires when either edge state flips, including the first
        /// measurement.
        #[props(default)]
        onedgechange: Option<EventHandler<ScrollerEdges>>,
        /// From [`use_scroller`], to step the strip from the caller's own
        /// buttons.
        #[props(default)]
        handle: Option<ScrollerHandle>,
        children: Element,
    }
}

/// A horizontal strip with a hidden scrollbar and a step control overlaid at
/// each end, shown while there is more content that way.
///
/// The scrolling is the browser's own, so touch, trackpad and the arrow keys
/// on the focused strip work untouched; the controls step by `scroll_amount`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Chip, Flex, Scroller};
/// # fn app() -> Element {
/// # const TAGS: [&str; 2] = ["rust", "ui"];
/// # rsx! {
/// Scroller { aria_label: "Tags",
///     Flex { direction: "row", gap: "sm",
///         for tag in TAGS { Chip { "{tag}" } }
///     }
/// }
/// # } }
/// ```
#[component]
pub fn Scroller(props: ScrollerProps) -> Element {
    let theme = use_theme();
    // Always called, so the hook order does not depend on the prop.
    let own = use_scroller();
    let handle = props.handle.unwrap_or(own);
    let (area, viewport) = (handle.area, handle.area.element);
    let viewport_id = use_id();

    let controls = props.controls.copied_or(theme.scroller.controls);
    let size = props.control_size.copied_or(theme.scroller.control_size);
    let mut amount = handle.amount;
    amount.set(props.scroll_amount.unwrap_or(theme.scroller.scroll_amount) as f64);
    let draggable = props.draggable.unwrap_or(theme.scroller.draggable);

    let mut edges = use_signal(|| ScrollerEdges::UNMEASURED);
    let onedgechange = props.onedgechange;
    let mut update = move |next: ScrollerEdges| {
        if next != *edges.peek() {
            edges.set(next);
            if let Some(handler) = &onedgechange {
                handler.call(next);
            }
        }
    };

    // Edges are compared in px, and `ScrollArea` reports a scroll as a percent,
    // so every report is a fresh read.
    let measure = move || {
        let (offset, content, size) = (
            viewport.scroll_offset(),
            viewport.scroll_size(),
            viewport.dimensions(),
        );
        spawn(async move {
            if let (Ok((x, _)), Ok(content), Ok(size)) = (offset.await, content.await, size.await) {
                update(ScrollerEdges::measure(x, content.width, size.width));
            }
        });
    };

    // `ResizeObserver` delivers an initial observation, so this is also the
    // mount-time measurement - before any scroll, nothing else says whether
    // the strip overflows at all.
    let resized = move |_: Event<ResizeData>| measure();

    // Mouse drag-to-pan. Deliberately **not** given `drag_handle_sx()`: that
    // is `touch-action: none`, and it would take away the native touch scroll
    // this strip exists for. A finger scrolls the platform's way, a mouse
    // drags.
    //
    // The press is held back until it travels `DRAG_THRESHOLD`, and only then
    // handed to `use_drag`, which takes pointer capture. Capture retargets the
    // click that follows a release onto the strip, so taken on the press it
    // would swallow every plain click on an item; taken after a real drag it
    // is exactly what stops the item under the pointer from activating.
    let mut press = use_signal(|| None::<Event<PointerData>>);
    let mut origin = use_signal(|| 0.0_f64);
    let drag = use_drag(DragOptions {
        capture: viewport,
        onstart: Callback::new(|_: DragStart| {}),
        onmove: Callback::new(move |moved: DragMove| {
            area.scroll_to((origin() - moved.delta().x).max(0.0), 0.0);
        }),
        onend: Callback::new(|()| {}),
    });

    let onpointerdown = move |event: Event<PointerData>| {
        if !draggable
            || event.data().pointer_type() != "mouse"
            || matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary)
        {
            return;
        }
        // Keeps a text selection or a native image or link drag from claiming
        // the pointer. The click still fires.
        event.prevent_default();
        // Read now, not when the drag starts: the read resolves in a task, and
        // the first move would otherwise run against the last drag's origin.
        let offset = viewport.scroll_offset();
        spawn(async move {
            if let Ok((x, _)) = offset.await {
                origin.set(x);
            }
        });
        press.set(Some(event));
    };

    let onpointermove = move |event: Event<PointerData>| {
        if *drag.dragging.peek() {
            drag.onpointermove.call(event);
            return;
        }
        let Some(down) = press.peek().clone() else {
            return;
        };
        let travelled = event.client_coordinates().x - down.client_coordinates().x;
        if travelled.abs() > DRAG_THRESHOLD {
            press.set(None);
            // The original press, so the drag measures from where the pointer
            // went down and the strip catches up with it at once.
            drag.onpointerdown.call(down);
            drag.onpointermove.call(event);
        }
    };

    let onpointerup = move |event: Event<PointerData>| {
        press.set(None);
        drag.onpointerup.call(event);
    };

    let viewport_states: Input<States> = states()
        .with("draggable", draggable)
        .with("dragging", (drag.dragging)())
        .into();

    let content = use_box()
        .framework_sx(&SCROLLER_CONTENT_SX)
        .prepare()
        .event("onresize", resized)
        .render(HtmlTag::Div, Vec::new(), props.children)?;

    let strip = rsx! {
        ScrollArea {
            handle: area,
            scrollbars: "horizontal",
            scrollbar_visibility: "hidden",
            // A tab stop, so a strip of plain images or text can still be
            // reached and scrolled with the arrow keys - and so it needs a
            // name.
            focusable: true,
            framework_sx: ScrollAreaBase(&SCROLLER_VIEWPORT_SX),
            states: viewport_states,
            id: viewport_id(),
            role: "region",
            aria_label: props.aria_label,
            onscroll: move |_| measure(),
            onresize: resized,
            onpointerdown,
            onpointermove,
            onpointerup,
            onpointercancel: onpointerup,
            {content}
        }
    };

    let current = edges();
    let viewport_id = viewport_id();
    let control = |forward: bool| -> Element {
        let at_edge = match forward {
            true => current.at_end,
            false => current.at_start,
        };
        let control_states: Input<States> = states()
            .with(if forward { "end" } else { "start" }, true)
            .with(controls.state_name(), true)
            .with("disabled", at_edge)
            .into();
        rsx! {
            Box {
                component: "button",
                r#type: "button",
                framework_sx: &SCROLLER_CONTROL_SX,
                states: control_states,
                aria_controls: viewport_id.clone(),
                aria_label: match forward {
                    true => theme.scroller.scroll_end_label,
                    false => theme.scroller.scroll_start_label,
                },
                // `aria-disabled`, not `disabled`: a focused button that
                // becomes `disabled` drops focus to the page. Out of the tab
                // order instead, since there is nothing to do there.
                aria_disabled: at_edge.to_string(),
                tabindex: if at_edge { "-1" } else { "0" },
                onclick: move |_| {
                    if !at_edge {
                        handle.step(forward);
                    }
                },
                ChevronDownIcon {}
            }
        }
    };

    let root_states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(controls.state_name(), true)
        .into();
    let root_variables: Input<Variables> = scroller_variables(props.fade_color.as_ref()).into();

    let show_controls = controls != ScrollerControls::Never;

    use_box()
        .framework_sx(&SCROLLER_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&root_states)
        .variables(&root_variables)
        .prepare()
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                if show_controls {
                    {control(false)}
                }
                {strip}
                if show_controls {
                    {control(true)}
                }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// 1000px of content in a 400px viewport: a 600px range.
    #[test]
    fn edges_are_read_in_px_with_a_one_pixel_tolerance() {
        let at = |offset| ScrollerEdges::measure(offset, 1000.0, 400.0);

        assert_eq!(
            at(0.0),
            ScrollerEdges {
                at_start: true,
                at_end: false
            }
        );
        // A fractional device-pixel rest still counts as the edge.
        assert!(at(0.6).at_start);
        assert!(!at(1.5).at_start);
        assert!(!at(598.5).at_end);
        assert!(at(599.4).at_end);
        assert!(at(600.0).at_end);
    }

    /// Content that fits rests against both ends, so under `Auto` neither
    /// control shows.
    #[test]
    fn content_that_fits_is_at_both_edges() {
        assert_eq!(
            ScrollerEdges::measure(0.0, 400.0, 400.0),
            ScrollerEdges::UNMEASURED
        );
        // A bounding width a fraction wider than the integer scroll width.
        assert_eq!(
            ScrollerEdges::measure(0.0, 400.0, 400.5),
            ScrollerEdges::UNMEASURED
        );
    }

    /// Each press steps from where the strip is now, not from where the last
    /// press aimed - a touch scroll in between is honoured.
    #[test]
    fn a_step_moves_from_the_current_offset_and_clamps() {
        assert_eq!(step_target(0.0, 200.0, true, 600.0), 200.0);
        assert_eq!(step_target(150.0, 200.0, true, 600.0), 350.0);
        assert_eq!(step_target(500.0, 200.0, true, 600.0), 600.0);
        assert_eq!(step_target(150.0, 200.0, false, 600.0), 0.0);
        assert_eq!(step_target(0.0, 200.0, true, -3.0), 0.0);
    }

    /// Chrome and Safari keep smooth scrolling under reduced motion, and a
    /// drag has to switch it off after the base declaration switched it on.
    #[test]
    fn smooth_scrolling_yields_to_reduced_motion_and_to_a_drag() {
        let css = Stylesheet::from(&SCROLLER_VIEWPORT_SX);
        let css = css.as_str();
        let smooth = css.find("scroll-behavior:smooth").expect("smooth: {css}");
        let reduced = css.find(REDUCED_MOTION).expect("a reduced-motion block");
        let dragging = css
            .find("[data-state~=\"dragging\"]")
            .expect("a dragging arm");

        assert!(smooth < reduced && smooth < dragging, "{css}");
        assert!(
            !css.contains("touch-action"),
            "touch must keep scrolling: {css}"
        );
    }
}
