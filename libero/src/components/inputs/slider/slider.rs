use dioxus::prelude::*;

use super::value::{fraction, snap};
use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_color, base_props, contrast_color, dom_api, variables},
        layout::use_box,
    },
    hooks::{
        DragMove, DragOptions, DragStart, drag_handle_sx, use_css, use_drag, use_id,
        use_local_state, use_root_id, use_theme,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, SLIDER_RADIUS, SLIDER_THUMB, SLIDER_TRACK, Size, SizeCss, SliderDefaults},
    utils::warn,
};

const SLIDER_FILLED: CssVar = CssVar::new("--lsx-slider-filled");
const SLIDER_COLOR: CssVar = CssVar::new("--lsx-slider-color");
const SLIDER_CONTRAST: CssVar = CssVar::new("--lsx-slider-contrast");
// Inherited down to the bubble, which cannot see the root's `data-state` or
// the thumb's `:focus-visible` itself.
const SLIDER_LABEL_OPACITY: CssVar = CssVar::new("--lsx-slider-label-opacity");
const SLIDER_MARK_AT: CssVar = CssVar::new("--lsx-slider-mark-at");

/// Where the value went, and how far along the interaction is. `Start` and
/// `End` bracket one pointer drag; a key press emits a lone `Change`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderChangeEvent {
    Start(f64),
    Change(f64),
    End(f64),
}

impl SliderChangeEvent {
    pub fn value(&self) -> f64 {
        match self {
            Self::Start(value) | Self::Change(value) | Self::End(value) => *value,
        }
    }
}

/// A tick on the track, with an optional caption under it.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderMark {
    pub value: f64,
    pub label: Option<String>,
}

impl SliderMark {
    pub fn new(value: f64) -> Self {
        Self { value, label: None }
    }

    pub fn labeled(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
        }
    }
}

impl From<f64> for SliderMark {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

static SLIDER_ROOT_SX: StaticSx = StaticSx::new(|| {
    SliderDefaults::theme_vars()
        .and(drag_handle_sx())
        .display("flex")
        .align_items("center")
        .position("relative")
        // A flex/grid parent sizes to content, and the track is empty - so
        // without this the whole slider collapses to its two paddings.
        .width("100%")
        .min_height(SLIDER_THUMB.value())
        // Mantine's reserve: the track's own thickness, not the thumb's - the
        // thumb is centred on its value and overhangs the ends by design.
        .padding_left(SLIDER_TRACK.value())
        .padding_right(SLIDER_TRACK.value())
        .user_select("none")
        // Captions sit below the root, so they need reserved space or they
        // overlap whatever follows. Padding, not margin: a margin collapses
        // with the next sibling's and the reserve goes away.
        .when(
            "marks-labeled",
            sx().padding_bottom(format!(
                "calc({} + 1.5em)",
                SizeCss::SPACING.value(Size::Xs)
            )),
        )
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
});

static SLIDER_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("1 1 auto")
        .height(SLIDER_TRACK.value())
        .background("grey.2")
        .border_radius(SLIDER_RADIUS.value())
        .cursor("pointer")
});

static SLIDER_BAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left("0")
        .width(SLIDER_FILLED.value_or("0%"))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
});

static SLIDER_THUMB_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(SLIDER_FILLED.value_or("0%"))
        .transform("translate(-50%, -50%)")
        .width(SLIDER_THUMB.value())
        .height(SLIDER_THUMB.value())
        .border_radius("50%")
        .background("white")
        .border_style("solid")
        .border_width("2px")
        .border_color(SLIDER_COLOR.value())
        .cursor("grab")
        // The ring itself comes from `Box`; only the bubble is new here.
        .focus_visible(sx().var(SLIDER_LABEL_OPACITY, "1"))
});

static SLIDER_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .bottom(format!("calc(100% + {})", SizeCss::SPACING.value(Size::Xs)))
        .left("50%")
        .transform("translateX(-50%)")
        .padding_left(SizeCss::SPACING.value(Size::Xs))
        .padding_right(SizeCss::SPACING.value(Size::Xs))
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .background(SLIDER_COLOR.value())
        .color(SLIDER_CONTRAST.value_or("white"))
        .white_space("nowrap")
        .pointer_events("none")
        .opacity(SLIDER_LABEL_OPACITY.value_or("0"))
        .transition("opacity 100ms ease")
});

