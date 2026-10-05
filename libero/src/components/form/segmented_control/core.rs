use dioxus::prelude::*;

use crate::{
    components::{
        common::Part,
        common::{
            BUTTON_HOVER_VAR, BUTTON_ON_STATE_VAR, BUTTON_SELECTED_VAR, BUTTON_VARS, HtmlTag,
            Orientation, States, ToolbarItem, Variant, disabled_look_sx, focus_ring_sx,
            has_shortcut_modifier, interactive_variant_sx, neighbour, variant_selected_sx,
        },
        form::{Activation, field_parts_enum},
        layout::use_box,
    },
    hooks::{ElementHandle, id_selector},
    platform::{ElementApi, focus_selector, logical_key, next_task},
    sx::{StaticSx, sx},
    theme::{ButtonDefaults, Size, SizeCss},
};

/// The segment itself. A `<label>`, because the radio it wraps is what
/// carries the semantics.
const SEGMENT: &str = "& > label";

field_parts_enum! {
    /// [`SegmentedControl`](super::SegmentedControl)'s inner parts, for its
    /// `parts` prop: a field's, the strip and its segments.
    pub enum SegmentedControlPart {
        /// The connected strip.
        Control = "control" => "& > [data-slot='control']",
        /// One segment's visible label.
        Segment = "segment" => "& > [data-slot='control'] > [data-slot='segment']",
    }
}

