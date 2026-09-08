use dioxus::prelude::*;

use super::slider_value::{SliderChangeEvent, SliderMark};
use super::value::{SliderCoreValue, fraction, sane_bounds};
use crate::{
    CssLayer,
    components::{
        ClassList, HtmlTag, Input, States, Variables,
        common::{base_color, shadow_sx, variables},
        layout::use_box,
        overlay::Tooltip,
    },
    hooks::{
        DragMove, DragOptions, DragStart, drag_handle_sx, use_css, use_drag, use_element, use_id,
        use_local_state, use_theme,
    },
    platform::ElementApi,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::NamedColorCss,
    theme::{
        Color, ColorShade, ColorValue, CssVar, SLIDER_THUMB, SLIDER_TRACK, Size, SliderDefaults,
    },
    utils::warn,
};

/// A unitless 0-1 fraction, not a percentage: everything positioned along the
/// track multiplies it by the travel, which is a `calc` of two lengths.
const SLIDER_FILLED: CssVar = CssVar::new("--lsx-slider-filled");
/// Where the bar starts, and how wide it is: `0` and the fill for one thumb,
/// the two thumbs' own fractions for a range.
const SLIDER_FILLED_FROM: CssVar = CssVar::new("--lsx-slider-filled-from");
const SLIDER_FILLED_SPAN: CssVar = CssVar::new("--lsx-slider-filled-span");
/// One thumb's own position, set on its anchor - the thumbs of a range sit at
/// two different fractions, so the fill's cannot serve them both.
const SLIDER_THUMB_AT: CssVar = CssVar::new("--lsx-slider-thumb-at");
const SLIDER_COLOR: CssVar = CssVar::new("--lsx-slider-color");
const SLIDER_MARK_AT: CssVar = CssVar::new("--lsx-slider-mark-at");
/// A plain slider's thumb face - the color the track is pointing at.
const SLIDER_THUMB_FILL: CssVar = CssVar::new("--lsx-slider-thumb-fill");
/// The side of the thumb's invisible hit area. 24px unless a skin whose
/// sliders sit closer than that sets it; see `SLIDER_THUMB_SX`.
pub(in crate::components::form) const SLIDER_HIT: CssVar = CssVar::new("--lsx-slider-hit");

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
        // Nothing to grab: the track's pointer and the thumb's grab hand would
        // promise a drag that `onstart` refuses.
        .when("readonly", sx().selector("& *", sx().cursor("default")))
        // The track is the scale, so the thumb must read against any color on
        // it: white ring, dark halo, the picked color as its face.
        .when(
            "plain",
            sx().selector(
                "& [role='slider']",
                sx().border_color("surface").and(shadow_sx(
                    "0 0 0 1px rgba(0, 0, 0, 0.2), inset 0 0 0 1px rgba(0, 0, 0, 0.2)".to_string(),
                )),
            ),
        )
});

static SLIDER_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("1 1 auto")
        .height(SLIDER_TRACK.value())
        .background("grey.2")
        // A pill, always: the radius scale starts at 2px and a track is 2-10px
        // tall, so every step above the smallest clamped to the same half-height
        // curve. The shared `radius` prop is not wired here for that reason.
        .border_radius("999px")
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

/// A range fills *between* its thumbs, so the bar starts at the lower one's
/// centre rather than at the track's edge, and spans the difference.
static SLIDER_RANGE_BAR_SX: StaticSx = StaticSx::new(|| {
    let thumb = SLIDER_THUMB.value();
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left(along_track(SLIDER_FILLED_FROM))
        .width(format!(
            "calc({} * (100% - {thumb}))",
            SLIDER_FILLED_SPAN.value_or("0")
        ))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
});

