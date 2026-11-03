use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, Variant, VariantVars, base_color, base_props,
            contrast_color, css_string, fill_color, literal_contrast, media_url, names_itself,
            text_color, use_name_warning, variables, variant_chrome_sx, variant_colors,
        },
        data_display::{Pictogram, SvgData},
        layout::use_box,
    },
    hooks::{use_cache, use_gradient_style, use_theme},
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, Gradient, ICON_DEFAULT_RADIUS, ICON_DEFAULT_SIZE, ICON_SIZE, ICON_SLOT_PAD, SizeCss,
    },
};

pub(crate) const ICON_COLOR_VAR: CssVar = CssVar::new("--lsx-icon-color");
pub(crate) const ICON_FILL_VAR: CssVar = CssVar::new("--lsx-icon-fill");
pub(crate) const ICON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-icon-contrast");
const ICON_RADIUS_VAR: CssVar = CssVar::new("--lsx-icon-radius");
const ICON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-icon-container");
const ICON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-icon-on-container");
const ICON_GLYPH_VAR: CssVar = CssVar::new("--lsx-icon-glyph");
/// A contained glyph's size: 60% of the box (24px in 40px), floored at 16px below lg
/// so it stays legible, and capped at 80% so it keeps clear of the edge.
const ICON_GLYPH_INSET: &str = "min(max(60%, 16px), 80%)";

pub(crate) const ICON_VARS: VariantVars<'static> = VariantVars {
    color: &ICON_COLOR_VAR,
    fill: &ICON_FILL_VAR,
    contrast: &ICON_CONTRAST_VAR,
    container: &ICON_CONTAINER_VAR,
    on_container: &ICON_ON_CONTAINER_VAR,
};

static ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    // In a field slot the box grows by the pad on every side and the block margin takes it back:
    // the glyph keeps its size and the container fills the slot button's box (4120).
    let pad = ICON_SLOT_PAD.value_or("0px");
    let side = format!("calc({} + 2 * {pad})", ICON_DEFAULT_SIZE.overridable());
    let base = sx()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        // So an outlined variant's border colour draws (as Badge).
        .border_style("solid")
        .border_width("1px")
        .width(side.clone())
        .height(side)
        .with("padding", pad.clone())
        .with("margin-block", format!("calc(-1 * {pad})"))
        .border_radius(ICON_RADIUS_VAR.value_or(ICON_DEFAULT_RADIUS.value()))
        .selector(
            "& svg",
            sx().width(ICON_GLYPH_VAR.value())
                .height(ICON_GLYPH_VAR.value()),
        );

    // Chrome only: a static icon takes no hover response.
    Variant::ALL.iter().fold(base, |base, &variant| {
        let chrome = variant_chrome_sx(variant, &ICON_VARS);
        // A glyph inside a container keeps clear of its edges; a bare one fills the box.
        let chrome = match variant {
            // No border, so the bare glyph keeps the whole box.
            Variant::Standard => chrome.border_width("0").var(ICON_GLYPH_VAR, "100%"),
            _ => chrome.var(ICON_GLYPH_VAR, ICON_GLYPH_INSET),
        };
        base.when(variant.state_name(), chrome)
    })
});

/// An image as a stencil filled with `currentColor`, so it takes the icon's color.
/// The `-webkit-` twins are for Chrome < 120, e.g. Android WebView 113 (1152).
static ICON_MASK_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width(ICON_GLYPH_VAR.value())
        .height(ICON_GLYPH_VAR.value())
        .background("currentColor")
        .with("-webkit-mask", "center / contain no-repeat")
        .mask("center / contain no-repeat")
        // Forced colours reset the fill to `Canvas`; this keeps it the parent's system colour.
        .media(
            FORCED_COLORS,
            sx().forced_color_adjust("preserve-parent-color"),
        )
});

