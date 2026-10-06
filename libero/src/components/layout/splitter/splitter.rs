use dioxus::prelude::*;

use super::divider::{SPLITTER_DIVIDER_COLOR_VAR, SplitterDivider};
use crate::{
    CssLayer,
    components::{
        common::{
            HtmlTag, Input, Orientation, States, Variables, base_props, has_shortcut_modifier,
            use_name_warning, variables,
        },
        layout::use_box,
    },
    hooks::{
        DragMove, DragOptions, DragStart, use_css, use_drag_with, use_element, use_id, use_theme,
    },
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, Size},
    utils::warn,
};

/// Both panes' resulting sizes as percentages, `A` (start/top) first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitterResizeEvent {
    Start(f64, f64),
    Change(f64, f64),
    End(f64, f64),
}

const SPLITTER_A_VAR: CssVar = CssVar::new("--lsx-splitter-a");

/// `value`, or `fallback` with a warning: a bad number doesn't crash the page.
fn finite_or(value: f64, fallback: f64, message: &str) -> f64 {
    if value.is_finite() {
        return value;
    }
    warn(message);
    fallback
}

/// How pane A grows per pixel rightwards: `-1` when a right-to-left row puts
/// it on the right.
fn inline_sign(rtl: bool) -> f64 {
    if rtl { -1.0 } else { 1.0 }
}

/// Where a double-click restores pane A to once a move took it from `from` to `to`:
/// `from` if that move reached the floor from above it, else the old point.
fn restore_point(from: f64, to: f64, min_size: f64, restore_to: f64) -> f64 {
    if to <= min_size && from > min_size {
        from
    } else {
        restore_to
    }
}

static SPLITTER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .height("100%")
        .when("horizontal", sx().flex_direction("column"))
        .when("vertical", sx().flex_direction("row"))
        .when("dragging", sx().user_select("none"))
});

// Each pane scrolls its own overflow: at its floor it would paint over the divider
// and the other pane, its focusables under them.
static SPLITTER_PANEL_A_SX: StaticSx = StaticSx::new(|| {
    sx().flex(format!("0 0 {}", SPLITTER_A_VAR.value_or("50%")))
        .min_width("0")
        .min_height("0")
        .overflow("auto")
});

static SPLITTER_PANEL_B_SX: StaticSx = StaticSx::new(|| {
    sx().flex("1 1 0%")
        .min_width("0")
        .min_height("0")
        .overflow("auto")
});

fn splitter_variables(a: f64, divider_color: Option<&ThemeAwareValue>) -> Variables {
    variables()
        .with(SPLITTER_A_VAR, Some(format!("{a}%")))
        .with(
            SPLITTER_DIVIDER_COLOR_VAR,
            divider_color.and_then(|v| v.resolve(None)),
        )
}

base_props! {
    pub struct SplitterProps {
        /// Divider line axis: `"vertical"` (default, side by side) or `"horizontal"` (stacked).
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Initial % of pane A; uncontrolled afterwards.
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
        /// Names the divider, after the pane it resizes, e.g. `"Resize sidebar"`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Pane A (start/top: the right under RTL).
        panel_a: Element,
        /// Pane B (end/bottom).
        panel_b: Element,
    }
}

