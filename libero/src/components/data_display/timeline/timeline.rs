use dioxus::prelude::*;

use super::event::TimelineEvent;
use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{
            ABSENT, HtmlTag, Input, LogicalTextAlign, Part, Rail, RailInset, States, Variables,
            base_props, input_from_str, parts_enum, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        Size, TIMELINE_BULLET, TIMELINE_BULLET_BACKGROUND, TIMELINE_COLOR, TIMELINE_CONNECTOR,
        TIMELINE_LINE_COLOR, TIMELINE_LINE_STYLE, TIMELINE_LINE_WIDTH, TIMELINE_MARKER,
        TIMELINE_RADIUS, TIMELINE_SPACE, TimelineAlign, gap_state_name,
    },
};

input_from_str!(TimelineAlign);

/// Horizontal space between the marker and the text column.
const CONTENT_SPACE: &str = "var(--lsx-spacing-md)";

fn rail() -> Rail {
    Rail {
        marker: TIMELINE_BULLET.value(),
        line: TIMELINE_LINE_WIDTH.value(),
        gap: TIMELINE_SPACE.value(),
        style: TIMELINE_LINE_STYLE.value(),
        color: TIMELINE_CONNECTOR.value(),
    }
}

static TIMELINE_BASE_SX: StaticSx = StaticSx::new(|| {
    crate::theme::TimelineDefaults::theme_vars()
        .list_style("none")
        .margin("0")
        .padding("0")
        .display("flex")
        .flex_direction("column")
        .gap(TIMELINE_SPACE.value())
        // A centred rail needs a width to centre in; narrower means a narrower parent.
        .when(TimelineAlign::Alternate.state_name(), sx().width("100%"))
});

/// One event, the positioned ancestor of its `::before` connector and bullet.
static TIMELINE_ITEM_SX: StaticSx = StaticSx::new(|| {
    let rail = rail();
    let inset = rail.content_inset(CONTENT_SPACE);
    // Through `Rail`: `calc(50% + space)` ignored the half-marker and overlapped
    // on the two largest bullets.
    let centred = rail.centred_content_inset(CONTENT_SPACE);

    let base = sx()
        .position("relative")
        // A long word in the title or content wraps instead of widening the page (1.4.10).
        .with("overflow-wrap", "anywhere")
        .var(TIMELINE_MARKER, TIMELINE_LINE_COLOR.value())
        .var(TIMELINE_CONNECTOR, TIMELINE_LINE_COLOR.value())
        // Bullet and connector below flip to the accent independently.
        .when("active", sx().var(TIMELINE_MARKER, TIMELINE_COLOR.value()))
        .when(
            "line-active",
            sx().var(TIMELINE_CONNECTOR, TIMELINE_COLOR.value()),
        );

    // `Rail` is physical, so each logical side takes a swapping `:dir(rtl)` arm.
    let rail_on = |edge: RailInset, padding: &str| {
        // The rail's edge, and the other one, which the LTR arm had set.
        let (near, far) = match edge {
            RailInset::End => ("right", "left"),
            _ => ("left", "right"),
        };
        rail.connector_sx(edge)
            .selector("&:not(:last-of-type)::before", sx().with(far, "auto"))
            .with(format!("padding-{near}"), padding.to_string())
            .with(format!("padding-{far}"), "0")
    };
    let one_sided = base
        .when(
            TimelineAlign::Start.state_name(),
            sx().padding_left(inset.clone())
                .and(rail.connector_sx(RailInset::Start))
                .rtl(rail_on(RailInset::End, &inset)),
        )
        .when(
            TimelineAlign::End.state_name(),
            sx().padding_right(inset.clone())
                .text_align_end()
                .and(rail.connector_sx(RailInset::End))
                .rtl(rail_on(RailInset::Start, &inset)),
        );

    // Alternating at every width: no collapse threshold (see the brain page).
    one_sided.when(
        TimelineAlign::Alternate.state_name(),
        sx().padding_left(centred.clone())
            .and(rail.connector_sx(RailInset::Center))
            .selector(
                "&:nth-of-type(even)",
                sx().padding_left("0")
                    .padding_right(centred.clone())
                    .text_align_end(),
            )
            // Mirrored: the first event's content sits left of the rail.
            .rtl(
                sx().padding_left("0")
                    .padding_right(centred.clone())
                    .selector(
                        "&:nth-of-type(even)",
                        sx().padding_right("0").padding_left(centred.clone()),
                    ),
            ),
    )
});

