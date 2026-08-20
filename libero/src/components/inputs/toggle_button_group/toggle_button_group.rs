use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{Orientation, base_props},
        inputs::ButtonVariant,
        layout::use_box,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::Size,
    utils::warn,
};

/// What the group hands every `ToggleButton`. Behind a `Signal` because
/// `use_context_provider`/`use_context` each run once: a plain value would be
/// the first render's snapshot forever, and clicking would never repaint.
#[derive(Clone, PartialEq)]
pub(crate) struct ToggleGroupState {
    pub value: Vec<String>,
    pub exclusive: bool,
    pub disabled: bool,
    pub size: Input<Size>,
    pub radius: Input<Size>,
    pub variant: Input<ButtonVariant>,
    pub color: Input<ThemeAwareValue>,
}

#[derive(Clone, Copy)]
pub(crate) struct ToggleGroupContext {
    pub state: Signal<ToggleGroupState>,
    /// Also a signal, for the same reason. Dioxus moves an `EventHandler` over
    /// in place when it diffs, which keeps a captured one current - but only
    /// while both sides are `Some`, so a caller that gates its handler
    /// (`enabled.then(|| ..)`) would otherwise hand the group a `None` it
    /// could never replace.
    pub onchange: Signal<Option<EventHandler<Vec<String>>>>,
}

static TOGGLE_GROUP_SX: StaticSx = StaticSx::new(|| {
    // Nested under the orientation `when`, these render as
    // `.cls[data-state~="horizontal"] > [data-state]:not(:first-child)` -
    // (0,4,0) against `Button`'s own (0,2,0) radius rules. Source order cannot
    // settle this: the two live in different stylesheets, and the registry
    // emits them in hash order.
    let collapse_start = "& > [data-state]:not(:first-child)";
    let collapse_end = "& > [data-state]:not(:last-child)";

    sx().display("inline-flex")
        .align_items("center")
        // Pins its own size, so a `Flex` column's `stretch` cannot widen it.
        .width("max-content")
        // The overlapped border would otherwise cut into the next button's
        // focus ring and selected background.
        .selector("& > *", sx().position("relative"))
        .selector("& > *:hover", sx().z_index("1"))
        .selector("& > *[data-state~=\"checked\"]", sx().z_index("1"))
        .selector("& > *:focus-visible", sx().z_index("2"))
        .when(
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
            // Stretch, or each button sizes to its own label and the column
            // steps in and out down its edges.
            sx().flex_direction("column")
                .align_items("stretch")
                .selector(
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
        )
        // Only the row shares out its main axis; in a column `flex` would
        // stretch the buttons' heights past their size scale, and
        // `align-items: stretch` has already equalised what "full width" means
        // there.
        .when(
            "full-width",
            sx().width("100%").when(
                Orientation::Horizontal.state_name(),
                sx().selector("& > *", sx().flex("1 1 0")),
            ),
        )
});

base_props! {
    pub struct ToggleButtonGroupProps {
        /// Strictly controlled - pair it with `onchange`. `exclusive` holds it
        /// to at most one entry.
        #[props(default)]
        value: Option<Vec<String>>,
        /// Called with the selection the group should take next.
        #[props(default)]
        onchange: Option<EventHandler<Vec<String>>>,
        /// Single-select: picking one clears the rest. On by default.
        #[props(default)]
        exclusive: Option<bool>,
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// The *unselected* look, passed to every button.
        #[props(default, into)]
        variant: Input<ButtonVariant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius of the group's outer corners; inner ones are square.
        #[props(default, into)]
        radius: Input<Size>,
        /// Buttons share the width evenly instead of sizing to their label.
        #[props(default)]
        full_width: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// `ToggleButton`s.
        children: Element,
    }
}

/// A row of connected `ToggleButton`s sharing one selection.
#[component]
pub fn ToggleButtonGroup(props: ToggleButtonGroupProps) -> Element {
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let full_width = props.full_width.unwrap_or(false);

    if props.value.is_some() && props.onchange.is_none() {
        warn("ToggleButtonGroup: `value` without `onchange` can never change.");
    }
    if props.onchange.is_some() && props.value.is_none() {
        warn("ToggleButtonGroup: `onchange` without `value` can never show a selection.");
    }

    let state = ToggleGroupState {
        value: props.value.clone().unwrap_or_default(),
        exclusive: props.exclusive.unwrap_or(true),
        disabled: props.disabled.unwrap_or(false),
        size: props.size.clone(),
        radius: props.radius.clone(),
        variant: props.variant.clone(),
        color: props.color.clone(),
    };

    let context = use_context_provider(|| ToggleGroupContext {
        state: Signal::new(state.clone()),
        onchange: Signal::new(props.onchange),
    });

    // Only the `Some`/`None` transition needs the write - dioxus keeps the box
    // itself current, and a write per render would wake every button.
    if context.onchange.peek().is_some() != props.onchange.is_some() {
        let mut onchange = context.onchange;
        onchange.set(props.onchange);
    }
    // Render-time write, which costs no extra pass; guarded so an unchanged
    // render does not wake every button.
    if *context.state.peek() != state {
        let mut signal = context.state;
        signal.set(state);
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .with("full-width", full_width)
        .into();

    use_box()
        .framework_sx(&TOGGLE_GROUP_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .attr_default("role", "group")
        .render(HtmlTag::Div, props.attributes, props.children)
}
