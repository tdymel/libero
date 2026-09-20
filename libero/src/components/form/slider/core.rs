use dioxus::prelude::*;

use super::slider_value::{SliderChangeEvent, SliderMark};
use super::value::{SliderCoreValue, fraction, sane_bounds};
use crate::{
    CssLayer,
    components::{
        common::{
            ClassList, HtmlTag, Input, States, Variables, base_color, has_shortcut_modifier,
            shadow_sx, variables,
        },
        layout::use_box,
        overlay::{PressFocus, Tooltip},
    },
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, drag_handle_sx, use_css, use_drag,
        use_element, use_formats, use_id, use_local_state, use_theme,
    },
    platform::{ElementApi, logical_key, next_task},
    sx::{FORCED_COLORS, StaticSx, Sx, ThemeAwareValue, sx},
    theme::NamedColorCss,
    theme::{
        Color, ColorShade, ColorValue, CssVar, OWN_SHADOW, SLIDER_THUMB, SLIDER_TRACK, Size,
        SliderDefaults,
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

/// Where a 0-1 `fraction` sits along the track, inset by half a thumb so it
/// never overhangs.
fn along_track(fraction: CssVar) -> String {
    let thumb = SLIDER_THUMB.value();
    format!(
        "calc({thumb} / 2 + {} * (100% - {thumb}))",
        fraction.value_or("0")
    )
}

const PLAIN_THUMB_SHADOW: &str = "0 0 0 1px rgba(0, 0, 0, 0.2), inset 0 0 0 1px rgba(0, 0, 0, 0.2)";

static SLIDER_ROOT_SX: StaticSx = StaticSx::new(|| {
    SliderDefaults::theme_vars()
        .and(drag_handle_sx())
        .display("flex")
        .align_items("center")
        .position("relative")
        // Or a flex/grid parent collapses the empty track to nothing.
        .width("100%")
        .min_height(SLIDER_THUMB.value())
        .user_select("none")
        // Room for the captions. Padding, not margin, which would collapse.
        .when("marks-labeled", sx().padding_bottom("1.5em"))
        // No grab cursor for a drag `onstart` refuses.
        .when("readonly", sx().selector("& *", sx().cursor("default")))
        // No `pointer-events: none`, so `not-allowed` shows (todo 596): `onstart`
        // refuses the drag, and the thumb's bubble is held shut. After `readonly`.
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .selector("& *", sx().cursor("not-allowed")),
        )
        // White ring and dark halo read on any track color. The shadow only
        // unfocused, or it outranks the thumb's `:focus-visible` ring.
        .when(
            "plain",
            sx().selector(
                "& [role='slider']",
                sx().border_color("surface")
                    .var(OWN_SHADOW, PLAIN_THUMB_SHADOW),
            )
            .selector(
                "& [role='slider']:not(:focus-visible)",
                shadow_sx(PLAIN_THUMB_SHADOW.to_string()),
            ),
        )
});

static SLIDER_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("1 1 auto")
        .height(SLIDER_TRACK.value())
        // 3:1 in both schemes (WCAG 1.4.11, todo 512). Via its var: a named
        // shade would publish a `--lsx-focus-contrast` the thumb's ring takes.
        .background(ColorValue::Shade(Color::Muted, ColorShade::S6).value())
        // Always a pill: at 2-10px tall every radius step clamps the same.
        .border_radius("999px")
        .cursor("pointer")
        // Forced colours drop the background but paint a transparent outline
        // (todo 506).
        .outline("1px solid transparent")
});

static SLIDER_BAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left("0")
        .width(along_track(SLIDER_FILLED))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
        .media(FORCED_COLORS, sx().background("Highlight"))
        // Under RTL the track runs right to left, as a native range does.
        .rtl(sx().left("auto").right("0"))
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
        .media(FORCED_COLORS, sx().background("Highlight"))
        .rtl(sx().left("auto").right(along_track(SLIDER_FILLED_FROM)))
});

