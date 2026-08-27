use dioxus::prelude::*;

use super::slider_value::{SliderChangeEvent, SliderMark};
use super::value::{fraction, snap};
use crate::{
    CssLayer,
    components::{
        ClassList, HtmlTag, Input, States, Variables,
        common::{base_color, variables},
        layout::use_box,
        overlay::Tooltip,
    },
    hooks::{
        DragMove, DragOptions, DragStart, drag_handle_sx, use_css, use_drag, use_element,
        use_local_state, use_theme,
    },
    platform::ElementApi,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        Color, ColorShade, ColorValue, CssVar, SLIDER_RADIUS, SLIDER_THUMB, SLIDER_TRACK, Size,
        SizeCss, SliderDefaults,
    },
    utils::warn,
};

/// A unitless 0-1 fraction, not a percentage: everything positioned along the
/// track multiplies it by the travel, which is a `calc` of two lengths.
const SLIDER_FILLED: CssVar = CssVar::new("--lsx-slider-filled");
const SLIDER_COLOR: CssVar = CssVar::new("--lsx-slider-color");
const SLIDER_MARK_AT: CssVar = CssVar::new("--lsx-slider-mark-at");
/// Set only on marks the bar has already reached, so one class covers both.
const SLIDER_MARK_FILL: CssVar = CssVar::new("--lsx-slider-mark-fill");

/// Where a 0-1 `fraction` sits along the track. The track is the full width,
/// so the thumb travels inset by half its own width and never overhangs -
/// which is also what keeps stacked sliders of different sizes aligned.
fn along_track(fraction: CssVar) -> String {
    let thumb = SLIDER_THUMB.value();
    format!(
        "calc({thumb} / 2 + {} * (100% - {thumb}))",
        fraction.value_or("0")
    )
}

static SLIDER_ROOT_SX: StaticSx = StaticSx::new(|| {
    SliderDefaults::theme_vars()
        .and(drag_handle_sx())
        .display("flex")
        .align_items("center")
        .position("relative")
        // A flex/grid parent sizes to content, and the track is empty - so
        // without this the whole slider collapses to nothing.
        .width("100%")
        .min_height(SLIDER_THUMB.value())
        .user_select("none")
        // Captions sit below the root, so they need reserved space or they
        // overlap whatever follows. Padding, not margin: a margin collapses
        // with the next sibling's and the reserve goes away.
        .when("marks-labeled", sx().padding_bottom("1.5em"))
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
        .width(along_track(SLIDER_FILLED))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
});

/// Carries the thumb's position, because the `Tooltip` between them styles
/// only its own bubble - its wrapper cannot be positioned from outside.
static SLIDER_THUMB_ANCHOR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(along_track(SLIDER_FILLED))
        .transform("translate(-50%, -50%)")
        // Not inline: the tooltip's inline-block wrapper would sit on a
        // baseline and pull the thumb off the track's centre.
        .display("flex")
});

static SLIDER_THUMB_SX: StaticSx = StaticSx::new(|| {
    // A `span`, because the tooltip's wrapper is one - so it needs a box.
    sx().display("block")
        .width(SLIDER_THUMB.value())
        .height(SLIDER_THUMB.value())
        .border_radius("50%")
        .background("white")
        .border_style("solid")
        .border_width("2px")
        .border_color(SLIDER_COLOR.value())
        .cursor("grab")
});

static SLIDER_MARK_SX: StaticSx = StaticSx::new(|| {
    // Sized against the thumb, not the track: a 2px track leaves a dot too
    // small to see, and it has to read against the thumb beside it.
    let dot = format!("calc({} / 3)", SLIDER_THUMB.value());
    sx().position("absolute")
        .top("50%")
        .left(along_track(SLIDER_MARK_AT))
        .transform("translate(-50%, -50%)")
        .width(dot.clone())
        .height(dot)
        .border_radius("50%")
        .background(
            SLIDER_MARK_FILL.value_or(ColorValue::Shade(Color::Grey, ColorShade::S4).value()),
        )
});

