use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, Orientation, States, Variables,
        common::{base_props, dom_api, variables, warn},
    },
    hooks::{use_id, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, CssVar, SPLITTER_DIVIDER_SIZE, SPLITTER_HIT_SIZE, Size},
};

/// Fired as the divider is dragged/keyed - both panes' resulting sizes, in
/// percent of the container, with `A` (left/top) first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitterResizeEvent {
    Start(f64, f64),
    Change(f64, f64),
    End(f64, f64),
}

const SPLITTER_A_VAR: CssVar = CssVar::new("--lsx-splitter-a");
const SPLITTER_DIVIDER_COLOR_VAR: CssVar = CssVar::new("--lsx-splitter-divider-color");

#[derive(Clone, Copy, Debug, PartialEq)]
struct DragState {
    /// `client_x` (vertical orientation) or `client_y` (horizontal) at
    /// `mousedown`.
    start_client: f64,
    start_a: f64,
    /// Container's width (vertical) or height (horizontal) in px, measured
    /// once at `mousedown` - needed to convert the drag's pixel delta into a
    /// percent of the container.
    container_size: f64,
}

static SPLITTER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .height("100%")
        .when("horizontal", sx().flex_direction("column"))
        .when("vertical", sx().flex_direction("row"))
        .when("dragging", sx().user_select("none"))
});

static SPLITTER_PANEL_A_SX: StaticSx = StaticSx::new(|| {
    sx().flex(format!("0 0 {}", SPLITTER_A_VAR.value_or("50%")))
        .min_width("0")
        .min_height("0")
});

static SPLITTER_PANEL_B_SX: StaticSx =
    StaticSx::new(|| sx().flex("1 1 0%").min_width("0").min_height("0"));

// The layout-participating element - only as wide/tall as `divider_size`, so
// the panes sit flush against it with no visible gap. The larger pointer/
// keyboard hit target is a child overlay that doesn't take up flex space of
// its own (see `SPLITTER_HIT_SX` below).
static SPLITTER_BAR_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .position("relative")
        .flex_shrink("0")
        .align_self("stretch")
        .background(SPLITTER_DIVIDER_COLOR_VAR.value_or(ColorCss::GREY.value(ColorShade::S4)));

    Size::ALL.into_iter().fold(base, |acc, size| {
        acc.when(
            format!("vertical && {}", size.state_name()),
            sx().width(SPLITTER_DIVIDER_SIZE.value(size)),
        )
        .when(
            format!("horizontal && {}", size.state_name()),
            sx().height(SPLITTER_DIVIDER_SIZE.value(size)),
        )
    })
});

// Invisible - extends symmetrically past the bar into both neighboring
// panes (via a negative inset, `divider_size` minus `hit_size`) so the
// actual clickable/draggable target is comfortably larger than the thin
// visible line, without the panes themselves leaving a gap for it.
static SPLITTER_HIT_SX: StaticSx = StaticSx::new(|| {
    let base = sx().position("absolute");

    Size::ALL.into_iter().fold(base, |acc, size| {
        let inset = format!(
            "calc(({} - {}) / 2)",
            SPLITTER_DIVIDER_SIZE.value(size),
            SPLITTER_HIT_SIZE.value(size)
        );
        acc.when(
            format!("vertical && {}", size.state_name()),
            sx().top("0")
                .bottom("0")
                .left(inset.clone())
                .right(inset.clone())
                .cursor("col-resize"),
        )
        .when(
            format!("horizontal && {}", size.state_name()),
            sx().left("0")
                .right("0")
                .top(inset.clone())
                .bottom(inset)
                .cursor("row-resize"),
        )
    })
});

fn splitter_variables(a: f64, divider_color: Option<&ThemeAwareValue>) -> Variables {
    variables()
        .with(SPLITTER_A_VAR, Some(format!("{a}%")))
        .with(
            SPLITTER_DIVIDER_COLOR_VAR,
            divider_color.and_then(|v| v.resolve(None)),
        )
}

/// Exactly 2 children expected (`A`/left-top, then `B`/right-bottom) - a
/// missing one renders as an empty pane, extras are dropped. Both cases
/// warn.
fn resolve_panels(children: Vec<Element>) -> (Element, Element) {
    let count = children.len();
    let mut children = children.into_iter();
    let panel_a = children.next();
    let panel_b = children.next();

    if children.next().is_some() {
        warn(&format!(
            "Splitter expects exactly 2 children, got {count} - extra children are ignored"
        ));
    } else if panel_a.is_none() || panel_b.is_none() {
        warn(&format!(
            "Splitter expects exactly 2 children, got {count} - missing pane(s) render empty"
        ));
    }

    (
        panel_a.unwrap_or_else(|| rsx! { Box {} }),
        panel_b.unwrap_or_else(|| rsx! { Box {} }),
    )
}