static SLIDER_MARK_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(SLIDER_MARK_AT.value_or("0%"))
        .transform("translate(-50%, -50%)")
        .width(format!("calc({} + 2px)", SLIDER_TRACK.value()))
        .height(format!("calc({} + 2px)", SLIDER_TRACK.value()))
        .border_radius("50%")
        .background("white")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.4")
});

static SLIDER_MARK_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        // Off the track's centre line, not its bottom: the track is thinner
        // than the thumb, so `100%` would put the caption under the thumb.
        .top(format!(
            "calc(50% + {} / 2 + {})",
            SLIDER_THUMB.value(),
            SizeCss::SPACING.value(Size::Xs)
        ))
        .left(SLIDER_MARK_AT.value_or("0%"))
        .transform("translateX(-50%)")
        .color("grey.7")
        .white_space("nowrap")
});

fn slider_variables(filled: f64, base: &ThemeAwareValue, dragging: bool) -> Variables {
    variables()
        .with(SLIDER_FILLED, Some(format!("{}%", filled * 100.0)))
        .with(SLIDER_COLOR, base.resolve(None))
        .with(
            SLIDER_CONTRAST,
            contrast_color(base).and_then(|color| color.resolve(None)),
        )
        .with(SLIDER_LABEL_OPACITY, dragging.then(|| "1".to_string()))
}

base_props! {
    pub struct SliderProps {
        /// Strictly controlled - pair it with `on_change`.
        value: f64,
        #[props(default, into)]
        min: Input<f64>,
        #[props(default, into)]
        max: Input<f64>,
        /// Grid the value snaps to, measured from `min`. Also sets how many
        /// decimals an emitted value keeps.
        #[props(default, into)]
        step: Input<f64>,
        #[props(default, into)]
        size: Input<Size>,
        /// Track corner radius, independent of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default)]
        disabled: Option<bool>,
        /// Formats the bubble shown while dragging or focused, and the
        /// thumb's `aria-valuetext`. Without it there is no bubble.
        #[props(default)]
        label: Option<Callback<f64, String>>,
        /// Ticks on the track; a labeled one gets a caption below it.
        #[props(default)]
        marks: Vec<SliderMark>,
        /// Names the thumb, which is the `role="slider"` element - an
        /// `aria_label` in `attributes` would land on the root instead.
        #[props(default)]
        aria_label: Option<String>,
        /// Emits a hidden input of that name, so the value posts with a form.
        #[props(default)]
        name: Option<String>,
        /// `Start`/`End` bracket a drag, `Change` carries every new value.
        #[props(default)]
        on_change: Option<EventHandler<SliderChangeEvent>>,
    }
}

