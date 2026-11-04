use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{
            Glyph, HtmlTag, Input, PROBES_AFTER_MOUNT, Part, SVG_FIT, ScaleOrCss, States,
            Variables, Variant, VariantVars, attr, base_color, base_props, contrast_color,
            fill_color, media_url, names_itself, on_failed_after_mount, parts_enum, responsive_sx,
            svg_fit, svg_fit_sx, svg_fit_variables, text_color, use_name_warning, use_svg_probe,
            variables, variant_chrome_sx, variant_colors, with_own,
        },
        layout::{Box, use_box},
    },
    context::IconSlot,
    hooks::{listener, use_range_css, use_theme},
    platform,
    sx::{FORCED_COLORS, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        AVATAR_GROUP_INDEX, AVATAR_GROUP_RING, AVATAR_OVERHANG, AVATAR_RADII, AVATAR_RADIUS,
        AvatarDefaults, CssVar, PAPER_BACKGROUND, Responsive, Size,
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
            AvatarPart::Image.selector(),
            sx().width("100%").height("100%").object_fit("cover"),
        )
        // The person glyph, which has no size of its own.
        .selector("& > svg", sx().width("60%").height("60%"))
        .when(LAYERED, sx().position("relative"))
        // The picture spills past every edge, centred and whole.
        .when(
            OVERHANG,
            sx().overflow("visible").position("relative").selector(
                AvatarPart::Image.selector(),
                sx().position("absolute")
                    .top(format!("calc(-1 * {})", AVATAR_OVERHANG.value()))
                    .left(format!("calc(-1 * {})", AVATAR_OVERHANG.value()))
                    .width(format!("calc(100% + 2 * {})", AVATAR_OVERHANG.value()))
                    .height(format!("calc(100% + 2 * {})", AVATAR_OVERHANG.value()))
                    .max_width("none")
                    .object_fit("contain"),
            ),
        )
        // In a group: a page-colour ring separates overlaps; the index is paint order.
        .when(
            "grouped",
            sx().position("relative")
                .z_index(AVATAR_GROUP_INDEX.value())
                .box_shadow(format!(
                    "0 0 0 {} {}",
                    AVATAR_GROUP_RING.value(),
                    PAPER_BACKGROUND.value()
                ))
                // Forced colours drop shadows.
                .media(
                    FORCED_COLORS,
                    sx().outline(format!("{} solid Canvas", AVATAR_GROUP_RING.value())),
                ),
        );

    // Chrome only: an avatar is not interactive, so no hover response.
    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            variant_chrome_sx(variant, &AVATAR_VARS),
        )
    });

    // An unfilled member in a group takes the ring's colour, or the circles under it show through.
    // Standard keeps no edge: it is documented as a single-avatar variant (todo 4905).
    let base = [Variant::Outlined, Variant::Standard]
        .into_iter()
        .fold(base, |base, variant| {
            base.when(
                format!("{} && grouped", variant.state_name()),
                grouped_fill_sx(),
            )
        });

    // Forced colours drop the fill; they paint this outline, so the circle stays (todo 5505).
    // Not in a group, whose `Canvas` ring separates the overlaps.
    let base = Variant::ALL
        .iter()
        .filter(|&&variant| variant != Variant::Outlined)
        .fold(base, |base, &variant| {
            base.when(variant.state_name(), outline_sx())
        });

    // A border on the outlined arm only: elsewhere it would inset the picture.
    let base = base.when(Variant::Outlined.state_name(), outlined_border_sx());

    // No fill behind an overhanging picture; with the variant, past its own and the grouped fill.
    Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            format!("{} && {BARE}", variant.state_name()),
            sx().background("transparent"),
        )
    })
}

/// The paper fill an unfilled grouped member takes: see [`avatar_sx`].
fn grouped_fill_sx() -> Sx {
    sx().background(PAPER_BACKGROUND.value())
}

/// Forced colours drop the fill; this outline keeps the circle.
fn outline_sx() -> Sx {
    sx().selector(
        "&:not([data-state~=\"grouped\"])",
        sx().outline("1px solid transparent"),
    )
}

fn outlined_border_sx() -> Sx {
    sx().border_style("solid").border_width("1px")
}

/// A variant's whole look, as its range copy carries it: [`avatar_sx`]'s pieces for one state.
fn variant_sx(variant: Variant) -> Sx {
    let look = variant_chrome_sx(variant, &AVATAR_VARS);
    let look = match variant {
        Variant::Outlined | Variant::Standard => look.and(sx().when("grouped", grouped_fill_sx())),
        _ => look,
    };
    match variant {
        Variant::Outlined => look.and(outlined_border_sx()),
        _ => look.and(outline_sx()),
    }
}

/// A radius step in a range copy: the override var when set, else the step.
fn avatar_radius_sx(size: Size) -> Sx {
    sx().border_radius(
        AVATAR_RADIUS
            .override_var()
            .value_or(AVATAR_RADII.value(size)),
    )
}