fn icon_mask_style(src: &str) -> String {
    let url = css_string(src);
    format!("-webkit-mask-image:url({url});mask-image:url({url});")
}

#[component]
fn IconMask(src: String) -> Element {
    use_box()
        .framework_sx(&ICON_MASK_SX)
        .style(Some(icon_mask_style(&src)))
        .prepare()
        .render(HtmlTag::Span, Vec::new(), rsx! {})
}

fn icon_variables(props: &IconProps, variant: Variant) -> Variables {
    let base = base_color("Icon", props.color.as_ref());
    let contrast = contrast_color(&base);
    let colors = variant_colors(variant, &base);

    variables()
        .with(ICON_COLOR_VAR, text_color(&base))
        .with(ICON_FILL_VAR, fill_color(&base))
        .with(
            ICON_CONTRAST_VAR,
            contrast
                .and_then(|c| c.resolve(None))
                .or_else(|| literal_contrast(&base)),
        )
        .with(ICON_CONTAINER_VAR, colors.container)
        .with(ICON_ON_CONTAINER_VAR, colors.on_container)
        .with(
            ICON_SIZE.override_var(),
            props.size.resolve(Some(ICON_SIZE)),
        )
        // A sized icon keeps its own box in a slot.
        .with(
            ICON_SLOT_PAD,
            props.size.as_ref().map(|_| "0px".to_string()),
        )
        .with(ICON_RADIUS_VAR, props.radius.resolve(Some(SizeCss::RADIUS)))
}

base_props! {
    pub struct IconProps {
        /// Element to render as; `span` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        #[props(default, into)]
        variant: Input<Variant>,
        /// Second stop and angle of `variant: "gradient"`, whose first stop is `color`:
        /// `("secondary", 45)` or a [`Gradient`]. Unset keys take the theme's.
        #[props(default, into)]
        gradient: Option<Gradient>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Unset, `theme.icon.size`.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Unset, `theme.icon.radius`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// An image URL drawn as the glyph, in the icon's color, instead of `children`.
        /// It is only a stencil: its own colors are ignored.
        #[props(default, into)]
        src: Option<String>,
        /// A glyph drawn as a [`Pictogram`], instead of `children`; `src` wins over it.
        #[props(default, into)]
        svg: Option<SvgData>,
        #[props(default)]
        children: Element,
    }
}