/// A draggable value along a track. Controlled: it renders `value` and asks
/// for a new one through `on_change`.
#[component]
pub fn Slider(props: SliderProps) -> Element {
    let theme = use_theme();
    let root_id = use_root_id(&props.attributes);
    let track_id = use_id();
    let thumb_id = use_id();

    let min = props.min.copied_or(0.0);
    let max = props.max.copied_or(100.0);
    let step = props.step.copied_or(1.0).max(0.0);
    let size = props.size.copied_or(theme.slider.size);
    let radius = props.radius.copied_or(theme.slider.radius);
    let color = base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);

    let value = snap(props.value, min, max, step);
    let interactive = props.on_change.is_some() && !disabled;

    if props.on_change.is_none() && !disabled {
        warn("Slider: `value` without `on_change` can never change.");
    }

    // Measured off the track at pointerdown, when layout is known settled.
    let track_left = use_local_state(|| 0.0_f64);
    let track_width = use_local_state(|| 0.0_f64);
    // What `End` reports: the drag's own last value, which a controlled
    // parent may not have echoed back yet.
    let latest = use_local_state(|| value);

    let on_change = props.on_change;
    let emit = {
        let latest = latest.clone();
        use_callback(move |event: SliderChangeEvent| {
            latest.set(event.value());
            if let Some(on_change) = &on_change {
                on_change.call(event);
            }
        })
    };

    // A `Callback`, not a closure: `LocalState` is not `Copy`, and two drag
    // handlers need this.
    let value_at = {
        let (track_left, track_width) = (track_left.clone(), track_width.clone());
        use_callback(move |client_x: f64| {
            let width = track_width.get();
            if width <= 0.0 {
                return None;
            }
            Some(snap(
                min + (client_x - track_left.get()) / width * (max - min),
                min,
                max,
                step,
            ))
        })
    };

    let drag = use_drag(DragOptions {
        capture: root_id,
        on_start: Callback::new(move |event: DragStart| {
            if !interactive {
                return false;
            }
            let Ok(track) = dom_api().query_selector(&format!("#{}", track_id())) else {
                return false;
            };
            let (Ok(dimensions), Ok((left, _))) = (track.dimensions(), track.client_offset())
            else {
                return false;
            };
            if dimensions.width <= 0.0 {
                return false;
            }

            track_left.set(left);
            track_width.set(dimensions.width);

            match value_at.call(event.client.x) {
                Some(value) => {
                    // `use_drag` cancels the pointerdown, which cancels the
                    // browser's own focus - so the keyboard would be
                    // unreachable after a mouse drag.
                    let _ = dom_api()
                        .query_selector(&format!("#{}", thumb_id()))
                        .and_then(|thumb| thumb.focus());
                    emit.call(SliderChangeEvent::Start(value));
                    true
                }
                None => false,
            }
        }),
        on_move: Callback::new(move |event: DragMove| {
            if let Some(value) = value_at.call(event.client.x) {
                emit.call(SliderChangeEvent::Change(value));
            }
        }),
        on_end: Callback::new(move |_| {
            emit.call(SliderChangeEvent::End(latest.get()));
        }),
    });

    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        if !interactive {
            return;
        }
        let distance = if event.modifiers().shift() {
            theme.slider.big_step
        } else {
            theme.slider.step
        } * step;

        let go_to = |raw: f64| {
            event.prevent_default();
            emit.call(SliderChangeEvent::Change(snap(raw, min, max, step)));
        };

        match event.key() {
            Key::ArrowRight | Key::ArrowUp => go_to(value + distance),
            Key::ArrowLeft | Key::ArrowDown => go_to(value - distance),
            Key::PageUp => go_to(value + theme.slider.big_step * step),
            Key::PageDown => go_to(value - theme.slider.big_step * step),
            Key::Home => go_to(min),
            Key::End => go_to(max),
            _ => {}
        }
    });

    let marks_labeled = props.marks.iter().any(|mark| mark.label.is_some());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("dragging", (drag.dragging)())
        .with("disabled", disabled)
        .with("marks-labeled", marks_labeled)
        .into();

    let filled = fraction(value, min, max);
    let root_variables: Input<Variables> =
        slider_variables(filled, &color, (drag.dragging)()).into();

    let track_class = use_css(Some(&SLIDER_TRACK_SX), CssLayer::Framework);
    let bar_class = use_css(Some(&SLIDER_BAR_SX), CssLayer::Framework);
    let mark_class = use_css(Some(&SLIDER_MARK_SX), CssLayer::Framework);
    let mark_label_class = use_css(Some(&SLIDER_MARK_LABEL_SX), CssLayer::Framework);
    let label_class = use_css(Some(&SLIDER_LABEL_SX), CssLayer::Framework);
    let thumb = use_box().framework_sx(&SLIDER_THUMB_SX).prepare();

    let text = props.label.map(|label| label.call(value));

    let bubble = text
        .clone()
        .map(|text| rsx! { span { class: label_class, {text} } });

    let marks = props.marks.iter().map(|mark| {
        let at = variables()
            .with(
                SLIDER_MARK_AT,
                Some(format!("{}%", fraction(mark.value, min, max) * 100.0)),
            )
            .render();
        let caption = mark.label.clone().map(|label| {
            rsx! { span { class: mark_label_class.clone(), style: "{at}", {label} } }
        });
        rsx! {
            span { class: mark_class.clone(), style: "{at}" }
            {caption}
        }
    });

    let thumb = thumb
        .attr("role", "slider")
        .attr("tabindex", if interactive { "0" } else { "-1" })
        .attr("aria-orientation", "horizontal")
        .attr("aria-valuemin", min)
        .attr("aria-valuemax", max)
        .attr("aria-valuenow", value)
        .attr("aria-valuetext", text)
        .attr("aria-label", props.aria_label.clone())
        .attr("aria-disabled", !interactive)
        .attr("id", thumb_id())
        .event("onkeydown", move |event: Event<KeyboardData>| {
            onkeydown.call(event)
        })
        .render(HtmlTag::Div, Vec::new(), bubble);

    let name = props.name.clone();

    use_box()
        .framework_sx(&SLIDER_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&root_variables)
        .prepare()
        .attr("id", root_id())
        .event("onpointerdown", drag.onpointerdown)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { class: track_class, id: track_id(),
                    div { class: bar_class }
                    {marks}
                    {thumb}
                }
                if let Some(name) = name {
                    input { r#type: "hidden", name, value: "{value}", disabled }
                }
            },
        )
}