static SLIDER_MARK_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        // Off the track's centre line, not its bottom: the track is thinner
        // than the thumb, so `100%` would put the caption under the thumb.
        //
        // Measured from the top in thumbs, never as a percentage: `50%`
        // resolves against the *padding* box, and the `marks-labeled` reserve
        // below is padding - so a percentage drifts the caption down by half
        // the reserve and eats the gap to whatever follows. The content box is
        // one thumb tall, so its centre line is at `thumb / 2`, and one whole
        // thumb sits the caption directly under the thumb's lower edge.
        //
        // The reserve below must stay this offset plus the caption's own line
        // box, or the captions overflow the root and land on whatever follows.
        .top(SLIDER_THUMB.value())
        .left(along_track(SLIDER_MARK_AT))
        // Centred in the middle, edge-aligned at the ends: a `-50%` caption
        // on the first or last mark hangs outside the slider's own box.
        .transform(format!(
            "translateX(calc({} * -100%))",
            SLIDER_MARK_AT.value_or("0")
        ))
        .color("grey.7")
        .white_space("nowrap")
});

fn slider_variables(filled: f64, base: &ThemeAwareValue) -> Variables {
    variables()
        .with(SLIDER_FILLED, Some(filled.to_string()))
        .with(SLIDER_COLOR, base.resolve(None))
}

/// The `f64` engine every `Slider<V>` renders. Non-generic on purpose: this
/// is the whole component, and it is compiled once no matter how many value
/// types a caller slides over.
#[derive(Props, Clone, PartialEq)]
pub(super) struct SliderCoreProps {
    /// Already resolved by the skin - the scale, not the caller's type.
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    attributes: Vec<Attribute>,
    /// The field owns the wrapper's styling, so the core's own is empty
    /// unless something inside the library styles the track directly.
    #[props(default)]
    class: Input<ClassList>,
    #[props(default)]
    sx: Input<Sx>,
    #[props(default)]
    states: Input<States>,
    size: Input<Size>,
    radius: Input<Size>,
    color: Input<ThemeAwareValue>,
    disabled: Option<bool>,
    /// `None` leaves the bubble showing the bare value and sets no
    /// `aria-valuetext`.
    label: Option<Callback<f64, String>>,
    marks: Vec<SliderMark>,
    aria_label: Option<String>,
    /// The field's label id, when a `<label for>` cannot name the thumb.
    labelledby: Option<String>,
    /// The field's filled caption slots, joined.
    describedby: Option<String>,
    /// The field's status is an error.
    invalid: bool,
    required: bool,
    name: Option<String>,
    oninput: Option<EventHandler<SliderChangeEvent>>,
}

