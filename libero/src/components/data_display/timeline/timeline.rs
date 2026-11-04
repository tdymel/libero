use dioxus::prelude::*;

use super::event::TimelineEvent;
use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{
            ABSENT, HtmlTag, Input, LogicalTextAlign, Part, Rail, RailInset, ResponsiveScale,
            States, Variables, base_props, input_from_str, parts_enum, variables, with_own,
        },
        data_display::responsive_props::and_sx,
        layout::use_box,
    },
    hooks::{use_range_css, use_theme},
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        Responsive, Size, SizeCss, TIMELINE_BULLET, TIMELINE_BULLET_BACKGROUND, TIMELINE_COLOR,
        TIMELINE_CONNECTOR, TIMELINE_GAP, TIMELINE_LINE_COLOR, TIMELINE_LINE_STYLE,
        TIMELINE_LINE_WIDTH, TIMELINE_MARKER, TIMELINE_RADIUS, TIMELINE_SPACE, TimelineAlign,
        TimelineDefaults, gap_state_name,
    },
};

input_from_str!(TimelineAlign);
crate::sx::input::input_from_responsive!(str TimelineAlign);

/// Horizontal space between the marker and the text column.
const CONTENT_SPACE: &str = "var(--lsx-spacing-md)";

/// An event's own body and opposite, relative to its `<li>`.
const ITEM_BODY: &str = "& > [data-slot='body']";
const ITEM_OPPOSITE: &str = "& > [data-slot='opposite']";

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
        // Nested timelines must not inherit this one's custom gap.
        .var(TIMELINE_GAP.override_var(), "initial")
        .list_style("none")
        .margin("0")
        .padding("0")
        .display("flex")
        .flex_direction("column")
        .gap(TIMELINE_SPACE.value())
        // A centred rail needs a width to centre in; narrower means a narrower parent.
        .when(TimelineAlign::Alternate.state_name(), sx().width("100%"))
        .when("opposite", sx().width("100%"))
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

    // Beside the marker's half: the centred rail's inset without the `50%`.
    let half = format!("calc({} / 2 + {CONTENT_SPACE})", rail.marker);
    // From `sm` up, a grid of two halves around the centred rail; the body takes the start
    // half unless `content-end`. Not under an align state: those pad for a one-sided rail.
    let centred_halves = sx()
        .with("grid-template-columns", "minmax(0, 1fr) minmax(0, 1fr)")
        .padding_left("0")
        .and(rail.connector_sx(RailInset::Center))
        .rtl(
            sx().padding_right("0")
                .and(rail.connector_sx(RailInset::Center))
                .selector("&:not(:last-of-type)::before", sx().with("right", "auto")),
        )
        .selector(
            ITEM_BODY,
            sx().with("grid-column", "1")
                .with("padding-inline-end", half.clone())
                .text_align_end(),
        )
        .selector(
            ITEM_OPPOSITE,
            sx().with("grid-column", "2")
                .with("padding-inline-start", half.clone())
                .with("padding-inline-end", "0")
                .text_align_start(),
        );
    // Below `sm` the rail runs down the start edge and the opposite takes only its
    // content's width, so a short time does not claim half a phone.
    let opposed = sx()
        .display("grid")
        .with("grid-template-columns", "auto minmax(0, 1fr)")
        .padding_left(inset.clone())
        .and(rail.connector_sx(RailInset::Start))
        .rtl(rail_on(RailInset::End, &inset))
        .selector(
            ITEM_OPPOSITE,
            sx().with("grid-column", "1")
                .with("grid-row", "1")
                .with("padding-inline-end", CONTENT_SPACE),
        )
        .selector(
            ITEM_BODY,
            sx().with("grid-column", "2").with("grid-row", "1"),
        )
        .breakpoint(Size::Sm, centred_halves);
    let content_end = sx()
        .selector(
            ITEM_BODY,
            sx().with("grid-column", "2")
                .with("padding-inline-start", half.clone())
                .with("padding-inline-end", "0")
                .text_align_start(),
        )
        .selector(
            ITEM_OPPOSITE,
            sx().with("grid-column", "1")
                .with("padding-inline-end", half)
                .with("padding-inline-start", "0")
                .text_align_end(),
        );
    let one_sided = one_sided.when("opposite", opposed).when(
        "opposite && content-end",
        sx().breakpoint(Size::Sm, content_end),
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
        // On the start edge below `sm`, where the item has no centred rail.
        .when(
            "opposite",
            sx().left("0")
                .right("auto")
                .rtl(sx().right("0").left("auto"))
                .breakpoint(
                    Size::Sm,
                    sx().left(rail.centred_marker_start())
                        .right("auto")
                        .rtl(sx().left(rail.centred_marker_start()).right("auto")),
                ),
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
        /// An event's `opposite` content, across the rail from the body.
        Opposite = "opposite" => "& > [data-slot='item'] > [data-slot='opposite']",
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
        /// Per breakpoint too: `responsive(TimelineAlign::Start).md(TimelineAlign::Alternate)`;
        /// with `opposite` content, the reading order follows the base.
        #[props(default, into)]
        align: Input<Responsive<TimelineAlign>>,
        /// The active accent. Per-event colours override it.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        bullet_size: Input<Responsive<Size>>,
        /// Bullet corner radius: a size word or any CSS, as `radius: "0"`. `Xl` is the dot.
        /// Per breakpoint too: `responsive(Size::Xs).md(Size::Xl)`.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// Space between events and each connector: a size word or any CSS, as `gap: "0"`.
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Xl)`.
        #[props(default, into)]
        gap: Input<Responsive<ThemeAwareValue>>,
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
    let aligns = props.align.copied_or(theme.timeline.align.into());
    // Item order and `content-end` are Rust's: they follow the base.
    let align = aligns.base();
    let bullet_sizes = props
        .bullet_size
        .copied_or(theme.timeline.bullet_size.into());
    let radius = ResponsiveScale::new(
        props.radius.as_ref(),
        theme.timeline.radius,
        SizeCss::RADIUS,
    );
    let gap = ResponsiveScale::new(props.gap.as_ref(), theme.timeline.gap, TIMELINE_GAP);
    let radius_steps = radius.steps;
    let gap_steps = gap.steps;
    let sx = with_own(and_sx(radius.sx.clone(), gap.sx.clone()), &props.sx);

    // One event with an opposite centres the rail of every event.
    let opposed = props.items.iter().any(|item| item.opposite.is_some());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .responsive(&aligns, TimelineAlign::state_name)
        .with("opposite", opposed)
        .responsive(&bullet_sizes, |size| size.state_name())
        .responsive(&radius_steps, |size| size.radius_state_name())
        // The third size axis has no `per_*` helper; see `TimelineDefaults`.
        .responsive(&gap_steps, gap_state_name)
        .into();
    let mut copies = aligns.range_copies(TimelineAlign::state_name, |align| {
        TIMELINE_BASE_SX.axis_block(align.state_name())
    });
    copies.extend(bullet_sizes.range_copies(|size| size.state_name(), TimelineDefaults::size_sx));
    copies.extend(
        radius_steps.range_copies(|size| size.radius_state_name(), TimelineDefaults::radius_sx),
    );
    copies.extend(gap_steps.range_copies(gap_state_name, |gap| {
        TIMELINE_BASE_SX.axis_block(gap_state_name(gap))
    }));
    use_range_css(&TIMELINE_BASE_SX, copies);
    // An item carries the align words only on a one-sided rail; a bullet centres beside an opposite.
    let item_aligns = (!opposed).then_some(aligns);
    let bullet_aligns = item_aligns.unwrap_or(TimelineAlign::Alternate.into());
    use_range_css(
        &TIMELINE_ITEM_SX,
        item_aligns.as_ref().map_or_else(Vec::new, |aligns| {
            aligns.range_copies(TimelineAlign::state_name, |align| {
                TIMELINE_ITEM_SX.axis_block(align.state_name())
            })
        }),
    );
    use_range_css(
        &TIMELINE_BULLET_SX,
        bullet_aligns.range_copies(TimelineAlign::state_name, |align| {
            TIMELINE_BULLET_SX.axis_block(align.state_name())
        }),
    );

    let root_variables: Input<Variables> = variables()
        .with(
            TIMELINE_COLOR,
            props.color.as_ref().and_then(|color| color.resolve(None)),
        )
        .with(SizeCss::RADIUS.override_var(), radius.custom_css())
        .with(TIMELINE_GAP.override_var(), gap.custom_css())
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

            // The body sits at the end of the rail for `start`, and for an alternating first event.
            let content_end = match align {
                TimelineAlign::Start => true,
                TimelineAlign::End => false,
                TimelineAlign::Alternate => index % 2 == 0,
            };
            let item_states = match &item_aligns {
                Some(aligns) => States::default().responsive(aligns, TimelineAlign::state_name),
                None => States::default(),
            };
            let item_states = item_states
                .with("opposite", opposed)
                .with("content-end", opposed && content_end)
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
                .responsive(&bullet_aligns, TimelineAlign::state_name)
                .with("opposite", opposed)
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

            let opposite = rsx! {
                if let Some(opposite) = &item.opposite {
                    div { "data-slot": TimelinePart::Opposite.slot(), {opposite.clone()} }
                }
            };

            // Reading order follows the drawn one: a `content-end` opposite sits first (5452).
            // Below `sm` every opposite is drawn first; the others keep body first there (accepted).
            let children = match opposed && content_end {
                true => vec![bullet, opposite, body],
                false => vec![bullet, body, opposite],
            };
            item_style
                .clone()
                .attr("data-state", item_states.data_state())
                .attr("style", item_variables.to_string())
                // Only the current event; hidden text on every done one is noise.
                .attr("aria-current", is_current.then_some("step"))
                .attr("data-slot", TimelinePart::Item.slot())
                .render(HtmlTag::Li, Vec::new(), children)
        })
        .collect::<Vec<_>>();

    use_box()
        .framework_sx(&TIMELINE_BASE_SX)
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&sx)
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
                (
                    "opposite",
                    "& > [data-slot='item'] > [data-slot='opposite']"
                ),
            ]
        );
    }

    /// Todo 4630: the two halves around a centred rail start at `sm`; a phone sizes the opposite to its text.
    #[test]
    fn the_opposite_column_takes_half_the_row_only_from_sm_up() {
        let sheet = crate::css::Stylesheet::from(&TIMELINE_ITEM_SX);
        let css = sheet.as_str();

        let phone = css.find("auto minmax(0, 1fr)").expect(css);
        let halves = css.find("minmax(0, 1fr) minmax(0, 1fr)").expect(css);
        let media = css.find("@media (min-width").expect(css);
        assert!(phone < media && media < halves, "{css}");
    }
}
