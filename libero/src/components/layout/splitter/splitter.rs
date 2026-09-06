use dioxus::prelude::*;

use super::divider::{SPLITTER_DIVIDER_COLOR_VAR, SplitterDivider};
use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, Orientation, States, Variables,
        common::{base_props, use_name_warning, variables},
        layout::use_box,
    },
    hooks::{DragMove, DragOptions, DragStart, use_css, use_drag, use_element, use_id, use_theme},
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, Size},
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

/// `value`, or `fallback` with a warning - a bad number from a caller falls
/// back, it does not crash the page. The theme's own defaults are finite, so
/// the fallback always is.
fn finite_or(value: f64, fallback: f64, message: &str) -> f64 {
    if value.is_finite() {
        return value;
    }
    warn(message);
    fallback
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

fn splitter_variables(a: f64, divider_color: Option<&ThemeAwareValue>) -> Variables {
    variables()
        .with(SPLITTER_A_VAR, Some(format!("{a}%")))
        .with(
            SPLITTER_DIVIDER_COLOR_VAR,
            divider_color.and_then(|v| v.resolve(None)),
        )
}

/// Fork only: `panel_a`/`panel_b` win, otherwise exactly 2 children fill what
/// they leave. A missing pane renders empty, extras are dropped; both warn.
#[cfg(feature = "dioxus-fork")]
fn resolve_panels(
    panel_a: Option<Element>,
    panel_b: Option<Element>,
    children: Vec<Element>,
) -> (Element, Element) {
    use crate::components::Box;

    if let (Some(a), Some(b)) = (panel_a, panel_b) {
        return (a, b);
    }

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
        /// afterward - `onresize` only notifies.
        initial_size: f64,
        /// % floor applied to both panes, capped at 50.
        #[props(default, into)]
        min_size: Input<f64>,
        #[props(default, into)]
        divider_size: Input<Size>,
        #[props(default, into)]
        divider_color: Input<ThemeAwareValue>,
        #[props(default)]
        onresize: Option<EventHandler<SplitterResizeEvent>>,
        /// Names the divider, which is a focusable `role="separator"` - after
        /// the pane it resizes, e.g. `"Resize sidebar"`. An `aria_label` in
        /// `attributes` would land on the root instead. Unset is a `warn()`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Pane A (left/top). Required against upstream main, which cannot
        /// split `children` apart; on the fork it falls back to the first
        /// child.
        #[cfg(feature = "dioxus-fork")]
        #[props(default)]
        panel_a: Option<Element>,
        #[cfg(not(feature = "dioxus-fork"))]
        panel_a: Element,
        /// Pane B (right/bottom). Same rules as `panel_a`, second child.
        #[cfg(feature = "dioxus-fork")]
        #[props(default)]
        panel_b: Option<Element>,
        #[cfg(not(feature = "dioxus-fork"))]
        panel_b: Element,
        /// Fork only: exactly 2, `A` then `B`. `panel_a`/`panel_b` win.
        #[cfg(feature = "dioxus-fork")]
        children: Vec<Element>,
    }
}