/// Carries the thumb's position, because the `Tooltip` between them styles
/// only its own bubble - its wrapper cannot be positioned from outside.
static SLIDER_THUMB_ANCHOR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(along_track(SLIDER_THUMB_AT))
        // Centred by margins, not a `transform`: Blitz's client rect ignores
        // that, so the thumb measured half a thumb off where it is drawn.
        .margin(format!("calc({} / -2)", SLIDER_THUMB.value()))
        // Not inline: the tooltip's inline-block wrapper would sit on a
        // baseline and pull the thumb off the track's centre.
        .display("flex")
        .rtl(sx().left("auto").right(along_track(SLIDER_THUMB_AT)))
});

static SLIDER_THUMB_SX: StaticSx = StaticSx::new(|| {
    // A `span`, because the tooltip's wrapper is one - so it needs a box.
    sx().display("block")
        .width(SLIDER_THUMB.value())
        .height(SLIDER_THUMB.value())
        .border_radius("50%")
        // `value_or`'s fallback is raw CSS, so a var, not `"surface"`, which
        // computes to transparent (todo 385).
        .background(SLIDER_THUMB_FILL.value_or(NamedColorCss::SURFACE.value()))
        .border_style("solid")
        .border_width("2px")
        .border_color(SLIDER_COLOR.value())
        .cursor("grab")
        // An invisible 24px square for WCAG 2.5.8. A press picks its thumb by
        // value, not hit-test, so overlapping squares cannot swap them.
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
        // An open ring in the track's own colour: the hole reads at the
        // track's 3:1 on either side of it (WCAG 1.4.11, todo 512).
        .background("surface")
        .border(format!(
            "1px solid {}",
            ColorValue::Shade(Color::Muted, ColorShade::S6).value()
        ))
        // Forced colours would paint it the track's `Canvas`; a filled one
        // stays `Canvas`, on the `Highlight` bar.
        .media(FORCED_COLORS, sx().background("CanvasText"))
        // White on the filled bar, the way the thumb is.
        .when(
            "filled",
            sx().background("surface").border_color("transparent"),
        )
        .rtl(
            sx().left("auto")
                .right(along_track(SLIDER_MARK_AT))
                .transform("translate(50%, -50%)"),
        )
});

static SLIDER_MARK_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        // In thumbs, not `%`, which counts the `marks-labeled` padding. Keep
        // that reserve equal to this offset plus a line, or captions overflow.
        .top(SLIDER_THUMB.value())
        .left(along_track(SLIDER_MARK_AT))
        // Centred in the middle, edge-aligned at the ends: a `-50%` caption
        // on the first or last mark hangs outside the slider's own box.
        .transform(format!(
            "translateX(calc({} * -100%))",
            SLIDER_MARK_AT.value_or("0")
        ))
        .color("muted.7")
        .white_space("nowrap")
        .rtl(
            sx().left("auto")
                .right(along_track(SLIDER_MARK_AT))
                .transform(format!(
                    "translateX(calc({} * 100%))",
                    SLIDER_MARK_AT.value_or("0")
                )),
        )
});

/// On the bar, not the root: the root's scope skips a value move.
fn bar_variables(bar: (f64, f64)) -> String {
    let (from, to) = bar;
    variables()
        .with(SLIDER_FILLED, Some(to.to_string()))
        .with(SLIDER_FILLED_FROM, Some(from.to_string()))
        .with(SLIDER_FILLED_SPAN, Some((to - from).to_string()))
        .render()
}

/// The `f64` engine every `Slider<V>`, `HueSlider` and `AlphaSlider` renders.
/// Non-generic, so it compiles once.
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
    /// `aria-hidden` on the mark captions, for a discrete scale whose
    /// `aria-valuetext` already names each value (todo 513).
    #[props(default)]
    captions_hidden: bool,
    aria_label: Option<String>,
    /// Names the second thumb of a range; `aria_label` names the first.
    aria_label_to: Option<String>,
    /// The field's label id, when a `<label for>` cannot name the thumb.
    labelledby: Option<String>,
    /// The field's filled caption slots, joined.
    describedby: Option<String>,
    /// The field's status is an error.
    invalid: bool,
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

/// What a value move changes. Only the thumbs' scope reads it, so the rest of
/// the slider skips a drag frame.
#[derive(Clone, PartialEq)]
struct Live {
    value: SliderCoreValue,
    thumb_fill: Option<String>,
}