static SEGMENTED_CONTROL_SX: StaticSx = StaticSx::new(|| {
    // Nested two `when`s deep, these outrank the radius rule, which is one
    // `when` shallower.
    let collapse_start = "& > label:not(:first-of-type)";
    let collapse_end = "& > label:not(:last-of-type)";

    let base = sx()
        // Contains the radios; against the viewport, focusing one scrolls the
        // document to it.
        .position("relative")
        .display("inline-flex")
        .align_items("center")
        // Pins its own size, so a `Flex` column's `stretch` cannot widen it.
        .width("max-content")
        // A long row wraps rather than run off a phone's page (WCAG 1.4.10).
        .max_width("100%")
        .flex_wrap("wrap")
        // The radio serves screen readers and keys; only the label shows.
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
                // Separates a rich label's two children.
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
        // The ring goes on the label: the radio has no size, and sits beside
        // its label so `+` reaches it.
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
                // Logical sides: under RTL the previous segment is on the right,
                // and Blitz's stylo matches no `:dir()` (todo 781).
                sx().selector(
                    collapse_start,
                    sx().margin_inline_start("-1px")
                        .border_start_start_radius("0")
                        .border_end_start_radius("0"),
                )
                .selector(
                    collapse_end,
                    sx().border_start_end_radius("0").border_end_end_radius("0"),
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
        // Only a row shares out its width; a shrunk segment ellipsizes rather
        // than wraps, and the radio's `aria-label` keeps the whole name.
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
            // The selected look sets `box-shadow` more specifically than the
            // ring above; this one outranks it and composes the marker back.
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
    /// Inside a `Toolbar`: the tab stop's place in its arrow order.
    pub toolbar_item: Option<ToolbarItem>,
    /// Disabled but focusable, as a toolbar keeps it: looks and reads disabled.
    pub soft_disabled: bool,
}

/// Whether an enabled segment lies past `index` towards `step`, without wrapping.
fn has_further(disabled: &[bool], index: usize, step: isize) -> bool {
    match step {
        1 => disabled.iter().skip(index + 1).any(|off| !off),
        _ => disabled.iter().take(index).any(|off| !off),
    }
}

/// Scoped to this strip's own root, so two controls can hold the same segment
/// count without colliding.
fn focus_segment(element: &ElementHandle, root: &str, index: usize) {
    let selector = id_selector(&format!("{root}-segment-{index}"));
    // A WebView queries nothing: the page focuses it by its id.
    let _ = element
        .query_selector(&selector)
        .and_then(|el| el.focus())
        .or_else(|_| focus_selector(&selector));
}

/// A segment label's `data-state`. A picked segment that is then disabled
/// still reads as picked, since its radio stays checked (todo 134).
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
        toolbar_item,
        soft_disabled,
    } = view;
    let bar_axis = toolbar_item.map(ToolbarItem::orientation);

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

    // Shared by every segment, so built once.
    let segment_states = format!("{} {}", size.state_name(), radius.radius_state_name());

    let items = segments.iter().enumerate().map(|(index, segment)| {
        // Every activation goes through `Activation`, never the native radio
        // (todo 66). Unfocusable: focusing here would close a host dropdown.
        let disabled = segment.disabled;
        let activation = {
            let root = root.clone();
            Activation::focusing(
                move || {
                    if focusable && !disabled {
                        // Read-only keeps focus on the tab stop (todo 746).
                        let target = if readonly { tab_stop } else { None };
                        focus_segment(&element, &root, target.unwrap_or(index));
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
        // Enter in a `Form` submits natively. The arrows move and select, as
        // a native radio group does; Blitz does neither on its own.
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
                let key = logical_key(&event);
                let step = match key {
                    Key::ArrowDown | Key::ArrowRight => 1,
                    Key::ArrowUp | Key::ArrowLeft => -1,
                    _ => return,
                };
                // In a toolbar, an arrow along its axis leaves past the last segment.
                let along = match bar_axis {
                    Some(Orientation::Horizontal) => {
                        matches!(key, Key::ArrowLeft | Key::ArrowRight)
                    }
                    Some(Orientation::Vertical) => matches!(key, Key::ArrowUp | Key::ArrowDown),
                    None => false,
                };
                if let Some(item) = toolbar_item
                    && along
                    && (readonly || !has_further(&disabled_segments, index, step))
                {
                    item.pass_on();
                    return;
                }
                // Cancelled even when read-only: the browser's own arrow would
                // move focus and check the next radio.
                event.prevent_default();
                // Read-only refuses them outright: moving focus alone would
                // leave the tab stop behind.
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
            // Beside its label, not inside: nesting would need `:has()`, which
            // Blitz's stylo rejects. `for` forwards the label's click.
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
                // `Some(true)` or nothing: dioxus-native writes `false` as a
                // string, and Blitz reads `disabled` by presence.
                disabled: segment.disabled.then_some(true),
                tabindex: match (focusable && tab_stop == Some(index), toolbar_item) {
                    (true, Some(item)) => item.tabindex(),
                    (true, None) => "0",
                    (false, _) => "-1",
                },
                "data-toolbar-item": toolbar_item
                    .filter(|_| focusable && tab_stop == Some(index))
                    .map(ToolbarItem::key),
                onclick: activation.input_click(),
                oninput: activation.input_input(),
                onkeydown: keydown,
                onkeyup: activation.input_keyup(),
            }
            label {
                r#for: "{root}-segment-{index}",
                "data-slot": SegmentedControlPart::Segment.slot(),
                "data-state": segment_state(
                    &segment_states,
                    segment.disabled || soft_disabled,
                    selected == Some(index),
                ),
                // The whole name, where an ellipsis cuts the visible one.
                title: segment.name.clone(),
                onclick: activation.label_click(),
                {segment.content.clone()}
            }
        }
    });

    // Read-only keeps focus on the checked segment (todo 746); a task later,
    // since the move fires `focusin` again.
    let focusin = {
        let root = root.clone();
        move |_: FocusEvent| {
            if let (true, Some(stop)) = (readonly && focusable, tab_stop) {
                let root = root.clone();
                spawn(async move {
                    next_task().await;
                    focus_segment(&element, &root, stop);
                });
            }
        }
    };
    use_box()
        .framework_sx(&SEGMENTED_CONTROL_SX)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare()
        .element(&element)
        .event("onfocusin", focusin)
        .attr("data-slot", SegmentedControlPart::Control.slot())
        .attr_default("role", "radiogroup")
        .attr("aria-labelledby", labelledby)
        .attr("aria-describedby", describedby)
        .attr("aria-invalid", invalid.then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .attr(
            "aria-readonly",
            (readonly && !soft_disabled).then_some("true"),
        )
        .attr("aria-disabled", soft_disabled.then_some("true"))
        .render(HtmlTag::Div, attributes, items.collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::{has_further, segment_state};

    #[test]
    fn an_arrow_past_the_last_enabled_segment_has_nowhere_further() {
        let disabled = [false, false, true];
        assert!(has_further(&disabled, 0, 1));
        assert!(!has_further(&disabled, 1, 1));
        assert!(!has_further(&disabled, 0, -1));
        assert!(has_further(&disabled, 1, -1));
    }

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