/// The dot, or the ring around a caller's icon.
static TIMELINE_BULLET_SX: StaticSx = StaticSx::new(|| {
    let rail = rail();

    sx().position("absolute")
        .top("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .width(TIMELINE_BULLET.value())
        .height(TIMELINE_BULLET.value())
        .border_radius(TIMELINE_RADIUS.value())
        .background(TIMELINE_BULLET_BACKGROUND.value())
        .border(format!(
            "{} solid {}",
            TIMELINE_LINE_WIDTH.value(),
            TIMELINE_MARKER.value()
        ))
        .color(TIMELINE_MARKER.value())
        // An active bullet fills: done and pending differ not by colour alone (1.4.1).
        .when("active", sx().background(TIMELINE_MARKER.value()))
        .when(
            "with-child && active",
            // The glyph inverts, or a light one vanishes on the fill.
            sx().color(TIMELINE_BULLET_BACKGROUND.value()),
        )
        // Forced colours paint every fill `Canvas`, so both would be rings again.
        .media(
            FORCED_COLORS,
            sx().when(
                "active",
                sx().background("Highlight").border_color("Highlight"),
            )
            .when("with-child && active", sx().color("HighlightText")),
        )
        .when(
            TimelineAlign::Start.state_name(),
            sx().left("0")
                .right("auto")
                .rtl(sx().right("0").left("auto")),
        )
        .when(
            TimelineAlign::End.state_name(),
            sx().right("0")
                .left("auto")
                .rtl(sx().left("0").right("auto")),
        )
        .when(
            TimelineAlign::Alternate.state_name(),
            // Through `Rail`: the other half of `centred_content_inset`.
            sx().left(rail.centred_marker_start()).right("auto"),
        )
});

static TIMELINE_TITLE_SX: StaticSx = StaticSx::new(|| sx().font_weight("600"));

parts_enum! {
    /// [`Timeline`]'s inner parts, per [`TimelineEvent`]; a nested timeline keeps its own.
    pub enum TimelinePart {
        /// One event's `<li>`.
        Item = "item" => "& > [data-slot='item']",
        /// The dot, or the ring around `TimelineEvent::bullet`.
        Bullet = "bullet" => "& > [data-slot='item'] > [data-slot='bullet']",
        /// Title and content, beside the rail.
        Body = "body" => "& > [data-slot='item'] > [data-slot='body']",
        Title = "title" => "& > [data-slot='item'] > [data-slot='body'] > [data-slot='title']",
    }
}

base_props! {
    parts(TimelinePart);
    pub struct TimelineProps {
        /// The events, in render order.
        #[props(default)]
        items: Vec<TimelineEvent>,
        /// The current event; it and those before draw active. Clamps to the last.
        #[props(default)]
        active: Option<usize>,
        /// Which side of the rail content sits on; `"alternate"` fills its parent.
        #[props(default, into)]
        align: Input<TimelineAlign>,
        /// The active accent. Per-event colours override it.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        bullet_size: Input<Size>,
        /// Bullet corner radius; `Xl` is the dot.
        #[props(default, into)]
        radius: Input<Size>,
        /// Space between events, and so each connector's length.
        #[props(default, into)]
        gap: Input<Size>,
    }
}

/// An ordered list of events drawn against a rail.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Timeline, TimelineEvent};
/// # fn app() -> Element {
/// rsx! {
///     Timeline {
///         active: 1,
///         items: vec![
///             TimelineEvent::new("Ordered"),
///             TimelineEvent::new("Shipped"),
///             TimelineEvent::new("Delivered"),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/timeline>
#[component]
pub fn Timeline(props: TimelineProps) -> Element {
    let theme = use_theme();
    let align = props.align.copied_or(theme.timeline.align);
    let bullet_size = props.bullet_size.copied_or(theme.timeline.bullet_size);
    let radius = props.radius.copied_or(theme.timeline.radius);
    let gap = props.gap.copied_or(theme.timeline.gap);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(align.state_name(), true)
        .with(bullet_size.state_name(), true)
        .with(radius.radius_state_name(), true)
        // The third size axis has no `per_*` helper; see `TimelineDefaults`.
        .with(gap_state_name(gap), true)
        .into();

    let root_variables: Input<Variables> = variables()
        .with(
            TIMELINE_COLOR,
            props.color.as_ref().and_then(|color| color.resolve(None)),
        )
        .into();

    // Prepared once, outside the loop (`use_box` is a hook), and cloned per item.
    let item_style = use_box()
        .framework_sx(&TIMELINE_ITEM_SX)
        .focus_ring(false)
        .prepare();
    let bullet_style = use_box()
        .framework_sx(&TIMELINE_BULLET_SX)
        .focus_ring(false)
        .prepare();
    let title_style = use_box()
        .framework_sx(&TIMELINE_TITLE_SX)
        .focus_ring(false)
        .prepare();

    // Past the end means everything is done, not nothing.
    let active = props
        .active
        .map(|active| active.min(props.items.len().saturating_sub(1)));

    let items = props
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_active = active.is_some_and(|active| index <= active);
            let line_active = active.is_some_and(|active| index < active);
            let is_current = active == Some(index);

            let item_states = States::default()
                .with(align.state_name(), true)
                .with("active", is_active)
                .with("line-active", line_active);
            // `ABSENT` when unset: a raw `style` var it stops declaring would
            // keep its last value ([[codebase/css-vars]]).
            let item_variables = variables()
                .with(TIMELINE_LINE_STYLE, item.line.as_str().to_string())
                .with(
                    TIMELINE_COLOR,
                    item.color
                        .as_ref()
                        .and_then(|color| color.resolve(None))
                        .unwrap_or_else(|| ABSENT.to_string()),
                );

            let bullet_states = States::default()
                .with(align.state_name(), true)
                .with("active", is_active)
                .with("with-child", item.bullet.is_some());

            let bullet = bullet_style
                .clone()
                .attr("data-state", bullet_states.data_state())
                // The rail's drawing; the title is the text.
                .attr("aria-hidden", "true")
                .attr("data-slot", TimelinePart::Bullet.slot())
                .render(HtmlTag::Span, Vec::new(), item.bullet.clone());

            // A rich title is drawn for the eye; the reader gets its plain-text name.
            let title_content = match &item.title.content {
                Some(content) => rsx! {
                    span { "aria-hidden": "true", {content.clone()} }
                    VisuallyHidden { "{item.title.name()}" }
                },
                None => item.title.render(),
            };
            let title = title_style
                .clone()
                .attr("data-slot", TimelinePart::Title.slot())
                .render(HtmlTag::Div, Vec::new(), title_content)?;

            let body = rsx! {
                div { "data-slot": TimelinePart::Body.slot(),
                    {title}
                    {item.content.clone()}
                }
            };

            item_style
                .clone()
                .attr("data-state", item_states.data_state())
                .attr("style", item_variables.to_string())
                // Only the current event; hidden text on every done one is noise.
                .attr("aria-current", is_current.then_some("step"))
                .attr("data-slot", TimelinePart::Item.slot())
                .render(HtmlTag::Li, Vec::new(), vec![bullet, body])
        })
        .collect::<Vec<_>>();

    use_box()
        .framework_sx(&TIMELINE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&root_variables)
        .prepare()
        .attr_default("role", "list")
        .render(HtmlTag::Ol, props.attributes, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<TimelinePart>(),
            [
                ("item", "& > [data-slot='item']"),
                ("bullet", "& > [data-slot='item'] > [data-slot='bullet']"),
                ("body", "& > [data-slot='item'] > [data-slot='body']"),
                (
                    "title",
                    "& > [data-slot='item'] > [data-slot='body'] > [data-slot='title']"
                ),
            ]
        );
    }
}
