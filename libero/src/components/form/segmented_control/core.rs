use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, States,
        common::{Orientation, focus_ring_sx},
        form::Activation,
        inputs::{
            BUTTON_COLOR_VAR, BUTTON_HOVER_VAR, BUTTON_SELECTED_VAR, BUTTON_VARS, ButtonVariant,
            button_selected_sx, button_variant_sx,
        },
        layout::use_box,
    },
    hooks::ElementHandle,
    platform::ElementApi,
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
    /// `false` keeps the radios out of the tab order and stops a label click
    /// from focusing its radio.
    pub focusable: bool,
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

/// The next segment in `step`'s direction that can be picked, wrapping and
/// stepping over the disabled ones. `None` when nothing else can be picked.
fn neighbour(disabled: &[bool], current: usize, step: isize) -> Option<usize> {
    let count = disabled.len() as isize;
    if count == 0 {
        return None;
    }
    let mut index = current as isize;
    for _ in 0..count {
        index = ((index + step) % count + count) % count;
        if !disabled[index as usize] {
            return Some(index as usize);
        }
    }
    None
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
        // The label's click, Space, Enter and a click on the radio itself all
        // go through `Activation` - see it for why none of them may activate
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
                    if !disabled {
                        onselect.call(index);
                    }
                },
            )
            .enter_activates()
        };
        // This is one control to its user, not a row of radios, so it answers
        // Enter as well as Space - the same call `Switch` makes. The arrows
        // move *and* select, which is what a native radio group does and what
        // Blitz, which does neither, now gets too.
        let keydown = {
            let activation = activation.clone();
            let disabled_segments = disabled_segments.clone();
            let root = root.clone();
            move |event: Event<KeyboardData>| {
                if disabled || activation.keydown(&event) {
                    return;
                }
                let step = match event.key() {
                    Key::ArrowDown | Key::ArrowRight => 1,
                    Key::ArrowUp | Key::ArrowLeft => -1,
                    _ => return,
                };
                event.prevent_default();
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
                value: "{index}",
                // The label's text is not the name when the label is an icon,
                // so the radio carries it.
                "aria-label": segment.name.clone(),
                checked: selected == Some(index),
                // `Some(true)` or nothing: dioxus-native writes a `false` bool
                // as the string "false", and Blitz reads `disabled` by
                // presence - so `false` would disable every segment.
                disabled: segment.disabled.then_some(true),
                tabindex: (!focusable).then_some("-1"),
                onclick: activation.input_click(),
                oninput: activation.input_input(),
                onkeydown: keydown,
                onkeyup: activation.input_keyup(),
            }
            label {
                r#for: "{root}-segment-{index}",
                "data-state": segment_state(&segment_states, segment.disabled, selected == Some(index)),
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
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {items}
            },
        )
}

#[cfg(test)]
mod tests {
    use super::{neighbour, segment_state};

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

    /// The arrow keys are answered in Rust now, so the wrap SSR cannot reach
    /// is at least pinned here.
    #[test]
    fn stepping_wraps_at_both_ends() {
        let all_on = [false, false, false];
        assert_eq!(neighbour(&all_on, 0, 1), Some(1));
        assert_eq!(neighbour(&all_on, 2, 1), Some(0));
        assert_eq!(neighbour(&all_on, 0, -1), Some(2));
        assert_eq!(neighbour(&all_on, 1, -1), Some(0));
    }

    /// A disabled segment renders but cannot be picked, so the arrows step
    /// over it rather than landing on it.
    #[test]
    fn stepping_skips_the_disabled_segments() {
        let middle_off = [false, true, false];
        assert_eq!(neighbour(&middle_off, 0, 1), Some(2));
        assert_eq!(neighbour(&middle_off, 2, -1), Some(0));

        let only_last = [true, true, false];
        assert_eq!(neighbour(&only_last, 2, 1), Some(2));
    }

    /// Nothing to move to is not a panic and not a wrap onto a disabled
    /// segment - it is simply no move.
    #[test]
    fn stepping_nowhere_selects_nothing() {
        assert_eq!(neighbour(&[], 0, 1), None);
        assert_eq!(neighbour(&[true, true], 0, 1), None);
    }
}
