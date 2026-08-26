use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{Orientation, focus_ring_sx},
        inputs::{
            BUTTON_COLOR_VAR, BUTTON_HOVER_VAR, BUTTON_SELECTED_VAR, BUTTON_VARS, ButtonVariant,
            button_selected_sx, button_variant_sx,
        },
        layout::use_box,
    },
    sx::{StaticSx, Sx, sx},
    theme::{ButtonDefaults, Size, SizeCss},
};

/// The segment itself. A `<label>`, because the radio it wraps is what
/// carries the semantics.
const SEGMENT: &str = "& > label";

static SEGMENTED_CONTROL_SX: StaticSx = StaticSx::new(|| {
    // Nested under the `collapsed` and orientation `when`s, these render as
    // `.cls[data-state~="collapsed"][data-state~="horizontal"] >
    // label:not(:first-of-type)` - enough to beat the radius rule below,
    // which is one `when` shallower.
    let collapse_start = "& > label:not(:first-of-type)";
    let collapse_end = "& > label:not(:last-of-type)";

    let base = sx()
        .display("inline-flex")
        .align_items("center")
        // Pins its own size, so a `Flex` column's `stretch` cannot widen it.
        .width("max-content")
        // The radio is what a screen reader and the keyboard use; the label
        // beside it is the whole of what anyone sees.
        .selector(
            "& > input",
            sx().position("absolute")
                .width("1px")
                .height("1px")
                .opacity("0")
                .margin("0")
                .pointer_events("none"),
        )
        .selector(
            SEGMENT,
            ButtonDefaults::theme_vars()
                .position("relative")
                .display("inline-flex")
                .align_items("center")
                .justify_content("center")
                // Only a rich label has two children to separate, and it is
                // the component's job rather than every caller's `sx`.
                .gap(SizeCss::SPACING.value(Size::Xs))
                .border_style("solid")
                .border_width("1px")
                .font_weight("600")
                .cursor("pointer")
                .user_select("none")
                .white_space("nowrap"),
        )
        // The overlapped border would otherwise cut into the next segment's
        // focus ring and selected background.
        .selector("& > label:hover", sx().z_index("1"))
        .selector("& > label[data-state~=\"checked\"]", sx().z_index("1"))
        // The ring goes on the label, since the input it belongs to is the
        // one thing here with no size - reachable as a sibling because the
        // radio is rendered beside its label rather than inside it.
        .selector("& > input:focus-visible + label", {
            focus_ring_sx().z_index("2")
        })
        .selector(
            "& > label[data-state~=\"disabled\"]",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
        .when(
            Orientation::Vertical.state_name(),
            // Stretch, or each segment sizes to its own label and the column
            // steps in and out down its edges.
            sx().flex_direction("column").align_items("stretch"),
        )
        // `gap` separates the segments, so they keep their own borders and
        // radii - only an ungapped control shares them.
        .when(
            "collapsed",
            sx().when(
                Orientation::Horizontal.state_name(),
                sx().selector(
                    collapse_start,
                    sx().margin_left("-1px")
                        .border_top_left_radius("0")
                        .border_bottom_left_radius("0"),
                )
                .selector(
                    collapse_end,
                    sx().border_top_right_radius("0")
                        .border_bottom_right_radius("0"),
                ),
            )
            .when(
                Orientation::Vertical.state_name(),
                sx().selector(
                    collapse_start,
                    sx().margin_top("-1px")
                        .border_top_left_radius("0")
                        .border_top_right_radius("0"),
                )
                .selector(
                    collapse_end,
                    sx().border_bottom_left_radius("0")
                        .border_bottom_right_radius("0"),
                ),
            ),
        )
        // Only the row shares out its main axis; in a column `flex` would
        // stretch the segments' heights past their size scale, and
        // `align-items: stretch` has already equalised what "full width"
        // means there.
        .when(
            "full-width",
            sx().width("100%").when(
                Orientation::Horizontal.state_name(),
                sx().selector(SEGMENT, sx().flex("1 1 0")),
            ),
        );

    let base = ButtonVariant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            sx().selector(
                SEGMENT,
                button_variant_sx(variant, &BUTTON_VARS, &BUTTON_HOVER_VAR),
            )
            // After the variant's own `:hover`, which it ties on specificity.
            .selector(
                "& > label[data-state~=\"checked\"]",
                button_selected_sx(variant, &BUTTON_COLOR_VAR, &BUTTON_SELECTED_VAR),
            ),
        )
    });

    Size::ALL.into_iter().fold(base, |base, size| {
        base.when(size.state_name(), sx().gap(SizeCss::SPACING.value(size)))
    })
});

