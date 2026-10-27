use dioxus::prelude::*;

use super::avatar::{avatar_sx, avatar_variables};
use super::spec::AvatarSpec;
use crate::{
    components::{
        common::{
            HtmlTag, Input, ScaleOrCss, States, Variables, Variant, base_props, focus_ring_sx,
            variables,
        },
        data_display::Avatar,
        layout::use_box,
        overlay::Tooltip,
    },
    hooks::{use_id, use_localization, use_theme},
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
        /// The full headcount when `people` lists only some. The `+N` chip counts the
        /// rest; a value below `people.len()` is ignored.
        #[props(default)]
        total: Option<usize>,
        /// Draws the `+N` chip's content from the hidden count, in place of the `+N` text.
        /// The chip keeps its tooltip and accessible name.
        #[props(default)]
        surplus: Option<Callback<usize, Element>>,
        /// How far each circle is pulled over the one before it.
        /// A size word or any CSS, as `spacing: "0"`.
        #[props(default, into)]
        spacing: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<Size>,
        /// A step on `Avatar`'s radius scale, or any CSS, for every member and the chip.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
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
    let spacing = ScaleOrCss::new(props.spacing.as_ref(), theme.avatar_group.spacing);
    let size = props.size.copied_or(theme.avatar.size);
    let variant = props.variant.copied_or(theme.avatar.variant);
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.avatar.color));

    let known = props.people.len();
    let total = props.total.unwrap_or(0).max(known);
    // `max` counts the chip, so it always stands for two or more: no `+1`.
    let shown = match props.max {
        Some(max) if total > max => max.saturating_sub(1).min(known),
        _ => known,
    };
    let hidden = total - shown;
    let circles = shown + usize::from(hidden > 0);

    let root_variables: Input<Variables> = variables()
        .with(AVATAR_GROUP_SPACING, spacing.resolve(SizeCss::SPACING))
        .into();

    // Through `variables`, so a var the chip stops setting is reverted
    // ([[codebase/css-vars]]). The hook runs unconditionally.
    let chip_variables: Input<Variables> = match hidden > 0 {
        // The bottom of the stack, under every member.
        true => avatar_variables(Some(&color), variant, props.radius.as_ref())
            .with(AVATAR_GROUP_INDEX, "1".to_string())
            .into(),
        false => Input::None,
    };
    let tip_id = use_id();
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
        // Fewer listed than counted: the names say there are others (todo 4538).
        let unlisted = names.len() < hidden;
        let names = names.join(", ");
        let holes: &[(&str, &dyn std::fmt::Display)] = &[("n", &hidden), ("names", &names)];
        let count = fill(labels.count, holes);
        // Nobody known to name: no tooltip, and the count is the name.
        let nameable = !names.is_empty();
        // Named by the count only; the tooltip describes it with the names.
        let more = match nameable {
            true => fill(labels.more, holes),
            false => count.clone(),
        };
        let names = match unlisted {
            true => fill(labels.others, holes),
            false => names,
        };
        let content = match &props.surplus {
            Some(surplus) => surplus.call(hidden),
            None => rsx! { "{count}" },
        };

        let chip_states = States::default()
            .with("grouped", true)
            .with(size.state_name(), true)
            .with(variant.state_name(), true);
        let chip = chip_style
            .clone()
            .attr("data-state", chip_states.data_state())
            .attr("role", "img")
            .attr("aria-label", more)
            // Focusable, so the tooltip is reachable without a pointer.
            .attr("tabindex", nameable.then_some("0"))
            .attr("aria-describedby", nameable.then(|| tip_id.cloned()))
            .render(HtmlTag::Span, Vec::new(), content);

        match nameable {
            true => rsx! {
                Tooltip { label: rsx! { "{names}" }, label_id: tip_id.cloned(), {chip} }
            },
            false => chip,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_overflow_chip_is_described_by_the_names_tooltip() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                AvatarGroup { people: vec!["Ada".into(), "Alan".into(), "Grace".into()], max: 2 }
            }
        });
        let id = html
            .split("aria-describedby=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_else(|| panic!("no aria-describedby: {html}"));
        assert!(html.contains(&format!("id=\"{id}\"")), "{html}");
    }

    #[test]
    fn total_counts_the_people_not_listed() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                AvatarGroup { people: vec!["Ada".into(), "Alan".into()], total: 5 }
            }
        });
        assert!(html.contains("+3"), "{html}");
        // A total below the listed people is ignored.
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                AvatarGroup { people: vec!["Ada".into(), "Alan".into()], total: 1 }
            }
        });
        assert!(!html.contains(">+"), "{html}");
    }

    #[test]
    fn total_with_max_counts_from_the_total() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                AvatarGroup {
                    people: vec!["Ada".into(), "Alan".into(), "Grace".into()],
                    max: 3,
                    total: 10,
                }
            }
        });
        assert!(html.contains("+8"), "{html}");
        // Todo 4538: one name for eight says there are others.
        assert!(html.contains("aria-label=\"8 more\""), "{html}");
        assert!(html.contains("Grace and others"), "{html}");
    }

    #[test]
    fn surplus_draws_the_chip_content() {
        #[component]
        fn Group() -> Element {
            rsx! {
                crate::LiberoProvider {
                    AvatarGroup {
                        people: vec!["Ada".into(), "Alan".into(), "Grace".into()],
                        max: 2,
                        surplus: |hidden: usize| rsx! { "and {hidden} more" },
                    }
                }
            }
        }
        let html = dioxus_ssr::render_element(rsx! { Group {} });
        assert!(html.contains("and 2 more"), "{html}");
        assert!(!html.contains(">+2"), "{html}");
    }

    #[test]
    fn a_chip_with_no_known_names_is_not_a_tab_stop() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                AvatarGroup { people: vec!["Ada".into()], total: 4 }
            }
        });
        assert!(html.contains("+3"), "{html}");
        assert!(!html.contains("tabindex"), "{html}");
    }

    #[test]
    fn a_script_src_member_falls_back() {
        let person = AvatarSpec {
            name: "Ada".into(),
            src: Some("javascript:alert(1)".into()),
            ..AvatarSpec::default()
        };
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider { AvatarGroup { people: vec![person] } }
        });
        assert!(!html.contains("javascript"), "{html}");
    }
}
