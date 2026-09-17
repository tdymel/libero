use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, States, Variant,
        common::{Orientation, disabled_look_sx, focus_ring_sx, has_shortcut_modifier, neighbour},
        form::Activation,
        inputs::{
            BUTTON_HOVER_VAR, BUTTON_ON_STATE_VAR, BUTTON_SELECTED_VAR, BUTTON_VARS,
            interactive_variant_sx, variant_selected_sx,
        },
        layout::use_box,
    },
    hooks::ElementHandle,
    platform::{ElementApi, logical_key},
    sx::{StaticSx, sx},
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
        // The containing block of the radios below. Without it they are laid
        // out against the viewport: they stay put while a scroll container
        // moves the labels, and focusing one scrolls the document to it.
        .position("relative")
        .display("inline-flex")
        .align_items("center")
        // Pins its own size, so a `Flex` column's `stretch` cannot widen it.
        .width("max-content")
        // A long row wraps rather than run off a phone's page (WCAG 1.4.10).
        .max_width("100%")
        .flex_wrap("wrap")
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
                .white_space("nowrap")
                // One line that never outgrows the strip (todo 481): a long
                // label ends in an ellipsis, and `title` shows the whole.
                .max_width("100%")
                .min_width("0")
                .overflow("hidden"),
        )
        .selector(
            "& > label > span",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis"),
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
        //
        // A shared-out row must fit the box it fills, so a segment may shrink
        // below its label and the label ends in an ellipsis - Mantine's
        // answer. Wrapping was the other option, but one word cannot wrap and
        // a row of segments at different heights stops reading as one strip.
        // The radio's `aria-label` still carries the whole name.
        .when(
            "full-width",
            sx().width("100%").when(
                Orientation::Horizontal.state_name(),
                sx().selector(SEGMENT, sx().flex("1 1 0")),
            ),
        );

    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            sx().selector(
                SEGMENT,
                interactive_variant_sx(
                    variant,
                    &BUTTON_VARS,
                    &BUTTON_HOVER_VAR,
                    &BUTTON_ON_STATE_VAR,
                ),
            )
            // After the variant's own `:hover`, which it ties on specificity.
            .selector(
                "& > label[data-state~=\"checked\"]",
                variant_selected_sx(
                    variant,
                    &BUTTON_VARS,
                    &BUTTON_SELECTED_VAR,
                    &BUTTON_ON_STATE_VAR,
                ),
            )
            // The selected look can set `box-shadow` more specifically than
            // the ring above (`Elevated`); this one outranks it.
            .selector(
                "& > input:focus-visible + label[data-state~=\"checked\"]",
                focus_ring_sx().z_index("2"),
            )
            // In here, so its `GrayText` outranks the variant's label colour.
            .selector(
                "& > label[data-state~=\"disabled\"]",
                disabled_look_sx("not-allowed").pointer_events("none"),
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
    /// What the segment's radio posts: `Options::value`.
    pub value: String,
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
    pub variant: Variant,
    pub full_width: bool,
    pub size: Size,
    pub radius: Size,
    pub gap: Option<Size>,
    /// `false` keeps the radios out of the tab order and stops a label click
    /// from focusing its radio.
    pub focusable: bool,
    /// Focusable and posted, but no click, key or arrow picks a segment.
    pub readonly: bool,
    /// Enter picks, as Space does: outside a `Form` or raw `<form>`, where it
    /// has nothing to submit.
    pub enter: bool,
    /// What the radios post as, and what makes them one exclusive set.
    pub name: String,
    /// The field's wiring, which lands on the `radiogroup` because that is
    /// the element a screen reader names.
    pub labelledby: Option<String>,
    pub describedby: Option<String>,
    pub invalid: bool,
    pub required: bool,
    /// The strip itself, so a segment can focus a sibling by id. The label
    /// click has to focus by hand - see the label's handler below.
    pub element: ElementHandle,
    /// `--lsx-button-*`, rendered. Set on the root, and the segments inherit.
    pub style: String,
    pub attributes: Vec<Attribute>,
}

/// Scoped to this strip's own root, so two controls can hold the same segment
/// count without colliding.
fn focus_segment(element: &ElementHandle, root: &str, index: usize) {
    let selector = format!("#{root}-segment-{index}");
    let _ = element.query_selector(&selector).and_then(|el| el.focus());
}

/// A segment label's `data-state`. The two flags are independent: a picked
/// segment that is then disabled still reads as picked, because its radio
/// stays checked (todo 134).
fn segment_state(shared: &str, disabled: bool, checked: bool) -> String {
    let mut state = shared.to_string();
    if checked {
        state.push_str(" checked");
    }
    if disabled {
        state.push_str(" disabled");
    }
    state
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
        focusable,
        readonly,
        enter,
        name,
        labelledby,
        describedby,
        invalid,
        required,
        element,
        style,
        attributes,
    } = view;

    // The arrow keys step over the disabled segments, and every segment's
    // handler needs the whole set to do it.
    let disabled_segments: Vec<bool> = segments.iter().map(|segment| segment.disabled).collect();
    // One tab stop, as `RadioGroup`: Blitz tabs through every radio of a group.
    let tab_stop = selected
        .filter(|&index| !disabled_segments[index])
        .or_else(|| disabled_segments.iter().position(|off| !off));

    let mut own = States::default()
        .with(orientation.state_name(), true)
        .with(variant.state_name(), true)
        .with("full-width", full_width)
        .with("collapsed", gap.is_none());
    if let Some(gap) = gap {
        own = own.with(gap.state_name(), true);
    }
    let states = own.into();

    // Every segment carries the same pair, so it is built once rather than
    // per segment.
    let segment_states = format!("{} {}", size.state_name(), radius.radius_state_name());

    let items = segments.iter().enumerate().map(|(index, segment)| {
        // The label's click, Space, Enter outside a `Form` and a click on the
        // radio itself all go through `Activation` - see it for why none of them may activate
        // the radio natively (todo 66). A control that is not focusable wants
        // no focus from its label: inside a dropdown that keeps focus on its
        // field, focusing here would blur the field and close it.
        let disabled = segment.disabled;
        let activation = {
            let root = root.clone();
            Activation::focusing(
                move || {
                    if focusable && !disabled {
                        focus_segment(&element, &root, index);
                    }
                },
                move || {
                    if !disabled && !readonly {
                        onselect.call(index);
                    }
                },
            )
            .enter_activates(enter)
        };
        // Inside a `Form`, Enter is left to the browser, which submits it as
        // for a native radio. The arrows move *and* select, which is what a
        // native radio group does and what Blitz, which does neither, now gets
        // too.
        let keydown = {
            let activation = activation.clone();
            let disabled_segments = disabled_segments.clone();
            let root = root.clone();
            move |event: Event<KeyboardData>| {
                // A native radio leaves Alt/Ctrl/Meta+arrow to the browser:
                // Alt+ArrowLeft is Back, not "pick the previous segment".
                if disabled || activation.keydown(&event) || has_shortcut_modifier(&event) {
                    return;
                }
                let step = match logical_key(&event) {
                    Key::ArrowDown | Key::ArrowRight => 1,
                    Key::ArrowUp | Key::ArrowLeft => -1,
                    _ => return,
                };
                // Cancelled even when read-only: the browser's own arrow would
                // move focus and check the next radio.
                event.prevent_default();
                // The arrows move *and* select, so read-only refuses them
                // outright, as `RadioGroup` does - moving focus alone would
                // leave the checked radio, and so the tab stop, behind.
                if readonly {
                    return;
                }
                if let Some(next) = neighbour(&disabled_segments, index, step) {
                    onselect.call(next);
                    focus_segment(&element, &root, next);
                }
            }
        };
        rsx! {
            // The radio sits *beside* its label, not inside it, so the focus
            // ring can be `input:focus-visible + label`. Nesting it would need
            // `label:has(> input:focus-visible)`, and `:has()` is not
            // universally supported - Blitz's stylo rejects it at parse time.
            // `for` binds the two, which is what forwards a label click to the
            // radio everywhere.
            input {
                key: "{index}",
                id: "{root}-segment-{index}",
                r#type: "radio",
                name: "{name}",
                value: segment.value.clone(),
                // The label's text is not the name when the label is an icon,
                // so the radio carries it.
                "aria-label": segment.name.clone(),
                checked: selected == Some(index),
                // `Some(true)` or nothing: dioxus-native writes a `false` bool
                // as the string "false", and Blitz reads `disabled` by
                // presence - so `false` would disable every segment.
                disabled: segment.disabled.then_some(true),
                tabindex: if focusable && tab_stop == Some(index) { "0" } else { "-1" },
                onclick: activation.input_click(),
                oninput: activation.input_input(),
                onkeydown: keydown,
                onkeyup: activation.input_keyup(),
            }
            label {
                r#for: "{root}-segment-{index}",
                "data-state": segment_state(&segment_states, segment.disabled, selected == Some(index)),
                // The whole name, where an ellipsis cuts the visible one.
                title: segment.name.clone(),
                onclick: activation.label_click(),
                {segment.content.clone()}
            }
        }
    });

    use_box()
        .framework_sx(&SEGMENTED_CONTROL_SX)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare()
        .element(&element)
        .attr_default("role", "radiogroup")
        .attr("aria-labelledby", labelledby)
        .attr("aria-describedby", describedby)
        .attr("aria-invalid", invalid.then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .attr("aria-readonly", readonly.then_some("true"))
        .render(HtmlTag::Div, attributes, items.collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::segment_state;

    /// Disabling the picked segment must not hide that it is picked: its
    /// radio stays checked, so the label has to say so too.
    #[test]
    fn a_disabled_pick_stays_checked() {
        assert_eq!(
            segment_state("size-md", true, true),
            "size-md checked disabled"
        );
        assert_eq!(segment_state("size-md", true, false), "size-md disabled");
        assert_eq!(segment_state("size-md", false, true), "size-md checked");
        assert_eq!(segment_state("size-md", false, false), "size-md");
    }
}