base_props! {
    pub struct SplitterProps {
        /// Divider line axis - `"vertical"` (default, side-by-side panes) or
        /// `"horizontal"` (stacked panes).
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Initial % of the left/top pane (A) - clamped to `min_size` at
        /// mount if lower. Uncontrolled afterward: drag/keyboard own the
        /// live value, `on_resize` only notifies.
        initial_size: f64,
        /// % floor applied to both panes - defaults to the theme's
        /// `splitter.min_size` setting.
        #[props(default, into)]
        min_size: Input<f64>,
        #[props(default, into)]
        divider_size: Input<Size>,
        #[props(default, into)]
        divider_color: Input<ThemeAwareValue>,
        #[props(default)]
        on_resize: EventHandler<SplitterResizeEvent>,
        /// Exactly 2 - `A` (left/top) then `B` (right/bottom). Nest another
        /// `Splitter` inside a pane for more than 2.
        children: Vec<Element>,
    }
}

/// Splits two panes with a draggable/keyboard-resizable divider. Only two
/// panes - nest another `Splitter` inside a pane for more.
#[component]
pub fn Splitter(props: SplitterProps) -> Element {
    let theme = use_theme();
    let root_id = use_id();
    let (panel_a, panel_b) = resolve_panels(props.children);

    let orientation = props.orientation.copied_or(Orientation::Vertical);
    let vertical = orientation == Orientation::Vertical;

    let min_size = props.min_size.copied_or(theme.splitter.min_size);
    let size = props.divider_size.copied_or(theme.splitter.size);

    let mut a = use_signal(|| props.initial_size.clamp(min_size, 100.0 - min_size));
    let mut drag = use_signal(|| Option::<DragState>::None);

    let on_resize = props.on_resize;

    // No `onmouseleave` - leaving the drag state alone when the cursor
    // exits the container just pauses updates (`onmousemove` only fires
    // while over it) rather than aborting the drag; it resumes if the
    // cursor comes back, and only `onmouseup` actually ends it.
    let onmousedown = move |event: Event<MouseData>| {
        event.prevent_default();
        let Ok(container) = dom_api().query_selector(&format!("#{}", root_id())) else {
            return;
        };
        let Ok(dimensions) = container.dimensions() else {
            return;
        };
        let container_size = if vertical {
            dimensions.width
        } else {
            dimensions.height
        };
        if container_size <= 0.0 {
            return;
        }
        let coordinates = event.client_coordinates();
        let start_client = if vertical {
            coordinates.x
        } else {
            coordinates.y
        };
        drag.set(Some(DragState {
            start_client,
            start_a: a(),
            container_size,
        }));
        on_resize.call(SplitterResizeEvent::Start(a(), 100.0 - a()));
    };

    let onmousemove = move |event: Event<MouseData>| {
        let Some(state) = drag() else {
            return;
        };
        let coordinates = event.client_coordinates();
        let client = if vertical {
            coordinates.x
        } else {
            coordinates.y
        };
        let delta_pct = (client - state.start_client) / state.container_size * 100.0;
        let new_a = (state.start_a + delta_pct).clamp(min_size, 100.0 - min_size);
        a.set(new_a);
        on_resize.call(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
    };

    let onmouseup = move |_| {
        if drag().is_some() {
            drag.set(None);
            on_resize.call(SplitterResizeEvent::End(a(), 100.0 - a()));
        }
    };

    let onkeydown = move |event: Event<KeyboardData>| {
        let step = if event.modifiers().shift() {
            theme.splitter.big_step
        } else {
            theme.splitter.step
        };

        let current = a();
        let mut go_to = |new_a: f64| {
            event.prevent_default();
            let new_a = new_a.clamp(min_size, 100.0 - min_size);
            a.set(new_a);
            on_resize.call(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
        };

        match event.key() {
            Key::ArrowRight if vertical => go_to(current + step),
            Key::ArrowLeft if vertical => go_to(current - step),
            Key::ArrowDown if !vertical => go_to(current + step),
            Key::ArrowUp if !vertical => go_to(current - step),
            Key::Home => go_to(min_size),
            Key::End => go_to(100.0 - min_size),
            _ => {}
        }
    };

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("dragging", drag().is_some());

    let divider_states = States::default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with(size.state_name(), true);

    let variables = splitter_variables(a(), props.divider_color.as_ref());
    let aria_orientation = if vertical { "vertical" } else { "horizontal" };

    rsx! {
        Box {
            id: "{root_id}",
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &SPLITTER_BASE_SX,
            attributes: props.attributes,
            onmousemove,
            onmouseup,
            Box {
                framework_sx: &SPLITTER_PANEL_A_SX,
                {panel_a}
            }
            Box {
                framework_sx: &SPLITTER_BAR_SX,
                states: divider_states.clone(),
                Box {
                    framework_sx: &SPLITTER_HIT_SX,
                    states: divider_states,
                    role: "separator",
                    tabindex: "0",
                    "aria-orientation": aria_orientation,
                    "aria-valuenow": "{a() as i64}",
                    "aria-valuemin": "{min_size as i64}",
                    "aria-valuemax": "{(100.0 - min_size) as i64}",
                    onmousedown,
                    onkeydown,
                }
            }
            Box {
                framework_sx: &SPLITTER_PANEL_B_SX,
                {panel_b}
            }
        }
    }
}