/// Carries the thumb's position, because the `Tooltip` between them styles
/// only its own bubble - its wrapper cannot be positioned from outside.
static SLIDER_THUMB_ANCHOR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(along_track(SLIDER_THUMB_AT))
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
        // `value_or`'s fallback is interpolated as raw CSS - `sx` never sees
        // it - so it has to be a var reference, not a colour name. `"surface"`
        // here would be an unknown CSS keyword and the thumb would compute to
        // transparent (todo 385).
        .background(SLIDER_THUMB_FILL.value_or(NamedColorCss::SURFACE.value()))
        .border_style("solid")
        .border_width("2px")
        .border_color(SLIDER_COLOR.value())
        .cursor("grab")
        // WCAG 2.5.8 wants a 24x24 target, and the default thumb is 16px. An
        // invisible square centred on the thumb takes the pointer instead, so
        // the thumb keeps its look. Which thumb a press grabs is picked by
        // value, never by hit-test, so two overlapping squares cannot swap
        // a range's thumbs.
        .position("relative")
        .selector(
            "&::before",
            sx().content("\"\"")
                .position("absolute")
                .top("50%")
                .left("50%")
                .width(SLIDER_HIT.value_or("24px"))
                .height(SLIDER_HIT.value_or("24px"))
                .transform("translate(-50%, -50%)"),
        )
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
        .background(ColorValue::Shade(Color::Grey, ColorShade::S4).value())
        // White on the filled bar, the way the thumb is - the grey dot would
        // disappear into it.
        .when("filled", sx().background("surface"))
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

fn slider_variables(
    bar: (f64, f64),
    base: &ThemeAwareValue,
    thumb_fill: Option<String>,
) -> Variables {
    let (from, to) = bar;
    variables()
        .with(SLIDER_FILLED, Some(to.to_string()))
        .with(SLIDER_FILLED_FROM, Some(from.to_string()))
        .with(SLIDER_FILLED_SPAN, Some((to - from).to_string()))
        .with(SLIDER_COLOR, base.resolve(None))
        .with(SLIDER_THUMB_FILL, thumb_fill)
}

/// The `f64` engine every `Slider<V>` renders. Non-generic on purpose: this
/// is the whole component, and it is compiled once no matter how many value
/// types a caller slides over.
///
/// `HueSlider` and `AlphaSlider` render it too, as a `plain` slider over a
/// `track` gradient.
#[derive(Props, Clone, PartialEq)]
pub(in crate::components::form) struct SliderCoreProps {
    /// Already resolved by the skin - the scale, not the caller's type. One
    /// thumb or two; a range's own clamping lives in `SliderCoreValue`.
    value: SliderCoreValue,
    min: f64,
    max: f64,
    step: f64,
    /// The smallest gap a range's two thumbs keep. Ignored by a single thumb.
    #[props(default)]
    min_range: f64,
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
    color: Input<ThemeAwareValue>,
    disabled: Option<bool>,
    /// Focusable and posted, but no key or drag moves a thumb.
    #[props(default)]
    readonly: bool,
    /// `None` leaves the bubble showing the bare value and sets no
    /// `aria-valuetext`.
    label: Option<Callback<f64, String>>,
    marks: Vec<SliderMark>,
    aria_label: Option<String>,
    /// Names the second thumb of a range; `aria_label` names the first.
    aria_label_to: Option<String>,
    /// The field's label id, when a `<label for>` cannot name the thumb.
    labelledby: Option<String>,
    /// The field's filled caption slots, joined.
    describedby: Option<String>,
    /// The field's status is an error.
    invalid: bool,
    required: bool,
    name: Option<String>,
    oninput: Option<EventHandler<SliderChangeEvent<SliderCoreValue>>>,
    /// A CSS `background` for the track in place of its grey - a hue or an
    /// alpha gradient.
    #[props(default)]
    track: Option<String>,
    /// No filled bar and no value bubble: the track itself shows the value.
    #[props(default)]
    plain: bool,
    /// The thumb's face, for a `plain` slider. White otherwise.
    #[props(default)]
    thumb_fill: Option<String>,
    /// `false` keeps the thumbs out of the tab order and a drag from focusing
    /// them - for a slider inside a dropdown whose trigger must keep focus.
    #[props(default = true)]
    focusable: bool,
}

