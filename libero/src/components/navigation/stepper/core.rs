use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, Orientation, States, Variables, VisuallyHidden,
        common::{
            CheckIcon, CloseIcon, Rail, RailInset, focus_ring_sx, on_ring_sx, use_closing_focus,
            variables,
        },
        layout::{Collapse, use_box},
    },
    hooks::{id_selector, use_element},
    str_enum::str_enum,
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{
        CssVar, STEPPER_COLOR, STEPPER_COLOR_CONTRAST, STEPPER_CONNECTOR_COLOR,
        STEPPER_CONTENT_PADDING, STEPPER_DESCRIPTION_COLOR, STEPPER_DESCRIPTION_SIZE,
        STEPPER_ERROR, STEPPER_ERROR_CONTRAST, STEPPER_FILL, STEPPER_GAP, STEPPER_LINE_WIDTH,
        STEPPER_MARKER, STEPPER_PENDING, STEPPER_SPACING, Size, StepLabelPosition, StepperDefaults,
    },
};

str_enum! {
    /// What a step's marker shows.
    ///
    /// Three of the four are derived from `active`'s position; `Error` is the
    /// one only the application knows, which is why `Stepper::state` is an
    /// override rather than a source.
    pub enum StepState {
        /// Not reached yet: a ring around the step's number.
        #[default]
        Pending = "pending",
        /// The current step: an accent ring around its number.
        Active = "active",
        /// Before the current one: a filled marker with a check.
        Completed = "completed",
        /// Filled in the error colour with a cross. Changes only the marker -
        /// an errored active step is still the current one.
        Error = "error",
    }
}

/// Per step: the connector's colour, resolved on the `<li>` so one rule
/// serves every step and the accent flips on a `data-state` token.
const STEPPER_CONNECTOR: CssVar = CssVar::new("--lsx-stepper-connector");

/// The root's container name, which the side-label fallback queries.
const STEPPER_CONTAINER: &str = "lsx-stepper";

/// Under the width three side-labelled steps need.
const NARROW: &str = "(max-width: 359.98px)";

fn rail() -> Rail {
    Rail {
        marker: STEPPER_MARKER.value(),
        line: STEPPER_LINE_WIDTH.value(),
        gap: STEPPER_SPACING.value(),
        style: "solid".to_string(),
        color: STEPPER_CONNECTOR.value(),
    }
}