#[component]
pub(in crate::components::form) fn SliderCore(props: SliderCoreProps) -> Element {
    // `f64::clamp` panics on a broken range; `step.max(0.0)` maps NaN to 0.
    let (min, max) = sane_bounds(props.min, props.max);
    let live_now = Live {
        value: props.value.snapped(min, max, props.step.max(0.0)),
        thumb_fill: props.thumb_fill.clone(),
    };
    let mut live = use_signal(|| live_now.clone());
    if *live.peek() != live_now {
        live.set(live_now);
    }
    // The body keeps the value's shape only, so its props compare equal.
    let shape = match props.value {
        SliderCoreValue::Single(_) => SliderCoreValue::Single(0.0),
        SliderCoreValue::Range { .. } => SliderCoreValue::Range { from: 0.0, to: 0.0 },
    };
    rsx! {
        SliderBody {
            live,
            core: SliderCoreProps {
                value: shape,
                thumb_fill: None,
                ..props
            },
        }
    }
}

#[component]
fn SliderBody(live: Signal<Live>, core: SliderCoreProps) -> Element {
    let props = core;
    let theme = use_theme();
    let root_element = use_element();
    let track_element = use_element();
    // Two handles, always: a hook cannot be conditional, and a single-thumb
    // slider simply never mounts the second.
    let thumb_elements = [use_element(), use_element()];
    // Names each thumb's value bubble, so the thumb can point at it.
    let bubble_id = use_id();
    let press_focus = use_context_provider(PressFocus::default);

    let (min, max) = sane_bounds(props.min, props.max);
    let step = props.step.max(0.0);
    let min_range = props.min_range.max(0.0);
    let size = props.size.copied_or(theme.slider.size);
    let color = base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);

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
    // Under RTL the minimum is at the right edge.
    let track_rtl = use_local_state(|| false);
    // The drag's own last value, which the parent may not have echoed yet.
    // Never rendered, so a write redraws nothing.
    let mut latest = use_hook(|| CopyValue::new(live.peek().value));
    // Which thumb the pointer grabbed. Always 0 for a single thumb. A signal:
    // the thumbs' scope reads it for the open bubble.
    let mut active = use_signal(|| 0_usize);

    let oninput = props.oninput;
    let emit = use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
        latest.set(event.value());
        if let Some(oninput) = &oninput {
            oninput.call(event);
        }
    });

    // A `Callback`: `LocalState` is not `Copy`. Unsnapped; `moved` applies
    // the grid.
    let position_at = {
        let (track_left, track_width, thumb_width, track_rtl) = (
            track_left.clone(),
            track_width.clone(),
            thumb_width.clone(),
            track_rtl.clone(),
        );
        use_callback(move |client_x: f64| {
            let thumb = thumb_width.get();
            // The thumb's centre only travels between the two half-thumb
            // insets, so the pointer has to be mapped over that span too.
            let travel = track_width.get() - thumb;
            if travel <= 0.0 {
                return None;
            }
            let along = (client_x - track_left.get() - thumb / 2.0) / travel;
            let along = if track_rtl.get() { 1.0 - along } else { along };
            Some(min + along * (max - min))
        })
    };

    // The drag's two steps, both through `use_callback` so they see this
    // render's bounds - the handlers `use_drag` holds do not.
    let grab = use_callback(move |raw: f64| {
        let value = live.peek().value;
        let index = value.nearest(raw);
        active.set(index);
        emit.call(SliderChangeEvent::Start(
            value.moved(index, raw, min, max, step, min_range),
        ));
        index
    });
    let slide = use_callback(move |raw: f64| {
        // Copied out first: `emit` writes `latest` while the call runs.
        let (from, index) = (*latest.peek(), *active.peek());
        emit.call(SliderChangeEvent::Change(
            from.moved(index, raw, min, max, step, min_range),
        ));
    });

    let drag = use_drag(DragOptions {
        capture: root_element,
        onstart: Callback::new(move |event: DragStart| {
            if !editable {
                event.cancel.call(());
                return;
            }

            let (track_left, track_width, thumb_width) =
                (track_left.clone(), track_width.clone(), thumb_width.clone());
            track_rtl.set(root_element.is_rtl());
            let press_focus = press_focus.clone();
            // Started here, awaited in the task: Blitz locks the document
            // while tasks drain.
            let track_size = track_element.dimensions();
            let track_offset = track_element.client_offset();
            let thumb_size = thumb_elements[0].dimensions();
            // Off the web the drag starts before the geometry; early moves hit
            // `position_at`'s zero-travel guard.
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
                        // `use_drag` cancels the pointerdown and so its focus;
                        // the grabbed thumb is focused here instead.
                        let index = grab.call(raw);
                        let thumb = thumb_elements[index];
                        if focusable && !thumb.is_focused() {
                            // A track press is outside the thumb's `Tooltip`:
                            // tell it this focus is the pointer's (todo 476).
                            press_focus.mark();
                            let _ = thumb.focus();
                            next_task().await;
                            press_focus.clear();
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
            let last = *latest.peek();
            emit.call(SliderChangeEvent::End(last));
        }),
    });

    // One handler for both thumbs: the focused thumb is the one the keys
    // move, so the index comes from whichever element fired.
    let onkeydown = use_callback(move |(index, event): (usize, Event<KeyboardData>)| {
        // Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: a chord is the browser's.
        if !editable || has_shortcut_modifier(&event) {
            return;
        }
        // `step: 0` has no grid, so a key moves 1% of the range, as a native
        // `step="any"` range input does.
        let unit = if step > 0.0 {
            step
        } else {
            (max - min) / 100.0
        };
        let distance = if event.modifiers().shift() {
            theme.slider.big_step
        } else {
            theme.slider.step
        } * unit;

        // `Change` then `End`: a key press settles at once, and callers commit
        // on `End`.
        let value = live.peek().value;
        let go_to = |raw: f64| {
            event.prevent_default();
            let moved = value.moved(index, raw, min, max, step, min_range);
            emit.call(SliderChangeEvent::Change(moved));
            emit.call(SliderChangeEvent::End(moved));
        };

        let thumb = value.thumb(index);
        // Under RTL ArrowLeft raises the value, as on a native range.
        match logical_key(&event) {
            Key::ArrowRight | Key::ArrowUp => go_to(thumb + distance),
            Key::ArrowLeft | Key::ArrowDown => go_to(thumb - distance),
            Key::PageUp => go_to(thumb + theme.slider.big_step * unit),
            Key::PageDown => go_to(thumb - theme.slider.big_step * unit),
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

    let root_variables: Input<Variables> =
        variables().with(SLIDER_COLOR, color.resolve(None)).into();
    let track_class = use_css(Some(&SLIDER_TRACK_SX), CssLayer::Framework);
    let track_style = props
        .track
        .as_ref()
        .map(|background| format!("background: {background}"));

    let hidden = props.name.clone().map(|name| {
        rsx! { SliderHidden { live, name, disabled } }
    });

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
                    SliderThumbs {
                        live,
                        min,
                        max,
                        step,
                        min_range,
                        size,
                        interactive,
                        disabled,
                        label: props.label,
                        marks: props.marks,
                        captions_hidden: props.captions_hidden,
                        aria_labels: [props.aria_label, props.aria_label_to],
                        labelledby: props.labelledby,
                        describedby: props.describedby,
                        invalid: props.invalid,
                        readonly: props.readonly,
                        plain: props.plain,
                        focusable,
                        bubble_id,
                        thumb_elements,
                        onkeydown,
                        dragging: drag.dragging,
                        active,
                    }
                }
                {hidden}
            },
        )
}

