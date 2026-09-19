use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, PersonIcon, SVG_FIT, States, Variables, Variant, VariantVars,
            base_color, base_props, contrast_color, fill_color, svg_fit, svg_fit_sx,
            svg_fit_variables, text_color, variables, variant_chrome_sx, variant_colors,
        },
        layout::{Box, use_box},
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        AVATAR_GROUP_INDEX, AVATAR_GROUP_RING, AVATAR_RADII, AVATAR_RADIUS, AvatarDefaults, CssVar,
        PAPER_BACKGROUND, Size,
    },
};

pub(super) const AVATAR_COLOR_VAR: CssVar = CssVar::new("--lsx-avatar-color");
pub(super) const AVATAR_FILL_VAR: CssVar = CssVar::new("--lsx-avatar-fill");
pub(super) const AVATAR_CONTRAST_VAR: CssVar = CssVar::new("--lsx-avatar-contrast");
const AVATAR_CONTAINER_VAR: CssVar = CssVar::new("--lsx-avatar-container");
const AVATAR_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-avatar-on-container");

const AVATAR_VARS: VariantVars<'static> = VariantVars {
    color: &AVATAR_COLOR_VAR,
    fill: &AVATAR_FILL_VAR,
    contrast: &AVATAR_CONTRAST_VAR,
    container: &AVATAR_CONTAINER_VAR,
    on_container: &AVATAR_ON_CONTAINER_VAR,
};

/// The circle, shared with the group's overflow chip - which is an avatar in
/// every way but what it means, so it takes the same rules plus a focus state.
/// A function rather than the static itself, the `paper_sx()` arrangement:
/// `framework_sx` has one slot, so a second static has to rebuild this base
/// rather than layer on it.
pub(super) fn avatar_sx() -> Sx {
    let base = AvatarDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // `safe`, not a plain `center`: the box is a fixed square with
        // `overflow: hidden`, and `initials` is whatever the caller wrote. A
        // centred label longer than the circle is cut at *both* ends, so
        // "ABCDEFGHIJ" reads as "DEFG" - the start, which is the part that
        // identifies the person, is the first thing to go, and the overflow
        // ahead of the box is not even scrollable. `safe` centres exactly as
        // before while the label fits and falls back to the start edge once it
        // does not, so nothing about the common one- or two-grapheme avatar
        // changes. `Badge` met the same defect and answered it by scoping its
        // centring to the `circle` arm; an avatar is always the circle, so
        // scoping is not open here [[codebase/components/badge]].
        .justify_content("safe center")
        // A flex parent would otherwise squash the square, and the group is
        // a flex row.
        .flex_shrink("0")
        .overflow("hidden")
        .user_select("none")
        // The initials are a label, not body copy: a line box taller than the
        // glyphs would push them off centre.
        .line_height("1")
        .font_weight("600")
        .selector(
            "& > img",
            sx().width("100%").height("100%").object_fit("cover"),
        )
        // The person glyph, which has no size of its own.
        .selector("& > svg", sx().width("60%").height("60%"))
        // Inside a group: a ring in the page colour is what makes two
        // overlapping circles legible, and the index is the paint order.
        .when(
            "grouped",
            sx().position("relative")
                .z_index(AVATAR_GROUP_INDEX.value())
                .box_shadow(format!(
                    "0 0 0 {} {}",
                    AVATAR_GROUP_RING.value(),
                    PAPER_BACKGROUND.value()
                )),
        );

    // Chrome only, `Icon`'s rule: an avatar is not interactive, so it takes no
    // hover response.
    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            variant_chrome_sx(variant, &AVATAR_VARS),
        )
    });

    // Only the outlined arm, unlike `Button`/`ActionIcon`, which give every
    // variant the width their `border-color` needs. An avatar's child is a
    // picture filling the box, and a border it did not ask for insets that
    // picture by a pixel on three variants that draw no border at all.
    base.when(
        Variant::Outlined.state_name(),
        sx().border_style("solid").border_width("1px"),
    )
}

static AVATAR_BASE_SX: StaticSx = StaticSx::new(avatar_sx);

static AVATAR_IMAGE_SX: StaticSx = StaticSx::new(|| sx().when(SVG_FIT, svg_fit_sx()));