/// `li[data-state~=".."]`, for a step in state `state`.
fn step(state: &str) -> String {
    format!("& > ol > li[data-state~=\"{state}\"]")
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

    let header = sx()
        .position("relative")
        .display("flex")
        .align_items("flex-start")
        .gap(STEPPER_GAP.value())
        .appearance("none")
        .background("transparent")
        .border("0")
        .padding("0")
        .margin("0")
        .font("inherit")
        .color("inherit")
        .text_align("start")
        // A long unbreakable word wraps inside the step instead of widening a
        // narrow page (1.4.10).
        .with("overflow-wrap", "anywhere")
        .selector("& [data-step-marker]", marker)
        // At least a marker tall and centred, so a one-line label sits on the
        // marker's middle and a label with a description grows downward.
        .selector(
            "& [data-step-body]",
            sx().display("flex")
                .flex_direction("column")
                .justify_content("center")
                .min_width("0")
                .min_height(STEPPER_MARKER.value()),
        )
        .selector("& [data-step-label]", sx().font_weight("500"))
        .selector(
            "& [data-step-description]",
            sx().font_size(STEPPER_DESCRIPTION_SIZE.value())
                .color(STEPPER_DESCRIPTION_COLOR.value()),
        );

    // The horizontal connector is a flex item *before* each step but the
    // first, so it is drawn between the previous marker and this one. Its
    // top edge sits at `Rail::connector_start`, the same centreline the
    // vertical rail uses, because the marker is at the top of every step in
    // both label positions.
    let horizontal = sx()
        .selector(
            "& > ol",
            sx().flex_direction("row").align_items("flex-start"),
        )
        .selector(
            "& > ol > li",
            sx().display("flex").align_items("flex-start"),
        )
        .selector("& > ol > li + li", sx().flex("1 1 auto"))
        .selector(
            "& > ol > li + li::before",
            sx().content("\"\"")
                .flex("1 1 auto")
                .min_width(STEPPER_SPACING.value())
                .margin_top(rail.connector_start())
                // The gap a marker keeps from its own label, so a connector
                // sits as close to a marker as the label does.
                .margin_left(STEPPER_GAP.value())
                .margin_right(STEPPER_GAP.value())
                .border_top(format!("{} solid {}", rail.line, rail.color)),
        )
        .selector(
            "& > [data-stepper-content]",
            sx().padding_top(STEPPER_CONTENT_PADDING.value()),
        );

    let below = sx().selector(
        "& > ol > li > [data-step-header]",
        sx().flex_direction("column")
            .align_items("center")
            .text_align("center"),
    );
    // Three side-labelled steps need ~360px; narrower, they broke words
    // mid-word, so they stack as `below` does (1.4.10). Blitz has no
    // container queries: natively only a narrow window stacks them.
    let side = sx()
        .container_query(STEPPER_CONTAINER, NARROW, below.clone())
        .media(NARROW, below.clone());

    // Everything hangs off the item's own inline-start edge, the origin
    // `Rail` measures from: the marker sits at it, the rail runs from under
    // the marker to the next step, and content clears the marker by one gap.
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
        .selector(
            "& > ol > li [data-step-content]",
            sx().padding_left(rail.content_inset(&STEPPER_GAP.value()))
                .padding_top(STEPPER_CONTENT_PADDING.value())
                .padding_bottom(STEPPER_CONTENT_PADDING.value()),
        );

    StepperDefaults::theme_vars()
        .display("block")
        // Fills its column: the horizontal connectors share whatever width
        // there is, and in a flex parent there would otherwise be none.
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
        .selector("& > ol > li > [data-step-header]", header)
        .selector(
            "& > ol > li > button[data-step-header]",
            sx().cursor("pointer"),
        )
        // `appearance: none` and `border: 0` take the UA ring with the
        // button, and a non-clickable header only ever takes focus from the
        // focus return - it needs a ring then too. Doubled to outrank a
        // `Button`'s own ring whatever the stylesheet order, as `Carousel`.
        .selector(
            "& > ol > li > [data-step-header]:focus-visible:focus-visible",
            focus_ring_sx().outline_offset("2px").border_radius("4px"),
        )
        .selector(
            format!("{} [data-step-marker]", step("active")),
            sx().border_color(STEPPER_COLOR.value())
                .color(STEPPER_COLOR.value()),
        )
        .selector(
            format!("{} [data-step-marker]", step("completed")),
            sx().background(STEPPER_FILL.value())
                .border_color(STEPPER_FILL.value())
                .color(STEPPER_COLOR_CONTRAST.value()),
        )
        .selector(
            format!("{} [data-step-marker]", step("error")),
            sx().background(STEPPER_ERROR.value())
                .border_color(STEPPER_ERROR.value())
                .color(STEPPER_ERROR_CONTRAST.value()),
        )
        // The house on-state ring inside the current marker, so current and
        // pending differ in shape too; an errored current step keeps it.
        .selector(
            "& > ol > li > [aria-current=\"step\"] [data-step-marker]",
            on_ring_sx(None),
        )
        // Forced colours paint every fill `Canvas` and every ring `CanvasText`:
        // the completed fill and the current ring would vanish into the rest.
        .media(
            FORCED_COLORS,
            sx().selector(
                format!("{} [data-step-marker]", step("completed")),
                sx().background("Highlight")
                    .border_color("Highlight")
                    .color("HighlightText"),
            )
            .selector(
                format!("{} [data-step-marker]", step("active")),
                sx().border_color("Highlight"),
            )
            // Forcing drops `box-shadow`; an outline survives it.
            .selector(
                "& > ol > li > [aria-current=\"step\"] [data-step-marker]",
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
        .when(
            format!(
                "{} && {}",
                Orientation::Horizontal.state_name(),
                StepLabelPosition::Side.state_name()
            ),
            side,
        )
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
    /// The step's body. Filled for every step in the vertical arm, where a
    /// closing step animates out around its content; empty otherwise.
    pub content: Element,
}

pub(crate) struct StepperView {
    pub steps: Vec<StepSpec>,
    /// How many steps are behind the current one. Every step when finished.
    pub reached: usize,
    /// The current step. `None` when every step is finished, or when
    /// `active` is not among the steps.
    pub current: Option<usize>,
    /// The horizontal arm's one content region, for `current`.
    pub content: Option<Element>,
    pub onstepclick: Option<Callback<usize>>,
    pub allow_next_steps: bool,
    pub orientation: Orientation,
    pub label_position: StepLabelPosition,
    pub size: Size,
    /// Only a per-instance override; the theme's colour is already on `:root`.
    /// `(text role, fill role, the foreground on the fill)`.
    pub color: Option<(String, String, Option<String>)>,
    pub completed_label: &'static str,
    pub error_label: &'static str,
    pub class: Input<ClassList>,
    pub sx: Input<Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// The state a step's position gives it: behind the current step is done,
/// the current one is active, the rest are still to come.
pub(crate) fn derived_state(index: usize, reached: usize, current: Option<usize>) -> StepState {
    if current == Some(index) {
        StepState::Active
    } else if index < reached {
        StepState::Completed
    } else {
        StepState::Pending
    }
}

/// A plain `fn`, not a component: `Vec<Element>` props defeat memoization, so
/// a scope here would cost a scope and buy nothing.
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

    // Focus return. A step's content goes away when `active` moves - it
    // collapses in the vertical arm and is swapped out in the horizontal one -
    // and the usual way `active` moves is a "Continue" button *inside* that
    // content, which drops focus to `<body>`. The destination is the step the
    // user is now on, `aria-current`, the `Pagination` anchor-point rule: it
    // exists whatever was clicked, and Tab from it reaches that step's content.
    // When every step is finished there is no current one, and the step that
    // just closed takes it back instead.
    //
    // Checked during render against the DOM the previous render left, as
    // `Accordion` does: by the time an effect runs a zero-duration panel is
    // already gone. The move itself waits for the effect.
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

    let states: Input<States> = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(orientation.state_name(), true)
        .with(label_position.state_name(), !vertical)
        .into();

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
        // The connector a step carries is the one *before* it horizontally
        // and the one *after* it vertically; either way it is accented once
        // the step it leads away from is done.
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
                "data-stepper-content": "",
                {content}
            }
        },
        _ => rsx! {},
    };

    // The caller's name belongs on the list: `aria-label` is prohibited on
    // the role-less root.
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
                // The explicit role is not redundant: Safari with VoiceOver
                // drops list semantics from a `list-style: none` list.
                ol { role: "list", ..naming, {items.into_iter()} }
                {region}
            },
        )
}

