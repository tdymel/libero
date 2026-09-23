use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{
            Glyph, HtmlTag, Input, SVG_FIT, States, Variables, Variant, VariantVars, base_color,
            base_props, contrast_color, fill_color, svg_fit, svg_fit_sx, svg_fit_variables,
            text_color, variables, variant_chrome_sx, variant_colors,
        },
        layout::{Box, use_box},
    },
    context::IconSlot,
    hooks::use_theme,
    platform,
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

/// The circle, shared with the group's overflow chip. A function, not a static:
/// `framework_sx` has one slot, so the chip rebuilds this base.
pub(super) fn avatar_sx() -> Sx {
    let base = AvatarDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // `safe`: overlong initials clip at their end, not both ends.
        .justify_content("safe center")
        .flex_shrink("0")
        .overflow("hidden")
        .user_select("none")
        // A taller line box would push the initials off centre.
        .line_height("1")
        .font_weight("600")
        .selector(
            "& > img",
            sx().width("100%").height("100%").object_fit("cover"),
        )
        // The person glyph, which has no size of its own.
        .selector("& > svg", sx().width("60%").height("60%"))
        .when(LAYERED, sx().position("relative"))
        // In a group: a page-colour ring separates overlaps; the index is paint order.
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

    // Chrome only: an avatar is not interactive, so no hover response.
    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            variant_chrome_sx(variant, &AVATAR_VARS),
        )
    });

    // A border on the outlined arm only: elsewhere it would inset the picture.
    base.when(
        Variant::Outlined.state_name(),
        sx().border_style("solid").border_width("1px"),
    )
}

static AVATAR_BASE_SX: StaticSx = StaticSx::new(avatar_sx);

/// The picture lies over the fallback, for a renderer that fires no `error`.
const LAYERED: &str = "layered";

static AVATAR_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().when(SVG_FIT, svg_fit_sx())
        .when(LAYERED, sx().position("absolute").inset("0"))
});

/// The variant chrome's colours plus the radius override, from resolved values
/// so the group's overflow chip can share it.
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
        /// The person; the accessible name unless `alt` replaces it.
        #[props(into)]
        name: String,
        /// The picture. An empty string or a load failure falls back.
        #[props(default, into)]
        src: Option<String>,
        /// Drawn when there is no picture; never derived from `name`.
        #[props(default, into)]
        initials: Option<String>,
        /// Overrides the announced name; `""` hides the avatar whole, so nothing
        /// focusable may sit in it.
        #[props(default, into)]
        alt: Option<String>,
        #[props(default, into)]
        size: Input<Size>,
        /// A step on the avatar's own radius scale; the default is a circle.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Anything in place of the initials, such as an icon.
        children: Option<Element>,
    }
}

/// A person as a picture, falling back to `children`, `initials`, then a person glyph.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Avatar;
/// # fn app() -> Element {
/// rsx! {
///     Avatar { name: "Ada Lovelace", initials: "AL", src: "/ada.png" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/avatar>
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let theme = use_theme();
    let mut errored_src = use_signal(|| None::<String>);

    let size = props.size.copied_or(theme.avatar.size);
    let variant = props.variant.copied_or(theme.avatar.variant);

    let src = props.src.clone().filter(|src| !src.is_empty());
    let failed = src.is_some() && errored_src.read().as_deref() == src.as_deref();
    // Without an `error` event the fallback waits under the picture, and shows
    // where it fails to load (todo 884).
    let layered = !platform::fires_image_errors() && src.is_some();

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
        .with(LAYERED, layered)
        .into();

    let fallback = match (props.children, props.initials) {
        (Some(children), _) => children,
        (None, Some(initials)) => rsx! { "{initials}" },
        (None, None) => rsx! { Glyph { slot: IconSlot::Person, icon: lucide::user::outlined } },
    };
    let content = match (src, failed) {
        (Some(src), false) => {
            let errored = src.clone();
            let image_states: Input<States> = States::default()
                .with(SVG_FIT, svg_fit(&src))
                .with(LAYERED, layered)
                .into();
            let image_variables: Input<Variables> =
                svg_fit_variables(Variables::new(), &src, "cover").into();
            rsx! {
                if layered {
                    {fallback}
                }
                Box {
                    component: "img",
                    framework_sx: &AVATAR_IMAGE_SX,
                    states: image_states,
                    variables: image_variables,
                    src,
                    // The root is the `role="img"`, not a second image.
                    alt: "",
                    onerror: move |_| errored_src.set(Some(errored.clone())),
                }
            }
        }
        _ => fallback,
    };

    // The image rule: no `alt` uses the name, `""` is decorative.
    let (role, label) = match props.alt.as_deref() {
        None => ("img", Some(props.name)),
        Some("") => ("presentation", None),
        Some(_) => ("img", props.alt),
    };
    // `presentation` alone would leave the initials read as loose text.
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

    /// A step resolves through the avatar's own radius scale, not the global one.
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