/// A sized, coloured box around an svg glyph; decorative unless given `aria_label`.
/// As a `button`, `a` or form control it is never `aria-hidden`: name it.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Icon;
/// # fn app() -> Element {
/// rsx! {
///     Icon { aria_label: "Verified", "✓" }
///     Icon { src: "/assets/logo.svg", color: "primary" }
///     Icon { svg: pictogram_icons_lucide::house::outlined }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/icon>
#[component]
pub fn Icon(props: IconProps) -> Element {
    let component = props.component.copied_or(HtmlTag::Span);
    let variant = props.variant.copied_or(use_theme().icon.variant);
    // Colour resolution is most of an icon's render, and its inputs rarely change.
    let style = use_cache(
        (
            variant,
            props.color.clone(),
            props.size.clone(),
            props.radius.clone(),
        ),
        |_| icon_variables(&props, variant).render(),
    );
    let gradient = use_gradient_style(
        props.gradient.as_ref(),
        props.color.as_ref(),
        variant == Variant::Gradient,
        false,
    );
    let style = match gradient {
        Some(gradient) => format!("{style}{gradient}"),
        None => style,
    };
    let style = Some(style).filter(|style| !style.is_empty());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .into();
    let named = names_itself(&props.attributes);
    // `aria-hidden` on a focusable control leaves a Tab stop nothing announces.
    let interactive = is_interactive(component);
    let hideable = !named && !interactive;
    use_name_warning(
        !interactive || named,
        "Icon: a focusable icon needs an `aria-label` or `aria-labelledby`, or it is announced without a name.",
    );
    let children = match (props.src, props.svg) {
        (Some(src), _) => match media_url(&src, "Icon") {
            Some(src) => rsx! { IconMask { src } },
            None => rsx! {},
        },
        (None, Some(icon)) => rsx! { Pictogram { icon } },
        (None, None) => props.children,
    };

    use_box()
        .framework_sx(&ICON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(style)
        .prepare()
        .attr_default("data-slot", Some("icon"))
        .attr_default("role", (named && !interactive).then_some("img"))
        .attr_default("aria-hidden", hideable.then_some("true"))
        .render(component, props.attributes, children)
}

fn is_interactive(component: HtmlTag) -> bool {
    matches!(
        component.as_str(),
        "button" | "a" | "input" | "select" | "textarea"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::interactive_variant_sx;
    use crate::tokens::{Color, ColorShade, ColorValue};

    fn icon_props(color: Input<ThemeAwareValue>) -> IconProps {
        IconProps {
            component: Input::None,
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            variant: Input::None,
            gradient: None,
            color,
            size: Input::None,
            radius: Input::None,
            src: None,
            svg: None,
            children: rsx! {},
        }
    }

    /// Todo 3556: a focusable control must not be `aria-hidden`.
    #[test]
    fn only_a_non_focusable_component_is_decorative() {
        assert!(is_interactive(HtmlTag::Button));
        assert!(is_interactive(HtmlTag::A));
        assert!(is_interactive(HtmlTag::Input));
        assert!(!is_interactive(HtmlTag::Span));
        assert!(!is_interactive(HtmlTag::Div));
    }

    /// A named `button` keeps its own role: `role=img` would announce it as an image.
    #[test]
    fn a_named_interactive_icon_keeps_its_role() {
        for tag in [HtmlTag::Button, HtmlTag::A] {
            let html = dioxus_ssr::render_element(rsx! {
                crate::LiberoProvider { Icon { component: tag, "aria-label": "Close", "x" } }
            });
            assert!(!html.contains("role="), "{html}");
            assert!(!html.contains("aria-hidden"), "{html}");
        }
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider { Icon { "aria-label": "Verified", "x" } }
        });
        assert!(html.contains(r#"role="img""#), "{html}");
    }

    #[test]
    fn an_unnamed_or_blank_labelled_focusable_icon_warns() {
        let warns = |element: fn() -> Element| {
            crate::utils::warnings_of(element)
                .iter()
                .any(|w| w.starts_with("Icon:"))
        };
        assert!(warns(|| rsx! {
            crate::LiberoProvider { Icon { component: HtmlTag::Button } }
        }));
        assert!(warns(|| rsx! {
            crate::LiberoProvider { Icon { component: HtmlTag::Button, "aria-label": "  " } }
        }));
        assert!(!warns(|| rsx! {
            crate::LiberoProvider { Icon { component: HtmlTag::Button, "aria-label": "Close" } }
        }));
        assert!(!warns(|| rsx! {
            crate::LiberoProvider { Icon { component: HtmlTag::A, "aria-labelledby": "x" } }
        }));
        assert!(!warns(|| rsx! { crate::LiberoProvider { Icon {} } }));
    }

    /// The outlined variant's border colour needs a border to draw on.
    #[test]
    fn the_icon_box_has_a_one_pixel_border_except_when_bare() {
        let css = crate::css::Stylesheet::from(&ICON_BASE_SX);
        let css = css.as_str();
        assert!(css.contains("border-style:solid;"), "{css}");
        assert!(css.contains("border-width:1px;"), "{css}");
        assert!(css.contains("border-width:0;"), "{css}");
    }

    /// 5065: the 60% share left a 14px glyph at md; below lg a 16px floor holds.
    #[test]
    fn a_contained_glyph_has_a_floor_and_a_cap() {
        assert_eq!(ICON_GLYPH_INSET, "min(max(60%, 16px), 80%)");
        let css = crate::css::Stylesheet::from(&ICON_BASE_SX);
        assert!(css.as_str().contains(ICON_GLYPH_INSET), "{}", css.as_str());
    }

    #[test]
    fn a_script_src_draws_no_mask_and_warns() {
        let render = || {
            rsx! {
                crate::LiberoProvider {
                    Icon { src: "javascript:alert(1)" }
                    Icon { src: "data:image/svg+xml;base64,AAAA" }
                }
            }
        };
        let warnings = crate::utils::warnings_of(render);
        assert!(
            warnings.iter().any(|w| w.starts_with("Icon:")),
            "{warnings:?}"
        );
        let html = dioxus_ssr::render_element(render());
        assert!(!html.contains("javascript"), "{html}");
        assert!(html.contains("data:image/svg+xml"), "{html}");
    }

    #[test]
    fn a_source_url_is_quoted_into_the_mask() {
        assert_eq!(
            icon_mask_style("/a b\"c.svg"),
            "-webkit-mask-image:url(\"/a b\\\"c.svg\");mask-image:url(\"/a b\\\"c.svg\");"
        );
    }

    /// Chrome < 120 masks only through the prefixed shorthand (1152).
    #[test]
    fn the_mask_sx_carries_a_webkit_twin() {
        let css = crate::css::Stylesheet::from(&ICON_MASK_SX);
        let css = css.as_str();
        assert!(
            css.contains("-webkit-mask:center / contain no-repeat;"),
            "{css}"
        );
        assert!(css.contains(";mask:center / contain no-repeat;"), "{css}");
    }

    /// Forced colours turned the `currentColor` fill into `Canvas`: the glyph vanished.
    #[test]
    fn a_masked_glyph_keeps_its_fill_under_forced_colours() {
        let css = crate::css::Stylesheet::from(&ICON_MASK_SX);
        let css = css.as_str();
        let forced = css
            .find("@media (forced-colors: active)")
            .unwrap_or_else(|| panic!("no forced-colours rule: {css}"));
        assert!(
            css[forced..].contains("forced-color-adjust:preserve-parent-color"),
            "{css}"
        );
    }

    #[test]
    fn a_bare_theme_color_becomes_a_shade_plus_its_contrast() {
        let variables =
            icon_variables(&icon_props(Color::Error.into()), Variant::Filled).to_string();

        assert!(variables.contains(&format!(
            "{}:{};",
            ICON_COLOR_VAR.name(),
            ColorValue::Text(Color::Error, ColorShade::S6).value()
        )));
        assert!(variables.contains(ICON_CONTRAST_VAR.name()));
    }

    /// A literal has no theme shade: its label is black or white, whichever reads (2105).
    #[test]
    fn a_literal_color_gets_the_readable_end() {
        let variables = icon_variables(&icon_props("#123456".into()), Variant::Filled).to_string();

        assert!(variables.contains(&format!("{}:#123456;", ICON_COLOR_VAR.name())));
        assert!(variables.contains(&format!("{}:#FFFFFF;", ICON_CONTRAST_VAR.name())));
    }

    /// Chrome only: a hover response would claim interactivity.
    #[test]
    fn a_badge_renders_variant_chrome_without_a_hover() {
        let class_of = |variant| variant_chrome_sx(variant, &ICON_VARS).class_name();

        let classes: Vec<String> = Variant::ALL.iter().map(|v| class_of(*v)).collect();
        let mut unique = classes.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), Variant::ALL.len());

        // Interactive chrome adds a `:hover`; a matching class would mean one grew.
        assert_ne!(
            variant_chrome_sx(Variant::Filled, &ICON_VARS).class_name(),
            interactive_variant_sx(
                Variant::Filled,
                &ICON_VARS,
                &ICON_COLOR_VAR,
                &ICON_COLOR_VAR,
                &ICON_COLOR_VAR
            )
            .class_name()
        );
    }
}