static AVATAR_BASE_SX: StaticSx = StaticSx::new(avatar_sx);

/// An avatar's size, variant and radius as `states`, `sx` and vars, its range copies registered
/// on `base`: shared with the group's overflow chip.
pub(super) struct AvatarAxes<'a> {
    /// For the colour vars: `Tonal` if any range is, the only look reading the container colours.
    pub variant: Variant,
    /// A plain radius, for the override var.
    pub plain_radius: Option<&'a ThemeAwareValue>,
    /// A responsive radius's override var per breakpoint, for under the caller's `sx`.
    pub radius_sx: Option<Sx>,
    pub states: States,
}

pub(super) fn use_avatar_axes<'a>(
    base: &'static StaticSx,
    sizes: Responsive<Size>,
    variants: Responsive<Variant>,
    radius: Option<&'a Responsive<ThemeAwareValue>>,
    theme_radius: Size,
    states: States,
) -> AvatarAxes<'a> {
    let variant = match variants
        .breakpoints()
        .any(|(_, variant)| variant == Variant::Tonal)
    {
        true => Variant::Tonal,
        false => variants.base(),
    };
    // A plain radius stays on the override var; a responsive one rides the states and `sx`.
    let radii = radius.filter(|radii| !radii.is_plain());
    let radius_steps =
        radii.map(|radii| radii.map(|value| ScaleOrCss::new(Some(&value), theme_radius).size));
    let plain_radius = radius.filter(|_| radii.is_none()).map(Responsive::base_ref);
    let radius_sx = radii.and_then(|radii| {
        responsive_sx(radii, |sx, value| {
            let css = ScaleOrCss::new(Some(&value), theme_radius)
                .custom_css(AVATAR_RADII)
                .unwrap_or_else(|| "initial".into());
            sx.var(AVATAR_RADIUS.override_var(), css)
        })
    });

    let states = states
        .responsive(&sizes, |size| size.state_name())
        .responsive(&variants, Variant::state_name);
    let states = match &radius_steps {
        Some(steps) => states.responsive(steps, |size| size.radius_state_name()),
        None => states,
    };

    // Only the ranges this instance uses; a plain one registers nothing.
    let mut copies = sizes.range_copies(|size| size.state_name(), AvatarDefaults::size_sx);
    if let Some(steps) = &radius_steps {
        copies.extend(steps.range_copies(|size| size.radius_state_name(), avatar_radius_sx));
    }
    copies.extend(variants.range_copies(Variant::state_name, variant_sx));
    use_range_css(base, copies);

    AvatarAxes {
        variant,
        plain_radius,
        radius_sx,
        states,
    }
}

/// The picture lies over the fallback, for a renderer that fires no `error`.
const LAYERED: &str = "layered";
/// A picture shows with an `overhang`.
const OVERHANG: &str = "overhang";
/// An overhanging picture that is surely loaded, so nothing waits under it to need the fill.
const BARE: &str = "bare";

static AVATAR_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().when(SVG_FIT, svg_fit_sx())
        .when(LAYERED, sx().position("absolute").inset("0"))
});

/// The variant chrome's colours plus the radius override, from resolved values
/// so the group's overflow chip can share it.
pub(super) fn avatar_variables(
    color: Option<&ThemeAwareValue>,
    variant: Variant,
    radius: Option<&ThemeAwareValue>,
) -> Variables {
    let base = base_color("Avatar", color);
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
            radius.and_then(|radius| radius.resolve(Some(AVATAR_RADII))),
        )
}

parts_enum! {
    /// [`Avatar`]'s inner parts, for its `parts` prop. The fallback is bare
    /// content: style it through the root's `sx`.
    pub enum AvatarPart {
        /// The `<img>`, while a picture shows.
        Image = "image" => "& > [data-slot='image']",
    }
}

