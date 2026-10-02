use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{
            ClassList, Glyph, HtmlTag, Input, LogicalTextAlign, Orientation, Part, Rail, RailInset,
            States, Variables, focus_ring_sx, on_ring_sx, parts_enum, use_closing_focus, variables,
        },
        layout::{Collapse, use_box},
    },
    context::IconSlot,
    hooks::{id_selector, use_element},
    str_enum::str_enum,
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{
        CssVar, FOCUS_RING_HALO_SPREAD, STEPPER_COLOR, STEPPER_COLOR_CONTRAST,
        STEPPER_CONNECTOR_COLOR, STEPPER_CONTENT_PADDING, STEPPER_DESCRIPTION_COLOR,
        STEPPER_DESCRIPTION_SIZE, STEPPER_ERROR, STEPPER_ERROR_CONTRAST, STEPPER_FILL, STEPPER_GAP,
        STEPPER_LINE_WIDTH, STEPPER_MARKER, STEPPER_PENDING, STEPPER_SPACING, Size,
        StepLabelPosition, StepperDefaults,
    },
};

str_enum! {
    /// What a step's marker shows. Derived from the order, except `Error`,
    /// which only `Stepper::state` sets.
    pub enum StepState {
        /// Not reached yet: a ring around the step's number.
        #[default]
        Pending = "pending",
        /// The current step: an accent ring around its number.
        Active = "active",
        /// Before the current one: a filled marker with a check.
        Completed = "completed",
        /// Filled in the error colour with a cross; an errored step can still be current.
        Error = "error",
    }
}

/// The connector's colour, set per `<li>` so a `data-state` token flips it.
const STEPPER_CONNECTOR: CssVar = CssVar::new("--lsx-stepper-connector");

/// The root's container name, which the side-label fallback queries.
const STEPPER_CONTAINER: &str = "lsx-stepper";

/// Under the width `steps` side-labelled steps need, 120px each.
fn narrow(steps: usize) -> String {
    format!("(max-width: {}.98px)", steps * 120 - 1)
}

/// States for the step counts past three, whose side labels stack under a wider
/// box; more steps take the last.
const STEP_COUNTS: [&str; 5] = ["steps-4", "steps-5", "steps-6", "steps-7", "steps-8"];

/// The [`STEP_COUNTS`] state for `steps`, `None` for three or fewer.
fn step_count_state(steps: usize) -> Option<&'static str> {
    steps
        .checked_sub(4)
        .map(|index| STEP_COUNTS[index.min(STEP_COUNTS.len() - 1)])
}

fn rail() -> Rail {
    Rail {
        marker: STEPPER_MARKER.value(),
        line: STEPPER_LINE_WIDTH.value(),
        gap: STEPPER_SPACING.value(),
        style: "solid".to_string(),
        color: STEPPER_CONNECTOR.value(),
    }
}

parts_enum! {
    /// [`Stepper`](super::Stepper)'s inner parts, for its `parts` prop. Direct
    /// paths, so a stepper in a step's panel keeps its own.
    pub enum StepperPart {
        /// The `<ol>` of steps.
        List = "list" => "& > [data-slot='list']",
        /// One step's `<li>`; its `data-state` names the step's state.
        Step = "step" => "& > [data-slot='list'] > [data-slot='step']",
        /// The step's button, or a plain box when steps are not clickable.
        Header = "header" => "& > [data-slot='list'] > [data-slot='step'] > [data-slot='header']",
        /// The number, check or cross.
        Marker = "marker" => "& > [data-slot='list'] > [data-slot='step'] > [data-slot='header'] > [data-slot='marker']",
        /// Label and description, beside or below the marker.
        Body = "body" => "& > [data-slot='list'] > [data-slot='step'] > [data-slot='header'] > [data-slot='body']",
        Label = "label" => "& > [data-slot='list'] > [data-slot='step'] > [data-slot='header'] > [data-slot='body'] > [data-slot='label']",
        Description = "description" => "& > [data-slot='list'] > [data-slot='step'] > [data-slot='header'] > [data-slot='body'] > [data-slot='description']",
        /// The `panel` content: below the strip, or under each step when vertical.
        Panel = "panel" => "& > [data-slot='panel'], & > [data-slot='list'] > [data-slot='step'] > * > * > [data-slot='panel']",
    }
}

/// `[data-slot='..']`, for the framework's own selectors.
fn slot(part: StepperPart) -> String {
    format!("[data-slot='{}']", part.slot())
}