#[component]
pub(in crate::components::form) fn SliderCore(props: SliderCoreProps) -> Element {
    let theme = use_theme();
    let root_element = use_element();
    let track_element = use_element();
    // Two handles, always: a hook cannot be conditional, and a single-thumb
    // slider simply never mounts the second.
    let thumb_elements = [use_element(), use_element()];
    // Names each thumb's value bubble, so the thumb can point at it.
    let bubble_id = use_id();

    // `snap` clamps on every render, and `f64::clamp` panics on an inverted
    // or non-finite range - which `max: items.len() as f64 - 1.0` is for an
    // empty list. `step.max(0.0)` already maps a NaN step to 0 (continuous).
    let (min, max) = sane_bounds(props.min, props.max);
    let step = props.step.max(0.0);
    let min_range = props.min_range.max(0.0);
    let size = props.size.copied_or(theme.slider.size);
    let color = base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);

    let value = props.value.snapped(min, max, step);
    let interactive = props.oninput.is_some() && !disabled;
    // `interactive` keeps the tab stop, `editable` moves the value: a
    // read-only thumb is still reached, read and posted.
    let editable = interactive && !props.readonly;
    let focusable = props.focusable;

    if props.oninput.is_none() && !disabled {
        warn("Slider: `value` without `oninput` can never change.");
    }

    // Measured off the track at pointerdown, when layout is known settled.
    let track_left = use_local_state(|| 0.0_f64);
    let track_width = use_local_state(|| 0.0_f64);
    let thumb_width = use_local_state(|| 0.0_f64);
    // What `End` reports, and what a range's moves are measured against: the
    // drag's own last value, which a controlled parent may not have echoed
    // back yet.
    let latest = use_local_state(|| value);
    // Which thumb the pointer grabbed. Always 0 for a single thumb.
    let active = use_local_state(|| 0_usize);

    let oninput = props.oninput;
    let emit = {
        let latest = latest.clone();
        use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
            latest.set(event.value());
            if let Some(oninput) = &oninput {
                oninput.call(event);
            }
        })
    };

    // A `Callback`, not a closure: `LocalState` is not `Copy`, and two drag
    // handlers need this. Unsnapped - the grid is applied where the thumb is
    // also clamped against its neighbour.
    let position_at = {
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
            Some(min + (client_x - track_left.get() - thumb / 2.0) / travel * (max - min))
        })
    };

    // The drag's two steps, both through `use_callback` so they see this
    // render's `value` - the handlers `use_drag` holds do not.
    let grab = {
        let active = active.clone();
        use_callback(move |raw: f64| {
            let index = value.nearest(raw);
            active.set(index);
            emit.call(SliderChangeEvent::Start(
                value.moved(index, raw, min, max, step, min_range),
            ));
            index
        })
    };
    let slide = {
        let (active, latest) = (active.clone(), latest.clone());
        use_callback(move |raw: f64| {
            emit.call(SliderChangeEvent::Change(latest.get().moved(
                active.get(),
                raw,
                min,
                max,
                step,
                min_range,
            )));
        })
    };

    let drag = use_drag(DragOptions {
        capture: root_element,
        onstart: Callback::new(move |event: DragStart| {
            if !editable {
                event.cancel.call(());
                return;
            }

            let (track_left, track_width, thumb_width) =
                (track_left.clone(), track_width.clone(), thumb_width.clone());
            // Started here, awaited in the task: a read resolves where it is
            // called, and under Blitz that has to be inside the handler - the
            // document is locked for as long as tasks are draining.
            let track_size = track_element.dimensions();
            let track_offset = track_element.client_offset();
            let thumb_size = thumb_elements[0].dimensions();
            // Off the web a measurement is a round-trip, so the drag starts
            // before the geometry is known - moves landing first are dropped
            // by `position_at`'s own zero-travel guard.
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

                match position_at.call(event.client.x) {
                    Some(raw) => {
                        // `use_drag` cancels the pointerdown, which cancels
                        // the browser's own focus - so the keyboard would be
                        // unreachable after a mouse drag. Which thumb to
                        // focus is only known once the pointer is mapped.
                        let index = grab.call(raw);
                        if focusable {
                            let _ = thumb_elements[index].focus();
                        }
                    }
                    None => event.cancel.call(()),
                }
            });
        }),
        onmove: Callback::new(move |event: DragMove| {
            if let Some(raw) = position_at.call(event.client.x) {
                slide.call(raw);
            }
        }),
        onend: Callback::new(move |_| {
            emit.call(SliderChangeEvent::End(latest.get()));
        }),
    });

    // One handler for both thumbs: the focused thumb is the one the keys
    // move, so the index comes from whichever element fired.
    let onkeydown = use_callback(move |(index, event): (usize, Event<KeyboardData>)| {
        if !editable {
            return;
        }
        let distance = if event.modifiers().shift() {
            theme.slider.big_step
        } else {
            theme.slider.step
        } * step;

        // `Change` then `End`: a key press settles on its value the moment it
        // lands, so a caller that commits on `End` - which is what the docs
        // ask for - has to hear about a keyboard edit too.
        let go_to = |raw: f64| {
            event.prevent_default();
            let moved = value.moved(index, raw, min, max, step, min_range);
            emit.call(SliderChangeEvent::Change(moved));
            emit.call(SliderChangeEvent::End(moved));
        };

        let thumb = value.thumb(index);
        match event.key() {
            Key::ArrowRight | Key::ArrowUp => go_to(thumb + distance),
            Key::ArrowLeft | Key::ArrowDown => go_to(thumb - distance),
            Key::PageUp => go_to(thumb + theme.slider.big_step * step),
            Key::PageDown => go_to(thumb - theme.slider.big_step * step),
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
        .with("dragging", (drag.dragging)())
        .with("disabled", disabled)
        .with("readonly", props.readonly)
        .with("marks-labeled", marks_labeled)
        .with("plain", props.plain)
        .into();

    let bar = value.bar(min, max);
    let root_variables: Input<Variables> =
        slider_variables(bar, &color, props.thumb_fill.clone()).into();

    let track_class = use_css(Some(&SLIDER_TRACK_SX), CssLayer::Framework);
    let bar_class = use_css(
        Some(match props.value {
            SliderCoreValue::Single(_) => &SLIDER_BAR_SX,
            SliderCoreValue::Range { .. } => &SLIDER_RANGE_BAR_SX,
        }),
        CssLayer::Framework,
    );
    let mark_class = use_css(Some(&SLIDER_MARK_SX), CssLayer::Framework);
    let mark_label_class = use_css(Some(&SLIDER_MARK_LABEL_SX), CssLayer::Framework);
    let anchor_class = use_css(Some(&SLIDER_THUMB_ANCHOR_SX), CssLayer::Framework);
    // One prepared style, cloned per thumb: the two are identical, and a
    // second `use_box` would be a second hook for nothing.
    let thumb_style = use_box().framework_sx(&SLIDER_THUMB_SX).prepare();

    let marks = props.marks.iter().map(|mark| {
        let mark_at = fraction(mark.value, min, max);
        let at = variables()
            .with(SLIDER_MARK_AT, Some(mark_at.to_string()))
            .render();
        // A state, not a var set only when filled: a raw `style` that drops a
        // declaration keeps its last value ([[codebase/css-vars]]).
        let filled = (bar.0 <= mark_at && mark_at <= bar.1).then_some("filled");
        let caption = mark.label.clone().map(|label| {
            rsx! { span { class: mark_label_class.clone(), style: "{at}", {label} } }
        });
        rsx! {
            span { class: mark_class.clone(), "data-state": filled, style: "{at}" }
            {caption}
        }
    });

    let aria_labels = [props.aria_label.clone(), props.aria_label_to.clone()];
    let range = matches!(value, SliderCoreValue::Range { .. });
    let thumbs = value.thumbs().enumerate().map(|(index, thumb_value)| {
        let (thumb_min, thumb_max) = value.bounds(index, min, max, min_range);
        // Only a custom label is worth an `aria-valuetext` - the bare value
        // is already in `aria-valuenow`.
        let text = props.label.map(|label| label.call(thumb_value));
        let bubble_text = text.clone().unwrap_or_else(|| thumb_value.to_string());
        // `aria-labelledby` beats `aria-label`, so a range thumb that has both
        // lists itself after the field's label: its own `aria-label` then
        // follows the caption ("Price Minimum") instead of being dropped. A
        // single thumb's `aria_label` is the stand-in for a missing `label`,
        // so there the label keeps winning alone.
        let aria_label = aria_labels[index].clone();
        let (own_id, labelledby) = match (&props.labelledby, &aria_label) {
            (Some(label), Some(_)) if range => {
                let own_id = format!("{label}-thumb-{index}");
                let labelledby = format!("{label} {own_id}");
                (Some(own_id), Some(labelledby))
            }
            _ => (None, props.labelledby.clone()),
        };

        // A `plain` slider has no bubble to point at. The bubble goes after
        // the field's captions: a hint or an error is the news, the value is
        // already in `aria-valuenow`.
        let bubble_id = (!props.plain).then(|| format!("{}-value-{index}", bubble_id.read()));
        let describedby = [props.describedby.clone(), bubble_id.clone()]
            .into_iter()
            .flatten()
            .reduce(|captions, bubble| format!("{captions} {bubble}"));

        let thumb = thumb_style
            .clone()
            .attr("role", "slider")
            .attr(
                "tabindex",
                if interactive && focusable { "0" } else { "-1" },
            )
            .attr("aria-orientation", "horizontal")
            .attr("aria-valuemin", thumb_min)
            .attr("aria-valuemax", thumb_max)
            .attr("aria-valuenow", thumb_value)
            .attr("aria-valuetext", text)
            .attr("id", own_id)
            .attr("aria-label", aria_label)
            .attr("aria-labelledby", labelledby)
            .attr("aria-describedby", describedby)
            .attr("aria-invalid", props.invalid.then_some("true"))
            .attr("aria-required", props.required.then_some("true"))
            .attr("aria-disabled", !interactive)
            .attr("aria-readonly", props.readonly.then_some("true"))
            .element(&thumb_elements[index])
            .event("onkeydown", move |event: Event<KeyboardData>| {
                onkeydown.call((index, event))
            })
            .render(HtmlTag::Span, Vec::new(), rsx! {});

        let at = variables()
            .with(
                SLIDER_THUMB_AT,
                Some(fraction(thumb_value, min, max).to_string()),
            )
            .render();

        if props.plain {
            return rsx! {
                span { class: anchor_class.clone(), style: "{at}", {thumb} }
            };
        }

        // The drag keeps it open once the pointer has left the thumb; hover
        // and keyboard focus are the tooltip's own doing.
        rsx! {
            span { class: anchor_class.clone(), style: "{at}",
                Tooltip {
                    label: rsx! { {bubble_text} },
                    size,
                    open: ((drag.dragging)() && active.get() == index).then_some(true),
                    label_id: bubble_id,
                    {thumb}
                }
            }
        }
    });

    let hidden = props.name.clone().map(|name| {
        let inputs = value.thumbs().map(move |thumb_value| {
            // `Some(true)` or nothing - see `SegmentedControl`: a `false`
            // bool reaches a native renderer as the string "false", which
            // reads as disabled.
            rsx! {
                input {
                    r#type: "hidden",
                    name: name.clone(),
                    value: "{thumb_value}",
                    disabled: disabled.then_some(true),
                }
            }
        });
        // A range posts its two values under one name, in track order:
        // `FormData::get_all` reads them back as a pair.
        rsx! { {inputs} }
    });

    let track_style = props
        .track
        .as_ref()
        .map(|background| format!("background: {background}"));

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
                    style: track_style,
                    onmounted: track_element.mount(),
                    if !props.plain {
                        div { class: bar_class }
                    }
                    {marks}
                    {thumbs}
                }
                {hidden}
            },
        )
}