base_props! {
    parts(AvatarPart);
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
        /// focusable may sit in it. Other blank text counts as unset.
        #[props(default, into)]
        alt: Option<String>,
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
        /// A step on the avatar's own radius scale, or any CSS, as `radius: "0"`; the default is a circle.
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Xxl)`.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// Per breakpoint too: `responsive(Variant::Outlined).md(Variant::Filled)`.
        #[props(default, into)]
        variant: Input<Responsive<Variant>>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Percent of the side the picture spills past each edge, unclipped, whole
        /// (`object-fit: contain`) and with no fill behind it, for a cut-out portrait.
        /// `theme.avatar.overhang` (0, clipped) when unset; the fallback stays clipped.
        #[props(default)]
        overhang: Option<u8>,
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
    // A script URL is no picture: the avatar falls back.
    let own_src = props
        .src
        .as_deref()
        .filter(|src| !src.is_empty())
        .and_then(|src| media_url(src, "Avatar"));
    let probe = use_svg_probe(own_src.as_deref().unwrap_or_default());
    use_name_warning(
        !props.name.trim().is_empty()
            || props
                .alt
                .as_deref()
                .is_some_and(|alt| alt.is_empty() || !alt.trim().is_empty())
            || names_itself(&props.attributes),
        "Avatar: an empty `name` and no `alt`, so it is announced as an unnamed image. Set `alt: \"\"` if it is decorative.",
    );

    let sizes = props.size.copied_or(theme.avatar.size.into());
    let variants = props.variant.copied_or(theme.avatar.variant.into());
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.avatar.color));

    let axes = use_avatar_axes(
        &AVATAR_BASE_SX,
        sizes,
        variants,
        props.radius.as_ref(),
        theme.avatar.radius,
        props.states.unwrap_or_default(),
    );
    let own_sx = axes
        .radius_sx
        .map(|radius_sx| with_own(Some(radius_sx), &props.sx));
    let user_sx = own_sx.as_ref().unwrap_or(&props.sx);

    let src = own_src;
    let failed = src.is_some() && errored_src.read().as_deref() == src.as_deref();
    // Without an `error` event the fallback waits under the picture, and shows
    // where it fails to load (todo 884).
    let layered = !platform::fires_image_errors() && src.is_some();
    let overhang = props.overhang.unwrap_or(theme.avatar.overhang);
    let overhangs = overhang > 0 && src.is_some() && !failed;

    let variables = avatar_variables(Some(&color), axes.variant, axes.plain_radius);
    let variables: Input<Variables> = match overhangs {
        true => variables.with(AVATAR_OVERHANG, Some(format!("{overhang}%"))),
        false => variables,
    }
    .into();
    let states: Input<States> = axes
        .states
        .with(LAYERED, layered)
        .with(OVERHANG, overhangs)
        .with(BARE, overhangs && !layered)
        .into();

    let fallback = match (props.children, props.initials) {
        (Some(children), _) => children,
        (None, Some(initials)) => rsx! { "{initials}" },
        (None, None) => rsx! { Glyph { slot: IconSlot::Person, icon: lucide::user::outlined } },
    };
    let content = match (src, failed) {
        (Some(src), false) => {
            let errored = src.clone();
            let missed = src.clone();
            let image_states: Input<States> = States::default()
                .with(SVG_FIT, svg_fit(&src))
                .with(LAYERED, layered)
                .into();
            let image_variables: Input<Variables> =
                svg_fit_variables(Variables::new(), &src, "cover").into();
            let mut attributes = vec![
                attr("data-slot", AvatarPart::Image.slot()),
                attr("src", src),
                listener("onerror", move |_: Event<ImageData>| {
                    errored_src.set(Some(errored.clone()))
                }),
            ];
            attributes.extend(PROBES_AFTER_MOUNT.then(|| {
                listener("onmounted", move |event: Event<MountedData>| {
                    let failed = missed.clone();
                    on_failed_after_mount(&missed, event.data(), &probe, move || {
                        errored_src.set(Some(failed))
                    });
                })
            }));
            rsx! {
                if layered {
                    {fallback}
                }
                Box {
                    component: "img",
                    framework_sx: &AVATAR_IMAGE_SX,
                    states: image_states,
                    variables: image_variables,
                    // The root is the `role="img"`, not a second image.
                    alt: "",
                    attributes,
                }
            }
        }
        _ => fallback,
    };

    // The image rule: no `alt` uses the name, `""` is decorative. A blank text names nothing.
    let named = |text: String| Some(text).filter(|text| !text.trim().is_empty());
    let (role, label) = match props.alt.as_deref() {
        Some("") => ("presentation", None),
        Some(alt) if !alt.trim().is_empty() => ("img", props.alt),
        _ => ("img", named(props.name)),
    };
    // `presentation` alone would leave the initials read as loose text.
    let hidden = (role == "presentation").then_some("true");

    use_box()
        .framework_sx(&AVATAR_BASE_SX)
        .class(&props.class)
        .sx(user_sx)
        .visibility(&props.hidden_from, &props.visible_from)
        .parts(&props.parts)
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
    use crate::{
        components::common::part_table,
        tokens::{Color, ColorShade, ColorValue},
    };

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<AvatarPart>(),
            [("image", "& > [data-slot='image']")]
        );
    }

    #[test]
    fn a_script_src_falls_back_and_warns() {
        let app = || {
            rsx! {
                crate::LiberoProvider { Avatar { name: "Ada", initials: "AL", src: "javascript:alert(1)" } }
            }
        };
        let warnings = crate::utils::warnings_of(app);
        assert!(
            warnings.iter().any(|w| w.starts_with("Avatar:")),
            "{warnings:?}"
        );
        let html = dioxus_ssr::render_element(app());
        assert!(
            !html.contains("javascript") && !html.contains("<img"),
            "{html}"
        );
        assert!(html.contains("AL"), "{html}");
    }

    #[test]
    fn a_data_src_stays() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider { Avatar { name: "Ada", src: "data:image/png;base64,AAAA" } }
        });
        assert!(html.contains("data:image/png"), "{html}");
    }

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
        let variables = avatar_variables(None, Variant::Tonal, Some(&Size::Md.into())).to_string();

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