/// One segment, with `T` already gone: `content` is the rendered label and
/// `name` the accessible one.
pub(crate) struct SegmentSpec {
    pub name: String,
    pub content: Element,
    pub disabled: bool,
}

pub(crate) struct SegmentedControlView {
    pub segments: Vec<SegmentSpec>,
    /// `None` when `value` is not among the segments - the strip still
    /// renders, with nothing checked.
    pub selected: Option<usize>,
    pub onselect: Callback<usize>,
    pub orientation: Orientation,
    pub variant: ButtonVariant,
    pub full_width: bool,
    pub size: Size,
    pub radius: Size,
    pub gap: Option<Size>,
    /// `--lsx-button-*`, rendered. Set on the root, and the segments inherit.
    pub style: String,
    pub class: Input<ClassList>,
    pub sx: Input<Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// A plain `fn`, not a component: `Vec<Element>` props defeat memoization, so
/// a scope here would cost a scope and buy nothing.
pub(crate) fn render_segmented_control(view: SegmentedControlView, root: String) -> Element {
    let SegmentedControlView {
        segments,
        selected,
        onselect,
        orientation,
        variant,
        full_width,
        size,
        radius,
        gap,
        style,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let mut own = states
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .with(variant.state_name(), true)
        .with("full-width", full_width)
        .with("collapsed", gap.is_none());
    if let Some(gap) = gap {
        own = own.with(gap.state_name(), true);
    }
    let states: Input<States> = own.into();

    // Every segment carries the same pair, so it is built once rather than
    // per segment.
    let segment_states = format!("{} {}", size.state_name(), radius.radius_state_name());

    use_box()
        .framework_sx(&SEGMENTED_CONTROL_SX)
        .class(&class)
        .sx(&user_sx)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare()
        .attr_default("role", "radiogroup")
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                for (index, segment) in segments.iter().enumerate() {
                    // The radio sits *beside* its label, not inside it, so the
                    // focus ring can be `input:focus-visible + label`. Nesting
                    // it would need `label:has(> input:focus-visible)`, and
                    // `:has()` is not universally supported - Blitz's stylo
                    // rejects it at parse time. `for` binds the two, which is
                    // what forwards a label click to the radio everywhere.
                    input {
                        key: "{index}",
                        id: "{root}-segment-{index}",
                        r#type: "radio",
                        name: "{root}",
                        value: "{index}",
                        // The label's text is not the name when the
                        // label is an icon, so the radio carries it.
                        "aria-label": segment.name.clone(),
                        checked: selected == Some(index),
                        // `Some(true)` or nothing: dioxus-native writes a
                        // `false` bool as the string "false", and Blitz
                        // reads `disabled` by presence - so `false` would
                        // disable every segment.
                        disabled: segment.disabled.then_some(true),
                        // `onclick` and cancelled, not `onchange`: dioxus
                        // writes `checked` as a DOM property and skips
                        // unchanged attributes, so a controlled radio that
                        // the browser flipped stays flipped. A cancelled
                        // click restores it and leaves Rust the only
                        // source of truth. Space activates through `click`
                        // too, so the keyboard needs nothing extra there.
                        onclick: {
                            let disabled = segment.disabled;
                            move |event: Event<MouseData>| {
                                event.prevent_default();
                                if !disabled {
                                    onselect.call(index);
                                }
                            }
                        },
                        // Blitz forwards a `<label>` click to its input as
                        // a default action that emits `input`, never
                        // `click` - see `Switch`, same shape. Both
                        // handlers select the same index, so the two
                        // firing together is a no-op rather than a fight.
                        oninput: {
                            let disabled = segment.disabled;
                            move |_: FormEvent| {
                                if !disabled {
                                    onselect.call(index);
                                }
                            }
                        },
                        // A radio takes only Space, on every platform. This is
                        // one control to its user, not a row of radios, and a
                        // control answers Enter - the same call `Switch` makes.
                        onkeydown: {
                            let disabled = segment.disabled;
                            move |event: Event<KeyboardData>| {
                                if event.key() == Key::Enter && !disabled {
                                    event.prevent_default();
                                    onselect.call(index);
                                }
                            }
                        },
                    }
                    label {
                        r#for: "{root}-segment-{index}",
                        "data-state": if segment.disabled {
                            format!("{segment_states} disabled")
                        } else if selected == Some(index) {
                            format!("{segment_states} checked")
                        } else {
                            segment_states.clone()
                        },
                        {segment.content.clone()}
                    }
                }
            },
        )
}