/// Splits two panes with a draggable/keyboard-resizable divider. Only two
/// panes - nest another `Splitter` inside a pane for more.
#[component]
pub fn Splitter(props: SplitterProps) -> Element {
    let theme = use_theme();
    let root = use_element();
    // The divider controls pane A, so it points there.
    let panel_a_id = use_id();
    use_name_warning(
        props.aria_label.is_some(),
        "Splitter: no `aria_label`, so the divider is announced as just \"separator\" \
         and a number.",
    );
    #[cfg(feature = "dioxus-fork")]
    let (panel_a, panel_b) = { resolve_panels(props.panel_a, props.panel_b, props.children) };
    #[cfg(not(feature = "dioxus-fork"))]
    let (panel_a, panel_b) = (props.panel_a, props.panel_b);

    let orientation = props.orientation.copied_or(Orientation::Vertical);
    let vertical = orientation == Orientation::Vertical;

    // Both panes get the same floor, so anything above 50 leaves no range -
    // and `f64::clamp` asserts `min <= max`. `clamp` keeps a NaN receiver, so
    // a non-finite floor has to be dropped before it becomes a bound below.
    let min_size = finite_or(
        props.min_size.copied_or(theme.splitter.min_size),
        theme.splitter.min_size,
        "Splitter: `min_size` must be a finite number.",
    )
    .clamp(0.0, 50.0);
    let size = props.divider_size.copied_or(theme.splitter.size);

    // `NaN` survives a `clamp` as its receiver, and `a / (a + b)` is `NaN`
    // when both are 0 - which would write `--lsx-splitter-a:NaN%` and leave
    // both panes without a size.
    let initial_size = finite_or(
        props.initial_size,
        50.0,
        "Splitter: `initial_size` must be a finite number.",
    );
    let mut a = use_signal(|| initial_size.clamp(min_size, 100.0 - min_size));
    // Container width (vertical) or height (horizontal), measured once at
    // pointerdown to turn the drag's pixel delta into a percentage.
    let mut container_size = use_signal(|| 0.0_f64);
    let mut start_a = use_signal(|| 0.0_f64);

    let onresize = props.onresize;
    let notify = move |event: SplitterResizeEvent| {
        if let Some(onresize) = &onresize {
            onresize.call(event);
        }
    };

    let drag = use_drag(DragOptions {
        capture: root,
        onstart: Callback::new(move |start: DragStart| {
            let cancel = start.cancel;
            // Started here, awaited in the task: see `ElementApi::dimensions`.
            let size = root.dimensions();
            spawn(async move {
                let Ok(dimensions) = size.await else {
                    cancel.call(());
                    return;
                };
                let size = if vertical {
                    dimensions.width
                } else {
                    dimensions.height
                };
                if size <= 0.0 {
                    cancel.call(());
                    return;
                }

                container_size.set(size);
                start_a.set(a());
                notify(SplitterResizeEvent::Start(a(), 100.0 - a()));
            });
        }),
        onmove: Callback::new(move |event: DragMove| {
            let delta = event.delta();
            let pixels = if vertical { delta.x } else { delta.y };
            let delta_pct = pixels / container_size() * 100.0;
            let new_a = (start_a() + delta_pct).clamp(min_size, 100.0 - min_size);
            a.set(new_a);
            notify(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
        }),
        onend: Callback::new(move |_| {
            notify(SplitterResizeEvent::End(a(), 100.0 - a()));
        }),
    });

    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        let step = if event.modifiers().shift() {
            theme.splitter.big_step
        } else {
            theme.splitter.step
        };

        let current = a();
        // `Change` then `End`: a key press settles on its size at once, and
        // `End` is where the docs tell a caller to persist the layout.
        let mut go_to = |new_a: f64| {
            event.prevent_default();
            let new_a = new_a.clamp(min_size, 100.0 - min_size);
            a.set(new_a);
            notify(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
            notify(SplitterResizeEvent::End(new_a, 100.0 - new_a));
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
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("dragging", (drag.dragging)())
        .into();

    let variables: Input<Variables> = splitter_variables(a(), props.divider_color.as_ref()).into();

    // The two panes carry a static framework style and nothing else, so they
    // are classes on plain elements rather than component scopes.
    let panel_a_class = use_css(Some(&SPLITTER_PANEL_A_SX), CssLayer::Framework);
    let panel_b_class = use_css(Some(&SPLITTER_PANEL_B_SX), CssLayer::Framework);

    use_box()
        .framework_sx(&SPLITTER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .element(&root)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { id: panel_a_id, class: panel_a_class, {panel_a} }
                SplitterDivider {
                    a,
                    vertical,
                    size,
                    min_size,
                    aria_label: props.aria_label,
                    controls: panel_a_id(),
                    onpointerdown: drag.onpointerdown,
                    onkeydown,
                }
                div { class: panel_b_class, {panel_b} }
            },
        )
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