/// Splits two panes with a divider resizable by drag or keyboard. Nest another
/// `Splitter` in a pane for more.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Splitter;
/// # fn app() -> Element {
/// rsx! {
///     Splitter {
///         initial_size: 30.0,
///         aria_label: "Resize sidebar",
///         panel_a: rsx! { "Sidebar" },
///         panel_b: rsx! { "Editor" },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/splitter>
#[component]
pub fn Splitter(props: SplitterProps) -> Element {
    let theme = use_theme();
    let root = use_element();
    let divider = use_element();
    // The divider controls pane A, so it points there.
    let panel_a_id = use_id();
    use_name_warning(
        props.aria_label.is_some(),
        "Splitter: no `aria_label`, so the divider is announced as just \"separator\" \
         and a number.",
    );
    let (panel_a, panel_b) = (props.panel_a, props.panel_b);

    let orientation = props.orientation.copied_or(Orientation::Vertical);
    let vertical = orientation == Orientation::Vertical;

    // Above 50 leaves no range and `f64::clamp` asserts `min <= max`; a NaN
    // survives `clamp`, so it's dropped first.
    let min_size = finite_or(
        props.min_size.copied_or(theme.splitter.min_size),
        theme.splitter.min_size,
        "Splitter: `min_size` must be a finite number.",
    )
    .clamp(0.0, 50.0);
    let size = props.divider_size.copied_or(theme.splitter.size);

    // A NaN survives `clamp` and would write `--lsx-splitter-a:NaN%`.
    let initial_size = finite_or(
        props.initial_size,
        50.0,
        "Splitter: `initial_size` must be a finite number.",
    );
    let mut a = use_signal(|| initial_size.clamp(min_size, 100.0 - min_size));
    // A `min_size` raised after mount pulls pane A up to the new floor.
    let bounded = move || a().clamp(min_size, 100.0 - min_size);
    // Measured at pointerdown, to turn the drag's px delta into a percentage.
    let mut container_size = use_signal(|| 0.0_f64);
    let mut start_a = use_signal(|| 0.0_f64);
    // Under RTL a row puts pane A on the right, so moving right shrinks it.
    let mut toward_b = use_hook(|| CopyValue::new(1.0_f64));

    // Where a double-click restores pane A to after any move took it to the floor.
    let mut restore_to = use_signal(|| initial_size.clamp(min_size, 100.0 - min_size));
    // Pointer capture retargets the `dblclick` to the root, so the root learns
    // from the bubbling press whether the divider took it.
    let mut divider_pressing = use_signal(|| false);
    let mut pressed_divider = use_signal(|| false);

    let onresize = props.onresize;
    let notify = move |event: SplitterResizeEvent| {
        if let Some(onresize) = &onresize {
            onresize.call(event);
        }
    };
    // A key press or double-click settles at once; `End` is where callers persist.
    let settle = move |new_a: f64| {
        let new_a = new_a.clamp(min_size, 100.0 - min_size);
        restore_to.set(restore_point(bounded(), new_a, min_size, restore_to()));
        a.set(new_a);
        notify(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
        notify(SplitterResizeEvent::End(new_a, 100.0 - new_a));
    };

    // A touch tap ends before the measure lands: (measuring, released).
    let mut starting = use_hook(|| CopyValue::new((false, false)));

    // A touch on a vertical divider drags once it moves sideways (1039).
    let drag = use_drag_with(
        DragOptions {
            capture: root,
            onstart: use_callback(move |start: DragStart| {
                let cancel = start.cancel;
                // Unmeasured until the task below lands: `onmove` skips moves till then.
                container_size.set(0.0);
                starting.set((true, false));
                toward_b.set(inline_sign(vertical && root.is_rtl()));
                // Started here, awaited in the task: see `ElementApi::dimensions`.
                let size = root.dimensions();
                spawn(async move {
                    let measured = size.await;
                    let (_, released) = starting.replace((false, false));
                    let size = match measured {
                        Ok(dimensions) if vertical => dimensions.width,
                        Ok(dimensions) => dimensions.height,
                        Err(_) => 0.0,
                    };
                    if size <= 0.0 {
                        cancel.call(());
                        return;
                    }

                    // `use_drag` gives the divider focus back only on the web, where
                    // it can see the press's target: this covers the other renderers.
                    let _ = divider.focus();
                    container_size.set(size);
                    let from = bounded();
                    start_a.set(from);
                    notify(SplitterResizeEvent::Start(from, 100.0 - from));
                    if released {
                        notify(SplitterResizeEvent::End(from, 100.0 - from));
                    }
                });
            }),
            onmove: use_callback(move |event: DragMove| {
                // A move before the measure would divide by 0 and write `NaN%`.
                if container_size() <= 0.0 {
                    return;
                }
                let delta = event.delta();
                let pixels = if vertical {
                    delta.x * toward_b()
                } else {
                    delta.y
                };
                let delta_pct = pixels / container_size() * 100.0;
                let new_a = (start_a() + delta_pct).clamp(min_size, 100.0 - min_size);
                a.set(new_a);
                notify(SplitterResizeEvent::Change(new_a, 100.0 - new_a));
            }),
            onend: use_callback(move |()| {
                if starting.peek().0 {
                    starting.set((true, true));
                    return;
                }
                let to = bounded();
                restore_to.set(restore_point(start_a(), to, min_size, restore_to()));
                notify(SplitterResizeEvent::End(to, 100.0 - to));
            }),
        },
        vertical,
    );

    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        // Alt+ArrowLeft is Back: a chord is the browser's, not a step.
        if has_shortcut_modifier(&event) {
            return;
        }
        let step = if event.modifiers().shift() {
            theme.splitter.big_step
        } else {
            theme.splitter.step
        };

        let current = bounded();
        let mut settle = settle;
        let mut go_to = |new_a: f64| {
            event.prevent_default();
            settle(new_a);
        };

        // The arrow moves the divider the way it points.
        let right = step * inline_sign(vertical && root.is_rtl());
        match event.key() {
            Key::ArrowRight if vertical => go_to(current + right),
            Key::ArrowLeft if vertical => go_to(current - right),
            Key::ArrowDown if !vertical => go_to(current + step),
            Key::ArrowUp if !vertical => go_to(current - step),
            Key::Home => go_to(min_size),
            Key::End => go_to(100.0 - min_size),
            _ => {}
        }
    });

    // WCAG 2.5.7's drag-free path: collapse pane A to the floor, or restore it.
    // A double-click, so the click that focuses the divider moves nothing.
    let toggle = move || {
        let mut settle = settle;
        let current = bounded();
        if current > min_size {
            settle(min_size);
        } else {
            // A restore point at the floor would restore nothing.
            let to = restore_to();
            settle(if to > min_size { to } else { 50.0 });
        }
    };
    let ondoubleclick = use_callback(move |_: Event<MouseData>| {
        // A double-click in a pane, a nested splitter's included, is not ours.
        if pressed_divider() {
            toggle();
        }
    });
    let ondividerdown = use_callback(move |event: Event<PointerData>| {
        divider_pressing.set(true);
        drag.onpointerdown.call(event);
    });
    let onrootdown = use_callback(move |_: Event<PointerData>| {
        pressed_divider.set(divider_pressing());
        divider_pressing.set(false);
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("dragging", (drag.dragging)())
        .into();

    let variables: Input<Variables> =
        splitter_variables(bounded(), props.divider_color.as_ref()).into();

    // Static styles only, so classes on plain elements, not component scopes.
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
        .event("onpointerdown", onrootdown)
        // The DOM's name: `.event` skips the rsx `ondoubleclick` mapping.
        .event("ondblclick", ondoubleclick)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { id: panel_a_id, class: panel_a_class, {panel_a} }
                SplitterDivider {
                    element: divider,
                    a,
                    vertical,
                    size,
                    min_size,
                    aria_label: props.aria_label,
                    controls: panel_a_id(),
                    onpointerdown: ondividerdown,
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

    /// Todo 2406: `Home`, a drag or a double-click to the floor all keep the size before it.
    #[test]
    fn any_move_to_the_floor_keeps_the_size_before_it() {
        assert_eq!(restore_point(70.0, 10.0, 10.0, 50.0), 70.0);
        // Within the range, or already at the floor: the old point stays.
        assert_eq!(restore_point(70.0, 60.0, 10.0, 50.0), 50.0);
        assert_eq!(restore_point(10.0, 10.0, 10.0, 70.0), 70.0);
    }

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
        let color = ThemeAwareValue::Color(Color::Muted);
        let variables = splitter_variables(50.0, Some(&color));

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:50%;{}:{};",
                SPLITTER_A_VAR.name(),
                SPLITTER_DIVIDER_COLOR_VAR.name(),
                ColorValue::Shade(Color::Muted, ColorShade::DEFAULT).value()
            )
        );
    }
}