#[component]
pub(super) fn SliderCore(props: SliderCoreProps) -> Element {
    let theme = use_theme();
    let root_element = use_element();
    let track_element = use_element();
    let thumb_element = use_element();

    let (min, max, step) = (props.min, props.max, props.step.max(0.0));
    let size = props.size.copied_or(theme.slider.size);
    let radius = props.radius.copied_or(theme.slider.radius);
    let color = base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);

    let value = snap(props.value, min, max, step);
    let interactive = props.oninput.is_some() && !disabled;

    if props.oninput.is_none() && !disabled {
        warn("Slider: `value` without `oninput` can never change.");
    }

    // Measured off the track at pointerdown, when layout is known settled.
    let track_left = use_local_state(|| 0.0_f64);
    let track_width = use_local_state(|| 0.0_f64);
    let thumb_width = use_local_state(|| 0.0_f64);
    // What `End` reports: the drag's own last value, which a controlled
    // parent may not have echoed back yet.
    let latest = use_local_state(|| value);

    let oninput = props.oninput;
    let emit = {
        let latest = latest.clone();
        use_callback(move |event: SliderChangeEvent| {
            latest.set(event.value());
            if let Some(oninput) = &oninput {
                oninput.call(event);
            }
        })
    };

    // A `Callback`, not a closure: `LocalState` is not `Copy`, and two drag
    // handlers need this.
    let value_at = {
        let (track_left, track_width, thumb_width) =
            (track_left.clone(), track_width.clone(), thumb_width.clone());
        use_callback(move |client_x: f64| {
            let thumb = thumb_width.get();
            // The thumb's centre only travels between the two half-thumb
            // insets, so the pointer has to be mapped over that span too.
            let travel = track_width.get() - thumb;
            if travel <= 0.0 {
                return None;
            }
            Some(snap(
                min + (client_x - track_left.get() - thumb / 2.0) / travel * (max - min),
                min,
                max,
                step,
            ))
        })
    };

    let drag = use_drag(DragOptions {
        capture: root_element,
        on_start: Callback::new(move |event: DragStart| {
            if !interactive {
                event.cancel.call(());
                return;
            }
            // `use_drag` cancels the pointerdown, which cancels the browser's
            // own focus - so the keyboard would be unreachable after a mouse
            // drag. A command, so it needs no round-trip.
            let _ = thumb_element.focus();

            let (track_left, track_width, thumb_width) =
                (track_left.clone(), track_width.clone(), thumb_width.clone());
            // Started here, awaited in the task: a read resolves where it is
            // called, and under Blitz that has to be inside the handler - the
            // document is locked for as long as tasks are draining.
            let track_size = track_element.dimensions();
            let track_offset = track_element.client_offset();
            let thumb_size = thumb_element.dimensions();
            // Off the web a measurement is a round-trip, so the drag starts
            // before the geometry is known - moves landing first are dropped
            // by `value_at`'s own zero-travel guard.
            spawn(async move {
                let (Ok(dimensions), Ok((left, _))) = (track_size.await, track_offset.await) else {
                    event.cancel.call(());
                    return;
                };
                if dimensions.width <= 0.0 {
                    event.cancel.call(());
                    return;
                }

                track_left.set(left);
                track_width.set(dimensions.width);
                thumb_width.set(thumb_size.await.map_or(0.0, |size| size.width));

                match value_at.call(event.client.x) {
                    Some(value) => emit.call(SliderChangeEvent::Start(value)),
                    None => event.cancel.call(()),
                }
            });
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
    let root_variables: Input<Variables> = slider_variables(filled, &color).into();

    let track_class = use_css(Some(&SLIDER_TRACK_SX), CssLayer::Framework);
    let bar_class = use_css(Some(&SLIDER_BAR_SX), CssLayer::Framework);
    let mark_class = use_css(Some(&SLIDER_MARK_SX), CssLayer::Framework);
    let mark_label_class = use_css(Some(&SLIDER_MARK_LABEL_SX), CssLayer::Framework);
    let anchor_class = use_css(Some(&SLIDER_THUMB_ANCHOR_SX), CssLayer::Framework);
    let thumb = use_box().framework_sx(&SLIDER_THUMB_SX).prepare();

    // Only a custom label is worth an `aria-valuetext` - the bare value is
    // already in `aria-valuenow`.
    let text = props.label.map(|label| label.call(value));
    let bubble_text = text.clone().unwrap_or_else(|| value.to_string());

    let marks = props.marks.iter().map(|mark| {
        let mark_at = fraction(mark.value, min, max);
        let at = variables()
            .with(SLIDER_MARK_AT, Some(mark_at.to_string()))
            // White on the filled bar, the way the thumb is - the grey dot
            // would disappear into it.
            .with(
                SLIDER_MARK_FILL,
                (mark_at <= filled).then(|| "white".to_string()),
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
        .attr("aria-labelledby", props.labelledby.clone())
        .attr("aria-describedby", props.describedby.clone())
        .attr("aria-invalid", props.invalid.then_some("true"))
        .attr("aria-required", props.required.then_some("true"))
        .attr("aria-disabled", !interactive)
        .element(&thumb_element)
        .event("onkeydown", move |event: Event<KeyboardData>| {
            onkeydown.call(event)
        })
        .render(HtmlTag::Span, Vec::new(), rsx! {});

    // The drag keeps it open once the pointer has left the thumb; hover and
    // keyboard focus are the tooltip's own doing.
    let thumb = rsx! {
        span { class: anchor_class,
            Tooltip {
                label: rsx! { {bubble_text} },
                size,
                opened: (drag.dragging)().then_some(true),
                {thumb}
            }
        }
    };

    let name = props.name.clone();

    use_box()
        .framework_sx(&SLIDER_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&root_variables)
        .prepare()
        .element(&root_element)
        .event("onpointerdown", drag.onpointerdown)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div {
                    class: track_class,
                    onmounted: track_element.mount(),
                    div { class: bar_class }
                    {marks}
                    {thumb}
                }
                if let Some(name) = name {
                    // `Some(true)` or nothing - see `SegmentedControl`: a
                    // `false` bool reaches a native renderer as the string
                    // "false", which reads as disabled.
                    input {
                        r#type: "hidden",
                        name,
                        value: "{value}",
                        disabled: disabled.then_some(true),
                    }
                }
            },
        )
}