/// The bar, the marks and the thumbs: everything a value move redraws.
#[derive(Props, Clone, PartialEq)]
struct SliderThumbsProps {
    live: Signal<Live>,
    min: f64,
    max: f64,
    step: f64,
    min_range: f64,
    size: Size,
    interactive: bool,
    disabled: bool,
    label: Option<Callback<f64, String>>,
    marks: Vec<SliderMark>,
    captions_hidden: bool,
    aria_labels: [Option<String>; 2],
    labelledby: Option<String>,
    describedby: Option<String>,
    invalid: bool,
    readonly: bool,
    plain: bool,
    focusable: bool,
    bubble_id: Signal<String>,
    thumb_elements: [ElementHandle; 2],
    onkeydown: Callback<(usize, Event<KeyboardData>)>,
    dragging: Signal<bool>,
    active: Signal<usize>,
}

#[component]
fn SliderThumbs(props: SliderThumbsProps) -> Element {
    let SliderThumbsProps {
        live,
        min,
        max,
        step,
        min_range,
        size,
        interactive,
        focusable,
        bubble_id,
        thumb_elements,
        onkeydown,
        dragging,
        active,
        ..
    } = props;
    let Live { value, thumb_fill } = live();
    let bar = value.bar(min, max);

    let bar_class = use_css(
        Some(match value {
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
    // The bare value's bubble, `0,5` under `Formats::GERMAN`.
    let decimal_separator = use_formats().decimal_separator;

    let captions_hidden = props.captions_hidden.then_some("true");
    let marks = props.marks.iter().map(|mark| {
        let mark_at = fraction(mark.value, min, max);
        let at = variables()
            .with(SLIDER_MARK_AT, Some(mark_at.to_string()))
            .render();
        // A state, not a var set only when filled: a raw `style` that drops a
        // declaration keeps its last value ([[codebase/css-vars]]).
        let filled = (bar.0 <= mark_at && mark_at <= bar.1).then_some("filled");
        let caption = mark.label.clone().map(|label| {
            rsx! {
                span {
                    class: mark_label_class.clone(),
                    style: "{at}",
                    "aria-hidden": captions_hidden,
                    {label}
                }
            }
        });
        rsx! {
            span { class: mark_class.clone(), "data-state": filled, style: "{at}" }
            {caption}
        }
    });

    let aria_labels = &props.aria_labels;
    let range = matches!(value, SliderCoreValue::Range { .. });
    let thumbs = value.thumbs().enumerate().map(|(index, thumb_value)| {
        let (thumb_min, thumb_max) = value.bounds(index, min, max, step, min_range);
        // Only a custom label is worth an `aria-valuetext` - the bare value
        // is already in `aria-valuenow`.
        let text = props.label.map(|label| label.call(thumb_value));
        let bubble_text = text
            .clone()
            .unwrap_or_else(|| thumb_value.to_string().replacen('.', decimal_separator, 1));
        // `aria-labelledby` beats `aria-label`, so a range thumb lists itself
        // after the label ("Price Minimum"); a single thumb's label wins alone.
        let aria_label = aria_labels[index].clone();
        let (own_id, labelledby) = match (&props.labelledby, &aria_label) {
            (Some(label), Some(_)) if range => {
                let own_id = format!("{label}-thumb-{index}");
                let labelledby = format!("{label} {own_id}");
                (Some(own_id), Some(labelledby))
            }
            _ => (None, props.labelledby.clone()),
        };

        // A `plain` slider has no bubble. It goes after the captions: the
        // value is already in `aria-valuenow`.
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
            // No `aria-required`: ARIA 1.2 does not allow it on `slider`, and a
            // slider always holds a value anyway (todo 532).
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
            .with(SLIDER_THUMB_FILL, thumb_fill.clone())
            .render();

        if props.plain {
            return rsx! {
                span { class: anchor_class.clone(), style: "{at}", {thumb} }
            };
        }

        // A drag holds it open off the thumb; a disabled thumb opens none
        // (todo 596).
        let open = match props.disabled {
            true => Some(false),
            false => (dragging() && active() == index).then_some(true),
        };
        rsx! {
            span { class: anchor_class.clone(), style: "{at}",
                Tooltip {
                    label: rsx! { {bubble_text} },
                    size,
                    // Clear of the thumb's hit area, which overhangs a sub-24px
                    // thumb by up to 6px: the bubble took those presses (todo 649).
                    gap: Size::Sm,
                    open,
                    label_id: bubble_id,
                    // No bridge: it would take the thumb's presses, and Blitz
                    // hits even an overflowing box.
                    sx: sx().selector("&::before", sx().display("none")),
                    {thumb}
                }
            }
        }
    });

    let bar_style = bar_variables(bar);
    rsx! {
        if !props.plain {
            div { class: bar_class, style: bar_style }
        }
        {marks}
        {thumbs}
    }
}

/// The posted values, in their own scope for the same reason as the thumbs.
#[component]
fn SliderHidden(live: Signal<Live>, name: String, disabled: bool) -> Element {
    let inputs = live.read().value.thumbs().map(move |thumb_value| {
        // `Some(true)` or nothing: native writes `false` as a string, which
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
}