/// The colour set the variant chrome reads, plus the radius override. Takes
/// resolved values rather than the props struct, because the group's overflow
/// chip builds the same set without being an `Avatar`.
pub(super) fn avatar_variables(
    color: Option<&ThemeAwareValue>,
    variant: Variant,
    radius: Option<Size>,
) -> Variables {
    let base = base_color(color);
    let contrast = contrast_color(&base);
    let colors = variant_colors(variant, &base);

    variables()
        .with(AVATAR_COLOR_VAR, text_color(&base))
        .with(AVATAR_FILL_VAR, fill_color(&base))
        .with(AVATAR_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(AVATAR_CONTAINER_VAR, colors.container)
        .with(AVATAR_ON_CONTAINER_VAR, colors.on_container)
        .with(
            AVATAR_RADIUS.override_var(),
            radius.map(|radius| AVATAR_RADII.value(radius)),
        )
}

base_props! {
    pub struct AvatarProps {
        /// The person this avatar stands for. Announced as the accessible
        /// name unless `alt` replaces it.
        #[props(into)]
        name: String,
        /// The picture. Falls back to the rest of the chain once it fails to
        /// load, and an empty string counts as failed.
        #[props(default, into)]
        src: Option<String>,
        /// Drawn when there is no picture. Nothing is derived from `name` -
        /// an initial is a first grapheme cluster, and which one abbreviates a
        /// name is a property of the script.
        #[props(default, into)]
        initials: Option<String>,
        /// Overrides the announced name. `alt: ""` marks the avatar
        /// decorative, for the common case of one sitting beside the person's
        /// visible name - the standard image rule. A decorative avatar is
        /// hidden from the accessibility tree whole, so never put anything
        /// focusable in one: `aria-hidden` does not remove an element from
        /// the tab order.
        #[props(default, into)]
        alt: Option<String>,
        #[props(default, into)]
        size: Input<Size>,
        /// A step on the avatar's own radius scale. The theme's default,
        /// `xxl`, is a circle.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Anything at all in place of the initials - an icon, a glyph.
        children: Option<Element>,
    }
}

/// A person as a fixed square: a picture, and a fallback chain of `children`,
/// `initials` and a person glyph for when there is none.
///
/// Renders a `<span role="img">` carrying `name`, so the subtree is
/// presentational and a screen reader announces "Ada Lovelace" rather than
/// spelling out "A L". Not focusable and not interactive: wrap it in a
/// `Button` or an `Anchor` if it should be either.
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let theme = use_theme();
    let mut errored_src = use_signal(|| None::<String>);

    let size = props.size.copied_or(theme.avatar.size);
    let variant = props.variant.copied_or(theme.avatar.variant);

    let src = props.src.clone().filter(|src| !src.is_empty());
    let failed = src.is_some() && errored_src.read().as_deref() == src.as_deref();

    let variables: Input<Variables> = avatar_variables(
        props.color.as_ref(),
        variant,
        props.radius.as_ref().copied(),
    )
    .into();
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(variant.state_name(), true)
        .into();

    let content = match (src, failed) {
        (Some(src), false) => {
            let errored = src.clone();
            let image_states: Input<States> = States::default().with(SVG_FIT, svg_fit(&src)).into();
            let image_variables: Input<Variables> =
                svg_fit_variables(Variables::new(), &src, "cover").into();
            rsx! {
                Box {
                    component: "img",
                    framework_sx: &AVATAR_IMAGE_SX,
                    states: image_states,
                    variables: image_variables,
                    src,
                    // The root is the `role="img"`, so the picture inside it
                    // is presentational rather than a second image.
                    alt: "",
                    onerror: move |_| errored_src.set(Some(errored.clone())),
                }
            }
        }
        _ => match (props.children, props.initials) {
            (Some(children), _) => children,
            (None, Some(initials)) => rsx! { "{initials}" },
            (None, None) => rsx! { PersonIcon {} },
        },
    };

    // `alt` is the image rule: absent means "use the name", empty means
    // decorative. Both go through `attr_default`, so a caller spreading their
    // own `role`/`aria-label` still wins - a component's own attributes
    // otherwise render last and silently beat the caller's.
    let (role, label) = match props.alt.as_deref() {
        None => ("img", Some(props.name)),
        Some("") => ("presentation", None),
        Some(_) => ("img", props.alt),
    };
    // `role="presentation"` alone drops the element's own semantics and leaves
    // its *contents* in the tree, so a decorative avatar would still read out
    // its initials as loose text - which `<img alt="">`, where the rule comes
    // from, has no equivalent of. The avatar is never focusable, so hiding the
    // subtree costs nothing; a focusable `children` in a decorative avatar
    // would be the one way to make that untrue.
    let hidden = (role == "presentation").then_some("true");

    use_box()
        .framework_sx(&AVATAR_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .focus_ring(false)
        .prepare()
        .attr_default("role", role)
        .attr_default("aria-label", label)
        .attr_default("aria-hidden", hidden)
        .render(HtmlTag::Span, props.attributes, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue};

    #[test]
    fn a_bare_theme_color_becomes_a_shade_plus_its_contrast() {
        let variables =
            avatar_variables(Some(&Color::Error.into()), Variant::Filled, None).to_string();

        assert!(variables.contains(&format!(
            "{}:{};",
            AVATAR_COLOR_VAR.name(),
            ColorValue::Text(Color::Error, ColorShade::S6).value()
        )));
        assert!(variables.contains(AVATAR_CONTRAST_VAR.name()));
    }

    /// A step resolves through the avatar's own radius scale, not the
    /// global one, into the same override var `Image` uses.
    #[test]
    fn a_radius_step_resolves_through_the_avatar_scale() {
        let variables = avatar_variables(None, Variant::Tonal, Some(Size::Md)).to_string();

        assert!(variables.contains(&format!(
            "{}:{};",
            AVATAR_RADIUS.override_var().name(),
            AVATAR_RADII.value(Size::Md)
        )));
    }

    #[test]
    fn no_radius_emits_no_override() {
        let variables = avatar_variables(None, Variant::Tonal, None).to_string();

        assert!(!variables.contains(AVATAR_RADIUS.override_var().name()));
    }
}
