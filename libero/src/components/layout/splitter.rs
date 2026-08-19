use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, Orientation, States, Variables,
        common::{base_props, dom_api, variables},
    },
    hooks::{DragMove, DragOptions, DragStart, drag_handle_sx, use_drag, use_root_id, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, CssVar, SPLITTER_DIVIDER_SIZE, SPLITTER_HIT_SIZE, Size},
    utils::warn,
};

/// Both panes' resulting sizes as percentages, `A` (left/top) first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitterResizeEvent {
    Start(f64, f64),
    Change(f64, f64),
    End(f64, f64),
}

const SPLITTER_A_VAR: CssVar = CssVar::new("--lsx-splitter-a");
const SPLITTER_DIVIDER_COLOR_VAR: CssVar = CssVar::new("--lsx-splitter-divider-color");

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

// Only as thick as `divider_size`, so the panes sit flush. The larger hit
// target is a child overlay taking no flex space of its own.
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

// Invisible, and negatively inset past the bar into both panes, so the drag
// target is larger than the visible line without the panes leaving a gap.
static SPLITTER_HIT_SX: StaticSx = StaticSx::new(|| {
    let base = sx().position("absolute").and(drag_handle_sx());

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

/// Exactly 2 children. A missing one renders empty, extras are dropped; both
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
        /// Initial % of pane A, clamped to `min_size` at mount. Uncontrolled
        /// afterward - `on_resize` only notifies.
        initial_size: f64,
        /// % floor applied to both panes, capped at 50.
        #[props(default, into)]
        min_size: Input<f64>,
        #[props(default, into)]
        divider_size: Input<Size>,
        #[props(default, into)]
        divider_color: Input<ThemeAwareValue>,
        #[props(default)]
        on_resize: EventHandler<SplitterResizeEvent>,
        /// Exactly 2: `A` (left/top) then `B`. Nest for more.
        children: Vec<Element>,
    }
}

/// Splits two panes with a draggable/keyboard-resizable divider. Only two
/// panes - nest another `Splitter` inside a pane for more.
#[component]
pub fn Splitter(props: SplitterProps) -> Element {
    let theme = use_theme();
    let root_id = use_root_id(&props.attributes);
    let (panel_a, panel_b) = resolve_panels(props.children);

    let orientation = props.orientation.copied_or(Orientation::Vertical);
    let vertical = orientation == Orientation::Vertical;

    // Both panes get the same floor, so anything above 50 leaves no range -
    // and `f64::clamp` asserts `min <= max`.
    let min_size = props
        .min_size
        .copied_or(theme.splitter.min_size)
        .clamp(0.0, 50.0);
    let size = props.divider_size.copied_or(theme.splitter.size);

    let mut a = use_signal(|| props.initial_size.clamp(min_size, 100.0 - min_size));
    // Container width (vertical) or height (horizontal), measured once at
    // pointerdown to turn the drag's pixel delta into a percentage.
    let mut container_size = use_signal(|| 0.0_f64);
    let mut start_a = use_signal(|| 0.0_f64);

    let on_resize = props.on_resize;

    let drag = use_drag(DragOptions {
        capture: root_id,
        on_start: Callback::new(move |_: DragStart| {
            let Ok(container) = dom_api().query_selector(&format!("#{}", root_id())) else {
                return false;
            };
            let Ok(dimensions) = container.dimensions() else {
                return false;
            };
            let size = if vertical {
                dimensions.width
            } else {
                dimensions.height
            };
            if size <= 0.0 {
                return false;
            }

            container_size.set(size);
            start_a.set(a());
            on_resize.call(SplitterResizeEvent::Start(a(), 100.0 - a()));
            true
        }),
        on_move: Callback::new(move |event: DragMove| {
            let delta = event.delta();
            let pixels = if vertical { delta.x } else { delta.y };
            let delta_pct = pixels / container_size() * 100.0;
            let new_a = (start_a() + delta_pct).clamp(min_size, 100.0 - min_size);
            a.set(new_a);
            on_resize.call(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
        }),
        on_end: Callback::new(move |_| {
            on_resize.call(SplitterResizeEvent::End(a(), 100.0 - a()));
        }),
    });

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
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("dragging", (drag.dragging)());

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
            onpointermove: drag.onpointermove,
            onpointerup: drag.onpointerup,
            onpointercancel: drag.onpointercancel,
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
                    onpointerdown: drag.onpointerdown,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue};

    /// It positions the divider, so there is no "unset" for it.
    #[test]
    fn the_pane_fraction_is_always_a_percentage() {
        let variables = splitter_variables(37.5, None);

        assert_eq!(
            variables.to_string(),
            format!("{}:37.5%;", SPLITTER_A_VAR.name())
        );
    }

    #[test]
    fn a_divider_color_is_appended_when_given() {
        let color = ThemeAwareValue::Color(Color::Grey);
        let variables = splitter_variables(50.0, Some(&color));

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:50%;{}:{};",
                SPLITTER_A_VAR.name(),
                SPLITTER_DIVIDER_COLOR_VAR.name(),
                ColorValue::Shade(Color::Grey, ColorShade::DEFAULT).value()
            )
        );
    }
}