/// `li[data-state~=".."]`, for a step in state `state`.
fn step(state: &str) -> String {
    format!("& > ol > li[data-state~=\"{state}\"]")
}

/// The marker of a step in state `state`. `*`, not the header's slot: one more
/// attribute would outrank the current-step ring rule, which ties with it now.
fn step_marker(state: &str) -> String {
    format!("{} > * > {}", step(state), slot(StepperPart::Marker))
}

static STEPPER_SX: StaticSx = StaticSx::new(|| {
    let rail = rail();
    let ring = format!("{} solid", STEPPER_LINE_WIDTH.value());

    let marker = sx()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width(STEPPER_MARKER.value())
        .height(STEPPER_MARKER.value())
        .with("box-sizing", "border-box")
        .border_radius("50%")
        .border(format!("{ring} {}", STEPPER_PENDING.value()))
        .font_weight("600")
        .line_height("1")
        .selector("& > svg", sx().width("60%").height("60%"));
    let marker_slot = slot(StepperPart::Marker);
    let body_slot = slot(StepperPart::Body);

    let header = sx()
        .position("relative")
        .display("flex")
        .align_items("flex-start")
        // Blitz's UA sheet centres a button's content.
        .justify_content("flex-start")
        .gap(STEPPER_GAP.value())
        .appearance("none")
        .background("transparent")
        .border("0")
        .padding("0")
        .margin("0")
        .font("inherit")
        .color("inherit")
        .text_align_start()
        // Not `anywhere`: words stay whole down to the step's min-content, and the strip
        // scrolls before one splits (todo 2030); only a word wider than the strip breaks.
        .with("overflow-wrap", "break-word")
        // No min-content measure: Blitz kept a label broken per glyph from one (734).
        .min_width("0")
        .selector(format!("& > {marker_slot}"), marker)
        // A marker tall and centred: one line sits on the marker's middle, more grow down.
        .selector(
            format!("& > {body_slot}"),
            sx().display("flex")
                .flex_direction("column")
                .justify_content("center")
                .min_width("0")
                .min_height(STEPPER_MARKER.value()),
        )
        .selector(
            format!("& > {body_slot} > {}", slot(StepperPart::Label)),
            sx().font_weight("500"),
        )
        .selector(
            format!("& > {body_slot} > {}", slot(StepperPart::Description)),
            sx().font_size(STEPPER_DESCRIPTION_SIZE.value())
                .color(STEPPER_DESCRIPTION_COLOR.value()),
        );

    // The connector is a `::before` flex item on each step but the first, at
    // `Rail::connector_start`, the vertical rail's centreline.
    // Steps the shrunk connectors can't fit scroll in the strip, not the page (1.4.10).
    // The padding keeps the headers' rings clear of its clip; the margin takes it back.
    let ring_room = FOCUS_RING_HALO_SPREAD.value();
    let horizontal = sx()
        .selector(
            "& > ol",
            sx().flex_direction("row")
                .align_items("flex-start")
                .overflow_x("auto")
                .padding(ring_room.clone())
                .margin_top(format!("calc(-1 * {ring_room})"))
                .margin_bottom(format!("calc(-1 * {ring_room})")),
        )
        // The cap also caps the step's min-content floor, so a word wider than the strip breaks.
        .selector(
            "& > ol > li",
            sx().display("flex")
                .align_items("flex-start")
                .max_width("100%"),
        )
        .selector("& > ol > li + li", sx().flex("1 1 auto"))
        .selector(
            "& > ol > li + li::before",
            sx().content("\"\"")
                .flex("1 1 auto")
                // A quarter of the spacing, so five steps fit a 320px page.
                .min_width(format!("calc({} / 4)", STEPPER_SPACING.value()))
                .margin_top(rail.connector_start())
                // As close to a marker as its label sits.
                .margin_left(STEPPER_GAP.value())
                .margin_right(STEPPER_GAP.value())
                .border_top(format!("{} solid {}", rail.line, rail.color)),
        )
        .selector(
            format!("& > {}", slot(StepperPart::Panel)),
            sx().padding_top(STEPPER_CONTENT_PADDING.value()),
        );

    let header_slot = slot(StepperPart::Header);
    let below = sx().selector(
        format!("& > ol > li > {header_slot}"),
        sx().flex_direction("column")
            .align_items("center")
            .text_align("center"),
    );
    // Under ~120px a step side labels broke mid-word, so they stack (1.4.10). Blitz has
    // no container queries: natively only a narrow window stacks them.
    let stacked = |steps: usize| {
        sx().container_query(STEPPER_CONTAINER, narrow(steps), below.clone())
            .media(narrow(steps), below.clone())
    };
    let side = stacked(3);

    // Marker, rail and content inset all measure from the item's inline-start edge.
    let vertical = sx()
        .selector(
            "& > ol",
            sx().flex_direction("column").gap(STEPPER_SPACING.value()),
        )
        .selector(
            "& > ol > li",
            sx().position("relative")
                .and(rail.connector_sx(RailInset::Start)),
        )
        // `Rail` is physical; under RTL the inline-start edge is the right.
        .rtl(
            sx().selector(
                "& > ol > li",
                rail.connector_sx(RailInset::End)
                    .selector("&:not(:last-of-type)::before", sx().left("auto")),
            ),
        )
        // Through `Collapse`'s two boxes, so a nested stepper's panel keeps its own.
        .selector(
            format!("& > ol > li > * > * > {}", slot(StepperPart::Panel)),
            sx().padding_left(rail.content_inset(&STEPPER_GAP.value()))
                .padding_top(STEPPER_CONTENT_PADDING.value())
                .padding_bottom(STEPPER_CONTENT_PADDING.value())
                .rtl(
                    sx().padding_left("0")
                        .padding_right(rail.content_inset(&STEPPER_GAP.value())),
                ),
        );

    let side_state = format!(
        "{} && {}",
        Orientation::Horizontal.state_name(),
        StepLabelPosition::Side.state_name()
    );
    let by_count = STEP_COUNTS
        .iter()
        .zip(4..)
        .fold(sx(), |acc, (state, steps)| {
            acc.when(format!("{side_state} && {state}"), stacked(steps))
        });

    StepperDefaults::theme_vars()
        .display("block")
        // Else a flex parent leaves the connectors no width to share.
        .width("100%")
        .container(STEPPER_CONTAINER)
        .selector(
            "& > ol",
            sx().list_style("none")
                .margin("0")
                .padding("0")
                .display("flex"),
        )
        .selector(
            "& > ol > li",
            sx().var(STEPPER_CONNECTOR, STEPPER_CONNECTOR_COLOR.value()),
        )
        .selector(
            "& > ol > li[data-state~=\"line-active\"]",
            sx().var(STEPPER_CONNECTOR, STEPPER_COLOR.value()),
        )
        .selector(format!("& > ol > li > {header_slot}"), header)
        .selector(
            format!("& > ol > li > button{header_slot}"),
            sx().cursor("pointer"),
        )
        // Also on a non-clickable header, which the focus return focuses. Doubled
        // to outrank a `Button`'s ring whatever the order, as `Carousel`.
        .selector(
            format!("& > ol > li > {header_slot}:focus-visible:focus-visible"),
            focus_ring_sx().outline_offset("2px").border_radius("4px"),
        )
        .selector(
            step_marker("active"),
            sx().border_color(STEPPER_COLOR.value())
                .color(STEPPER_COLOR.value()),
        )
        .selector(
            step_marker("completed"),
            sx().background(STEPPER_FILL.value())
                .border_color(STEPPER_FILL.value())
                .color(STEPPER_COLOR_CONTRAST.value()),
        )
        .selector(
            step_marker("error"),
            sx().background(STEPPER_ERROR.value())
                .border_color(STEPPER_ERROR.value())
                .color(STEPPER_ERROR_CONTRAST.value()),
        )
        // On-state ring, so current and pending differ in shape too.
        .selector(
            format!("& > ol > li > [aria-current=\"step\"] > {marker_slot}"),
            on_ring_sx(None),
        )
        // Forced colours would flatten the completed fill and current ring into the rest.
        .media(
            FORCED_COLORS,
            sx().selector(
                step_marker("completed"),
                sx().background("Highlight")
                    .border_color("Highlight")
                    .color("HighlightText"),
            )
            .selector(step_marker("active"), sx().border_color("Highlight"))
            // Forcing drops `box-shadow`; an outline survives it.
            .selector(
                format!("& > ol > li > [aria-current=\"step\"] > {marker_slot}"),
                sx().outline("1px solid currentColor")
                    .outline_offset("-3px"),
            ),
        )
        .when(Orientation::Horizontal.state_name(), horizontal)
        .when(
            format!(
                "{} && {}",
                Orientation::Horizontal.state_name(),
                StepLabelPosition::Below.state_name()
            ),
            below,
        )
        .when(side_state, side)
        .and(by_count)
        .when(Orientation::Vertical.state_name(), vertical)
});

