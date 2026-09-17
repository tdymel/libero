use dioxus::prelude::*;

use super::event::TimelineEvent;
use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{ABSENT, Rail, RailInset, base_props, input_from_str, variables},
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
        // `Alternate` centres the rail in the list, so the list needs a width
        // to centre it in. `Left` and `Right` are happy shrink-to-fitting
        // around their content; an alternating timeline that did the same
        // would put two columns either side of a rail centred in its own
        // max-content width, which is not what anyone means by "alternate".
        // A caller who wants it narrower gives it a narrower parent.
        .when(TimelineAlign::Alternate.state_name(), sx().width("100%"))
});

/// One event. The connector is a `::before` on this element, so it has to be
/// the positioned ancestor of both it and the bullet.
static TIMELINE_ITEM_SX: StaticSx = StaticSx::new(|| {
    let rail = rail();
    let inset = rail.content_inset(CONTENT_SPACE);
    // Through `Rail`, not hand-rolled: the marker straddles the midline here,
    // so content starts a half-marker past 50%. Inlining `calc(50% + space)`
    // left a clearance of `space - marker/2`, which is negative on the two
    // largest bullet sizes - and C3's centred arm needs the same expression.
    let centred = rail.centred_content_inset(CONTENT_SPACE);

    let base = sx()
        .position("relative")
        // A long word in the title or content wraps instead of widening the page (1.4.10).
        .with("overflow-wrap", "anywhere")
        .var(TIMELINE_MARKER, TIMELINE_LINE_COLOR.value())
        .var(TIMELINE_CONNECTOR, TIMELINE_LINE_COLOR.value())
        // Both flip to the accent independently: the bullet for this event,
        // the connector for the span below it. That pair is what makes the
        // rail read as progress rather than as a highlight.
        .when("active", sx().var(TIMELINE_MARKER, TIMELINE_COLOR.value()))
        .when(
            "line-active",
            sx().var(TIMELINE_CONNECTOR, TIMELINE_COLOR.value()),
        );

    let one_sided = base
        .when(
            TimelineAlign::Left.state_name(),
            sx().padding_left(inset.clone())
                .and(rail.connector_sx(RailInset::Start)),
        )
        .when(
            TimelineAlign::Right.state_name(),
            sx().padding_right(inset.clone())
                .text_align("right")
                .and(rail.connector_sx(RailInset::End)),
        );

    // The alternating layout, unconditionally. There is no width below which
    // it collapses to one side: choosing `Alternate` in a 200px sidebar
    // buys a cramped alternating timeline, and that is the caller's call to
    // have made rather than the component's to overrule. A threshold also
    // made the mode undemonstrable - see the note in the brain page.
    one_sided.when(
        TimelineAlign::Alternate.state_name(),
        sx().padding_left(centred.clone())
            .and(rail.connector_sx(RailInset::Center))
            .selector(
                "&:nth-of-type(even)",
                sx().padding_left("0")
                    .padding_right(centred.clone())
                    .text_align("right"),
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
        // An active bullet fills, so done and pending differ by shape, not by
        // colour alone (1.4.1).
        .when("active", sx().background(TIMELINE_MARKER.value()))
        .when(
            "with-child && active",
            // The surface the bullet is drawn on: whatever contrast the
            // ring had against the accent, the filled glyph now has.
            sx().color(TIMELINE_BULLET_BACKGROUND.value()),
        )
        // Forced colours paint every fill `Canvas`, so done and pending dots
        // would both be rings again.
        .media(
            FORCED_COLORS,
            sx().when(
                "active",
                sx().background("Highlight").border_color("Highlight"),
            )
            .when("with-child && active", sx().color("HighlightText")),
        )
        .when(
            TimelineAlign::Left.state_name(),
            sx().left("0").right("auto"),
        )
        .when(
            TimelineAlign::Right.state_name(),
            sx().right("0").left("auto"),
        )
        .when(
            TimelineAlign::Alternate.state_name(),
            // Through `Rail` for the same reason the inset is: this and
            // `centred_content_inset` are two halves of one measurement, and
            // C3's centred marker needs both.
            sx().left(rail.centred_marker_start()).right("auto"),
        )
});

static TIMELINE_TITLE_SX: StaticSx = StaticSx::new(|| sx().font_weight("600"));

base_props! {
    pub struct TimelineProps {
        /// The events, in render order.
        #[props(default)]
        items: Vec<TimelineEvent>,
        /// The current event. Bullets `0..=active` and the connectors
        /// `0..active` draw active; out of range clamps to the last event.
        #[props(default)]
        active: Option<usize>,
        /// Which side of the rail content sits on - `"left"` (default),
        /// `"right"`, or `"alternate"`. `"alternate"` alternates at every
        /// width and fills its parent, since a centred rail needs a width to
        /// be centred in.
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
        /// Space between events, which is also the length of each connector.
        #[props(default, into)]
        gap: Input<Size>,
    }
}

/// An ordered list of events drawn against a rail.
///
/// Renders an `<ol role="list">`: the rail draws position and count visually,
/// and the list is how a screen-reader user gets the same two facts. The
/// explicit `role` is not redundant - Safari with VoiceOver drops list
/// semantics from a `list-style: none` list.
///
/// Not interactive: no keyboard contract, no focus, no `tabindex`. Focusable
/// content inside an event keeps document order, which is the visual order.
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

    // Prepared once each, outside the loop - `use_box` is a hook, so it can
    // never be called per item. Every item shares one class per element; only
    // `data-state` and the per-item vars differ, and those are plain
    // attributes. This is `PinField`'s one-prepared-frame-cloned-per-cell.
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

    // `active` names an event, so an index past the end means "everything is
    // done" rather than nothing - clamped once here instead of at each item.
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
            // Always written, `ABSENT` when unset: this `style` is a raw
            // attribute, so a var it stops declaring would keep its last value
            // ([[codebase/css-vars]]).
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
                .render(HtmlTag::Span, Vec::new(), item.bullet.clone());

            let title =
                title_style
                    .clone()
                    .render(HtmlTag::Div, Vec::new(), item.title.render())?;

            let body = rsx! {
                div {
                    {title}
                    {item.content.clone()}
                }
            };

            item_style
                .clone()
                .attr("data-state", item_states.data_state())
                .attr("style", item_variables.to_string())
                // A role token, so it needs no translation - and only the
                // current event carries one. Completed events are conveyed
                // visually; hidden text on every prior item is noise.
                .attr("aria-current", is_current.then_some("step"))
                // Two already-built elements and no markup of our own, so a
                // `vec!` rather than an `rsx!` block wrapping them.
                .render(HtmlTag::Li, Vec::new(), vec![bullet, body])
        })
        .collect::<Vec<_>>();

    use_box()
        .framework_sx(&TIMELINE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&root_variables)
        .prepare()
        .attr_default("role", "list")
        .render(HtmlTag::Ol, props.attributes, items)
}
