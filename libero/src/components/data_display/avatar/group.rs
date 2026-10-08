use dioxus::prelude::*;

use super::avatar::{avatar_sx, avatar_variables};
use super::spec::AvatarSpec;
use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, Variant, base_props, focus_ring_sx, variables,
        },
        data_display::Avatar,
        layout::use_box,
        overlay::Tooltip,
    },
    hooks::{use_localization, use_theme},
    localization::fill,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{AVATAR_GROUP_INDEX, AVATAR_GROUP_SPACING, Size, SizeCss},
};

/// How far the focused chip rises, so its neighbour does not cover the focus ring.
const FOCUS_RAISE: u16 = 100;

static AVATAR_GROUP_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .width("fit-content")
        // Each circle after the first is pulled over the previous one.
        .selector(
            "& > * + *",
            sx().margin_left(format!("calc(-1 * {})", AVATAR_GROUP_SPACING.value())),
        )
        // Under RTL the previous circle is on the right.
        .rtl(
            sx().selector(
                "& > * + *",
                sx().margin_left("0")
                    .margin_right(format!("calc(-1 * {})", AVATAR_GROUP_SPACING.value())),
            ),
        )
});

/// The avatar base plus the focus state an avatar never has.
static AVATAR_CHIP_SX: StaticSx = StaticSx::new(|| {
    avatar_sx().focus_visible(focus_ring_sx().z_index(format!(
        "calc({} + {FOCUS_RAISE})",
        AVATAR_GROUP_INDEX.value()
    )))
});

base_props! {
    pub struct AvatarGroupProps {
        /// The members, in paint order: the first is drawn on top.
        people: Vec<AvatarSpec>,
        /// Circles in total, the `+N` chip included.
        #[props(default)]
        max: Option<usize>,
        /// How far each circle is pulled over the one before it.
        #[props(default, into)]
        spacing: Input<Size>,
        #[props(default, into)]
        size: Input<Size>,
        /// A step on `Avatar`'s radius scale, for every member and the chip.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        variant: Input<Variant>,
        /// The tint every member without one of its own takes.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
    }
}

/// A row of overlapping [`Avatar`]s; the people past `max` collapse into a `+N` chip.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::AvatarGroup;
/// # fn app() -> Element {
/// rsx! {
///     AvatarGroup {
///         people: vec!["Ada Lovelace".into(), "Alan Turing".into(), "Grace Hopper".into()],
///         max: 2,
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/avatar>
#[component]
pub fn AvatarGroup(props: AvatarGroupProps) -> Element {
    let theme = use_theme();
    let labels = &use_localization().avatar;
    let spacing = props.spacing.copied_or(theme.avatar_group.spacing);
    let size = props.size.copied_or(theme.avatar.size);
    let variant = props.variant.copied_or(theme.avatar.variant);
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.avatar.color));

    let total = props.people.len();
    // `max` counts the chip, so it always stands for two or more: no `+1`.
    let shown = match props.max {
        Some(max) if total > max => max.saturating_sub(1),
        _ => total,
    };
    let hidden = total - shown;
    let circles = shown + usize::from(hidden > 0);

    let root_variables: Input<Variables> = variables()
        .with(AVATAR_GROUP_SPACING, SizeCss::SPACING.value(spacing))
        .into();

    // Through `variables`, so a var the chip stops setting is reverted
    // ([[codebase/css-vars]]). The hook runs unconditionally.
    let chip_variables: Input<Variables> = match hidden > 0 {
        // The bottom of the stack, under every member.
        true => avatar_variables(Some(&color), variant, props.radius.as_ref().copied())
            .with(AVATAR_GROUP_INDEX, "1".to_string())
            .into(),
        false => Input::None,
    };
    let chip_style = use_box()
        .framework_sx(&AVATAR_CHIP_SX)
        .focus_ring(false)
        .variables(&chip_variables)
        .prepare();

    let members = props
        .people
        .iter()
        .take(shown)
        .enumerate()
        .map(|(index, person)| {
            // Counting down, so the first avatar paints over the second.
            let member_variables = variables()
                .with(AVATAR_GROUP_INDEX, (circles - index).to_string())
                .to_string();

            rsx! {
                Avatar {
                    key: "{index}",
                    name: person.name.clone(),
                    src: person.src.clone(),
                    initials: person.initials.clone(),
                    size,
                    radius: props.radius.clone(),
                    // `input_from_str!` covers strings, not the enum itself.
                    variant: Input::Value(variant),
                    color: match person.color.clone() {
                        Some(color) => Input::Value(color),
                        None => props.color.clone(),
                    },
                    states: States::default().with("grouped", true),
                    style: "{member_variables}",
                }
            }
        });

    let chip = (hidden > 0).then(|| {
        let names: Vec<&str> = props.people[shown..]
            .iter()
            .map(|person| person.name.as_str())
            .collect();
        let names = names.join(", ");
        let holes: &[(&str, &dyn std::fmt::Display)] = &[("n", &hidden), ("names", &names)];
        let count = fill(labels.count, holes);
        let more = fill(labels.more, holes);

        let chip_states = States::default()
            .with("grouped", true)
            .with(size.state_name(), true)
            .with(variant.state_name(), true);
        let chip = chip_style
            .clone()
            .attr("data-state", chip_states.data_state())
            .attr("role", "img")
            // The tooltip's names; the tooltip renders only while open.
            .attr("aria-label", more)
            // Focusable, so the tooltip is reachable without a pointer.
            .attr("tabindex", "0")
            .render(HtmlTag::Span, Vec::new(), rsx! { "{count}" });

        rsx! {
            Tooltip { label: rsx! { "{names}" }, {chip} }
        }
    });

    use_box()
        .framework_sx(&AVATAR_GROUP_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&root_variables)
        .focus_ring(false)
        .prepare()
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                {members}
                {chip}
            },
        )
}