/// One step, with `T` already gone.
pub(crate) struct StepSpec {
    /// The accessible name, and the drawn label unless `rich` is set.
    pub name: String,
    /// `OptionLabel::rich`'s drawing, which a reader skips for `name`.
    pub rich: Option<Element>,
    pub description: Option<String>,
    /// What the marker shows: the derived state, or the caller's override.
    pub state: StepState,
    /// The step's body; filled only when vertical.
    pub content: Element,
}

pub(crate) struct StepperView {
    pub steps: Vec<StepSpec>,
    /// How many steps are behind the current one. Every step when finished.
    pub reached: usize,
    /// `None` when every step is finished or `active` is not a step.
    pub current: Option<usize>,
    /// The horizontal arm's one content region, for `current`.
    pub content: Option<Element>,
    pub onstepclick: Option<Callback<usize>>,
    pub allow_next_steps: bool,
    pub orientation: Orientation,
    pub label_position: StepLabelPosition,
    pub size: Size,
    /// Per-instance override: `(text, fill, foreground on the fill)`.
    pub color: Option<(String, String, Option<String>)>,
    pub completed_label: &'static str,
    pub error_label: &'static str,
    pub class: Input<ClassList>,
    pub sx: Input<Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// The state a step's position gives it.
pub(crate) fn derived_state(index: usize, reached: usize, current: Option<usize>) -> StepState {
    if current == Some(index) {
        StepState::Active
    } else if index < reached {
        StepState::Completed
    } else {
        StepState::Pending
    }
}

/// A plain `fn`: `Vec<Element>` props defeat memoization, so a scope buys nothing.
pub(crate) fn render_stepper(view: StepperView, root: String) -> Element {
    let StepperView {
        steps,
        reached,
        current,
        content,
        onstepclick,
        allow_next_steps,
        orientation,
        label_position,
        size,
        color,
        completed_label,
        error_label,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let vertical = orientation == Orientation::Vertical;
    let root_element = use_element();

    // A "Continue" inside closing content drops focus: move it to the current step,
    // or the closed one when finished. Checked in render, as `Accordion` does.
    let previous = use_hook(|| Rc::new(RefCell::new(current)));
    let closing = use_closing_focus(root_element);
    {
        let mut previous = previous.borrow_mut();
        if *previous != current {
            if let Some(closed) = *previous {
                let index = current.unwrap_or(closed);
                closing.closing(
                    id_selector(&format!("{root}-content-{closed}")),
                    id_selector(&format!("{root}-step-{index}")),
                );
            }
            *previous = current;
        }
    }
    let repay = closing.clone();
    use_effect(use_reactive!(|(current,)| {
        let _ = current;
        repay.repay();
    }));

    let mut states = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(orientation.state_name(), true)
        .with(label_position.state_name(), !vertical);
    if let Some(count) = step_count_state(steps.len()) {
        states = states.with(count, true);
    }
    let states: Input<States> = states.into();

    let mut root_variables = variables();
    if let Some((color, fill, contrast)) = color {
        root_variables = root_variables
            .with(STEPPER_COLOR, Some(color))
            .with(STEPPER_FILL, Some(fill))
            .with(STEPPER_COLOR_CONTRAST, contrast);
    }
    let root_variables: Input<Variables> = root_variables.into();

    let last = steps.len().saturating_sub(1);
    let items = steps.into_iter().enumerate().map(|(index, step)| {
        let derived = derived_state(index, reached, current);
        // A step's connector is before it horizontally, after it vertically;
        // accented once the step it leaves is done.
        let line_active = if vertical {
            derived == StepState::Completed && index < last
        } else {
            index > 0 && derived_state(index - 1, reached, current) == StepState::Completed
        };
        let clickable =
            onstepclick.is_some() && (derived != StepState::Pending || allow_next_steps);
        rsx! {
            StepItem {
                key: "{index}",
                index,
                root: root.clone(),
                name: step.name,
                rich: step.rich,
                description: step.description,
                state: step.state,
                line_active,
                is_current: current == Some(index),
                onstepclick: onstepclick.filter(|_| clickable),
                content: vertical.then_some(step.content),
                completed_label,
                error_label,
            }
        }
    });
    let items: Vec<Element> = items.collect();

    let region = match (vertical, current, content) {
        // Keyed by step: a swap replaces the box, so a focused "Continue" is
        // not patched into the next step's button (Blitz keeps focus there).
        (false, Some(current), Some(content)) => rsx! {
            div {
                key: "{current}",
                id: "{root}-content-{current}",
                role: "region",
                "aria-labelledby": "{root}-step-{current}",
                "data-slot": StepperPart::Panel.slot(),
                {content}
            }
        },
        _ => rsx! {},
    };

    // The caller's name goes on the list: `aria-label` is prohibited on the role-less root.
    let (naming, attributes): (Vec<Attribute>, Vec<Attribute>) = attributes
        .into_iter()
        .partition(|attribute| matches!(attribute.name, "aria-label" | "aria-labelledby"));

    use_box()
        .framework_sx(&STEPPER_SX)
        .class(&class)
        .sx(&user_sx)
        .states(&states)
        .variables(&root_variables)
        .prepare()
        .element(&root_element)
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                // Safari with VoiceOver drops list semantics from a `list-style: none` list.
                ol {
                    role: "list",
                    "data-slot": StepperPart::List.slot(),
                    ..naming,
                    {items.into_iter()}
                }
                {region}
            },
        )
}