/// One step. Its own scope, so a move redraws the steps it changes and the
/// rest skip; `rich` and a vertical step's `content` are drawn and never do.
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
        StepState::Completed => rsx! { CheckIcon {} },
        StepState::Error => rsx! { CloseIcon {} },
        _ => rsx! { "{index + 1}" },
    };
    // Every part is a `span`: a `<button>` takes phrasing content only.
    let inner = rsx! {
        // The number repeats the list's own position, and the glyphs are
        // drawing; the status text below is what a reader gets.
        span { "data-step-marker": "", "aria-hidden": "true", {marker} }
        span { "data-step-body": "",
            // Name and status in one text node: Chromium puts a space
            // between separate boxes, "Account , Completed" (todo 458).
            span { "data-step-label": "",
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
                span { "data-step-description": "", "{description}" }
            }
        }
    };

    // `aria-current` on the step's own element rather than the `<li>`:
    // tabbing to a button announces the button, not its list item, so on the
    // `<li>` a keyboard user would never hear which step is current.
    let header = match onstepclick {
        Some(onstepclick) => rsx! {
            button {
                id: "{header_id}",
                r#type: "button",
                "data-step-header": "",
                "aria-current": if is_current { "step" },
                onclick: move |_| onstepclick.call(index),
                {inner}
            }
        },
        // Not a control, so no tab stop - but `tabindex="-1"` lets the focus
        // return land here when the steps are not clickable.
        None => rsx! {
            span {
                id: "{header_id}",
                tabindex: "-1",
                "data-step-header": "",
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
                div { "data-step-content": "", {content} }
            }
        }
    });

    rsx! {
        li { "data-state": item_states.data_state(), {header} {body} }
    }
}