/// One step, its own scope so a move redraws only the steps it changes.
#[component]
fn StepItem(
    index: usize,
    root: String,
    name: String,
    rich: Option<Element>,
    description: Option<String>,
    state: StepState,
    line_active: bool,
    is_current: bool,
    /// Only while the step can be picked.
    onstepclick: Option<Callback<usize>>,
    /// The vertical arm's body, under the step itself.
    content: Option<Element>,
    completed_label: &'static str,
    error_label: &'static str,
) -> Element {
    let item_states = States::default()
        .with(state.state_name(), true)
        .with("line-active", line_active);
    let header_id = format!("{root}-step-{index}");
    let status = match state {
        StepState::Completed => format!(", {completed_label}"),
        StepState::Error => format!(", {error_label}"),
        _ => String::new(),
    };

    let marker = match state {
        StepState::Completed => {
            rsx! { Glyph { slot: IconSlot::Check, icon: lucide::check::outlined } }
        }
        StepState::Error => rsx! { Glyph { slot: IconSlot::Close, icon: lucide::x::outlined } },
        _ => rsx! { "{index + 1}" },
    };
    // Every part is a `span`: a `<button>` takes phrasing content only.
    let inner = rsx! {
        // Decorative; readers get the status text below.
        span { "data-slot": StepperPart::Marker.slot(), "aria-hidden": "true", {marker} }
        span { "data-slot": StepperPart::Body.slot(),
            // Name and status in one text node: Chromium puts a space
            // between separate boxes, "Account , Completed" (todo 458).
            span { "data-slot": StepperPart::Label.slot(),
                if rich.is_some() || !status.is_empty() {
                    span { "aria-hidden": "true",
                        {rich.unwrap_or_else(|| rsx! { "{name}" })}
                    }
                    VisuallyHidden { "{name}{status}" }
                } else {
                    "{name}"
                }
            }
            if let Some(description) = description {
                span { "data-slot": StepperPart::Description.slot(), "{description}" }
            }
        }
    };

    // `aria-current` on the header, not the `<li>`, which tabbing never announces.
    let header = match onstepclick {
        Some(onstepclick) => rsx! {
            button {
                id: "{header_id}",
                r#type: "button",
                "data-slot": StepperPart::Header.slot(),
                "aria-current": if is_current { "step" },
                onclick: move |_| onstepclick.call(index),
                {inner}
            }
        },
        // No tab stop, but `tabindex="-1"` lets the focus return land here.
        None => rsx! {
            span {
                id: "{header_id}",
                tabindex: "-1",
                "data-slot": StepperPart::Header.slot(),
                "aria-current": if is_current { "step" },
                {inner}
            }
        },
    };

    let body = content.map(|content| {
        rsx! {
            Collapse {
                open: is_current,
                keep_mounted: false,
                id: "{root}-content-{index}",
                role: "region",
                aria_labelledby: "{header_id}",
                div { "data-slot": StepperPart::Panel.slot(), {content} }
            }
        }
    });

    rsx! {
        li {
            "data-slot": StepperPart::Step.slot(),
            "data-state": item_states.data_state(),
            {header}
            {body}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_side_fallback_widens_with_the_step_count() {
        assert_eq!(narrow(3), "(max-width: 359.98px)");
        assert_eq!(narrow(5), "(max-width: 599.98px)");
        assert_eq!(step_count_state(3), None);
        assert_eq!(step_count_state(4), Some("steps-4"));
        assert_eq!(step_count_state(12), Some("steps-8"));
    }
}
