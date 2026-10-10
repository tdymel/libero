use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;

use super::paper::{PAPER_TINT_SHARE, glass_tint_color};
use crate::{
    CssLayer,
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_props, fill_color, input_from_str,
            literal_contrast, safe_area_padding, variables,
        },
        layout::use_box,
    },
    css::Stylesheet,
    hooks::{use_css, use_glass_gradient_style, use_glass_tint, use_id, use_theme},
    platform::{SCROLL_PADDING_VARS, document, draws_backdrop_filter, when_laid_out},
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ANCHOR_COLOR, AnchorDefaults, ColorShade, ColorValue, CssVar, FOCUS_RING_HALO, GLASS_SHEEN,
        GRADIENT_CONTRAST, GlassTint, Gradient, HEADER_DEFAULT_HEIGHT, HEADER_HEIGHT,
        HEADER_HEIGHT_VAR, NamedColorCss, PAPER_BACKGROUND, PaperDefaults, SURFACE_LABEL, Size,
        SizeCss, Z_INDEX_HEADER, gradient_surface_sx,
    },
};

str_enum! {
    /// How a [`Header`] stays on screen.
    pub enum HeaderPosition {
        Static = "static",
        #[default]
        Sticky = "sticky",
        Fixed = "fixed",
    }
}

input_from_str!(HeaderPosition);

// Matches Button's shade: bold enough for a solid brand-color banner.
const HEADER_DEFAULT_SHADE: ColorShade = ColorShade::S6;

// Unset keeps the neutral default, unlike `Icon`/`Button`; a bare color takes the shade above.
fn header_base_color(value: Option<&ThemeAwareValue>) -> Option<ThemeAwareValue> {
    match value {
        None => None,
        Some(ThemeAwareValue::Color(color)) => Some(ThemeAwareValue::ColorValue(
            ColorValue::Shade(*color, HEADER_DEFAULT_SHADE),
        )),
        Some(other) => Some(other.clone()),
    }
}

// Only a resolved theme shade has a precomputed contrast var.
fn header_contrast_color(base: &ThemeAwareValue) -> Option<ThemeAwareValue> {
    match base {
        ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) => Some(
            ThemeAwareValue::ColorValue(ColorValue::Contrast(*color, *shade)),
        ),
        _ => None,
    }
}

const HEADER_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-header-background");
const HEADER_COLOR_VAR: CssVar = CssVar::new("--lsx-header-color");

/// The root attribute naming the Header that publishes its height.
const PUBLISHER_ATTRIBUTE: &str = "data-lsx-header";

thread_local! {
    /// Mounted sticky/fixed Headers, oldest first: the last one publishes, and
    /// unmounting it hands the root back to the one before.
    static PUBLISHERS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn set_publishing(id: &str, publishes: bool) {
    let top = PUBLISHERS.with_borrow_mut(|stack| {
        stack.retain(|other| other != id);
        if publishes {
            stack.push(id.to_string());
        }
        stack.last().cloned()
    });
    if let Some(document) = document() {
        document.set_root_attribute(PUBLISHER_ATTRIBUTE, top.as_deref());
    }
}

/// One Header's place on the root: whether it publishes, and should.
struct Published {
    id: String,
    on: Cell<bool>,
    wanted: Cell<bool>,
}

/// No document (SSR, headless): nothing to publish on, and nothing to leak.
/// Natively there is none until the provider's outlet mounts, so it retries then.
fn publish(published: &Rc<Published>, retry: bool) {
    let wanted = published.wanted.get();
    if published.on.get() == wanted {
        return;
    }
    if document().is_some() {
        published.on.set(wanted);
        set_publishing(&published.id, wanted);
    } else if retry {
        // Weak: an unmounted Header must not publish.
        let published = Rc::downgrade(published);
        when_laid_out(move || {
            if let Some(published) = published.upgrade() {
                publish(&published, false);
            }
        });
    }
}

/// This Header's height, with the top safe area it pads, and the scroll padding on
/// `:root`, live while the root names it. Focus moved under a stuck banner is then scrolled clear (2.4.11), natively too.
fn publish_css(id: &str, height: &str) -> String {
    let padding = HEADER_HEIGHT_VAR.value();
    format!(
        ":root[{PUBLISHER_ATTRIBUTE}=\"{id}\"]{{{}:{};scroll-padding-top:{padding};{}:{padding};}}",
        HEADER_HEIGHT_VAR.name(),
        safe_area_padding(height, "top"),
        SCROLL_PADDING_VARS[0],
    )
}

static HEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    let spacing = SizeCss::SPACING.value(Size::Md);
    sx().display("flex")
        .align_items("center")
        .width("100%")
        .with(
            HEADER_HEIGHT_VAR.name(),
            HEADER_HEIGHT
                .override_var()
                .value_or(HEADER_DEFAULT_HEIGHT.value()),
        )
        // Content that wraps (200% text, 320 px) grows the banner instead of spilling.
        .min_height(safe_area_padding(&HEADER_HEIGHT_VAR.value(), "top"))
        // Physical, as `env()` is: clear of a phone's status bar and a landscape notch.
        .with("padding-top", safe_area_padding("0px", "top"))
        .with("padding-left", safe_area_padding(&spacing, "left"))
        .with("padding-right", safe_area_padding(&spacing, "right"))
        .background(HEADER_BACKGROUND_VAR.value_or(PAPER_BACKGROUND.value()))
        .color(HEADER_COLOR_VAR.value_or("inherit"))
        // Uncoloured, the surface: its buttons keep their own colour (todo 1663).
        .var(SURFACE_LABEL, "initial")
        .border_bottom("1px solid")
        .border_bottom_color("muted.4")
        .z_index(Z_INDEX_HEADER.overridable())
        .position("sticky")
        .top("0")
        // A banner in the flow need not sit at the top of the screen.
        .when(
            "static",
            sx().position("static")
                .padding_top("0")
                .min_height(HEADER_HEIGHT_VAR.value()),
        )
        // `left` too: `auto` keeps a padded parent's offset and runs past the viewport (todo 2392).
        .when("fixed", sx().position("fixed").top("0").left("0"))
        // Links take the fill's text colour there (todo 1576), so the underline marks them.
        .when("colored", AnchorDefaults::underline_at_rest())
        .when(
            "gradient",
            gradient_surface_sx()
                .var(ANCHOR_COLOR, GRADIENT_CONTRAST.value())
                .and(AnchorDefaults::underline_at_rest()),
        )
        // After `gradient`, which its shorthand would otherwise reset.
        .when(
            "glass",
            PaperDefaults::glass_sx()
                .when(
                    "colored",
                    PaperDefaults::glass_fill_sx(
                        &HEADER_BACKGROUND_VAR.value(),
                        Some(&PAPER_TINT_SHARE),
                    ),
                )
                .when("gradient", PaperDefaults::glass_gradient_sx()),
        )
});

/// `tint`: the label and share of a glass tint, which replace the solid fill's label.
fn header_variables(props: &HeaderProps, tint: Option<GlassTint>) -> Variables {
    // A gradient paints its own fill (it starts at `color`), with its own label.
    let color = props.color.as_ref().filter(|_| props.gradient.is_none());
    let base = header_base_color(color);
    let (contrast, share, sheen) = match tint {
        Some(tint) => (
            Some(tint.label),
            Some(format!("{}%", tint.share)),
            Some(format!("{}%", tint.sheen)),
        ),
        None => (
            base.as_ref()
                .and_then(header_contrast_color)
                .and_then(|v| v.resolve(None))
                .or_else(|| base.as_ref().and_then(literal_contrast)),
            None,
            None,
        ),
    };
    let fill = base.as_ref().and_then(fill_color);

    variables()
        .with(PAPER_TINT_SHARE, share)
        .with(GLASS_SHEEN, sheen)
        .with(
            HEADER_HEIGHT.override_var(),
            props.size.resolve(Some(HEADER_HEIGHT)),
        )
        // Through the fill ramp: `blue.6` under white text was 3.56:1 (todo 239).
        .with(HEADER_BACKGROUND_VAR, fill.clone())
        .with(HEADER_COLOR_VAR, contrast.clone())
        // A var background, so `sx` can't publish the focus contrast; only with a
        // `color`, so an uncoloured header inherits the page's.
        .with(FOCUS_RING_HALO, contrast.as_ref().and(fill))
        // The page's link colour reads on no fill: about 1.1:1 on `primary` (todo 1576).
        .with(ANCHOR_COLOR, contrast.clone())
        .with(SURFACE_LABEL, contrast.clone())
        .with(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            contrast,
        )
        .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
}

base_props! {
    pub struct HeaderProps {
        /// `Sticky` (default); `Fixed` content is offset by `var(--lsx-header-height)`.
        #[props(default, into)]
        position: Input<HeaderPosition>,
        /// Minimum height, a size or any CSS length. Unset, `theme.header.size`.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Banner fill; a bare color takes shade 6. The first stop under a `gradient`.
        /// A literal CSS colour is used as given, its text, link and focus colours black or white.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Publishes the height on `:root`, so focus scrolls clear. The page's banner only.
        #[props(default)]
        publish_height: bool,
        /// Frosted glass, as on `Paper`: a `color` tints it.
        #[props(default)]
        glass: bool,
        /// A linear gradient fill from `color`, as on `Paper`: `("secondary", 45)` or a [`Gradient`].
        #[props(default, into)]
        gradient: Option<Gradient>,
        children: Element,
    }
}

/// The page's `banner` landmark, a sticky `<header>` for nav and actions. It pads the
/// left and right safe-area insets, and unless `Static` the top, for a `viewport-fit=cover` page.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Header;
/// # fn app() -> Element {
/// rsx! {
///     Header { publish_height: true, "Libero" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/header>
#[component]
pub fn Header(props: HeaderProps) -> Element {
    let theme = use_theme();
    let position = props.position.copied_or_default();
    let glass = props.glass && draws_backdrop_filter();
    let tint = use_glass_tint(
        glass_tint_color(props.color.as_ref(), props.glass, props.gradient.is_some()).as_ref(),
    );
    let variables: Input<Variables> = header_variables(&props, tint).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("static", position == HeaderPosition::Static)
        .with("fixed", position == HeaderPosition::Fixed)
        .with("glass", glass)
        .with(
            "colored",
            props.color.as_ref().is_some() && props.gradient.is_none(),
        )
        .with("gradient", props.gradient.is_some())
        .into();
    let gradient = use_glass_gradient_style(
        props.gradient.as_ref(),
        props.color.as_ref(),
        props.gradient.is_some(),
        glass,
    );

    // The last mounted publisher wins; the size's own CSS, so nothing is measured.
    let id = use_id();
    let published = use_hook(|| {
        Rc::new(Published {
            id: id.peek().clone(),
            on: Cell::new(false),
            wanted: Cell::new(false),
        })
    });
    let publishes = props.publish_height && position != HeaderPosition::Static;
    let height = props
        .size
        .resolve(Some(HEADER_HEIGHT))
        .unwrap_or_else(|| HEADER_HEIGHT.value(theme.header.size));
    use_css(
        publishes.then(|| Stylesheet::from(publish_css(&published.id, &height).as_str())),
        CssLayer::Framework,
    );
    published.wanted.set(publishes);
    publish(&published, true);
    use_drop({
        let published = published.clone();
        move || {
            if published.on.get() {
                set_publishing(&published.id, false);
            }
        }
    });

    use_box()
        .framework_sx(&HEADER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .style(gradient)
        .prepare()
        .render(HtmlTag::Header, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Color;

    fn header_props(color: Input<ThemeAwareValue>) -> HeaderProps {
        HeaderProps {
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            position: Input::None,
            size: Input::None,
            color,
            z_index: Input::None,
            publish_height: false,
            glass: false,
            gradient: None,
            children: rsx! {},
        }
    }

    #[test]
    fn a_theme_color_brings_its_own_contrast_along() {
        let variables = header_variables(&header_props(Color::Primary.into()), None).to_string();

        assert!(variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(variables.contains(HEADER_COLOR_VAR.name()));
        assert!(variables.contains(NamedColorCss::FOCUS_CONTRAST.name()));
        assert!(variables.contains(&format!(
            "{}:{};",
            FOCUS_RING_HALO.name(),
            ColorValue::Fill(Color::Primary, HEADER_DEFAULT_SHADE).value()
        )));
        // A link takes the label too: the page's link colour is about 1.1:1 on the fill.
        assert!(variables.contains(&format!(
            "{}:{};",
            ANCHOR_COLOR.name(),
            ColorValue::Contrast(Color::Primary, HEADER_DEFAULT_SHADE).value()
        )));
        // So does an uncoloured standard Button (todo 1663).
        assert!(variables.contains(&format!(
            "{}:{};",
            SURFACE_LABEL.name(),
            ColorValue::Contrast(Color::Primary, HEADER_DEFAULT_SHADE).value()
        )));
    }

    #[test]
    fn a_publisher_sets_its_height_and_scroll_padding_on_the_root() {
        assert_eq!(
            publish_css("lsx-7", "var(--lsx-header-height-lg)"),
            ":root[data-lsx-header=\"lsx-7\"]{--lsx-header-height:\
             calc(var(--lsx-header-height-lg) + env(safe-area-inset-top, 0px));\
             scroll-padding-top:var(--lsx-header-height);\
             --lsx-scroll-padding-top:var(--lsx-header-height);}"
        );
    }

    /// Todo 1972: the browser harness cannot emulate a notch, so the rule itself.
    #[test]
    fn a_stuck_banner_pads_the_safe_areas() {
        let css = Stylesheet::from(&HEADER_BASE_SX);
        let css = css.as_str();
        for side in ["left", "right"] {
            assert!(
                css.contains(&format!(
                    "padding-{side}:calc({} + env(safe-area-inset-{side}, 0px))",
                    SizeCss::SPACING.value(Size::Md)
                )),
                "{css}"
            );
        }
        let (base, stat) = css
            .split_once(r#"[data-state~="static"]"#)
            .unwrap_or_else(|| panic!("no static rule: {css}"));
        assert!(
            base.contains(
                "min-height:calc(var(--lsx-header-height) + env(safe-area-inset-top, 0px));"
            ),
            "{css}"
        );
        assert!(
            base.contains("padding-top:calc(0px + env(safe-area-inset-top, 0px));"),
            "{css}"
        );
        // A banner in the flow need not sit at the top of the screen.
        let stat = &stat[..stat.find('}').unwrap()];
        assert!(stat.contains("padding-top:0;"), "{css}");
        assert!(
            stat.contains("min-height:var(--lsx-header-height);"),
            "{css}"
        );
    }

    #[test]
    fn the_last_publisher_wins_and_unmounting_restores_the_one_before() {
        set_publishing("a", true);
        set_publishing("b", true);
        assert_eq!(
            PUBLISHERS.with_borrow(|s| s.last().cloned()).as_deref(),
            Some("b")
        );
        set_publishing("b", false);
        assert_eq!(
            PUBLISHERS.with_borrow(|s| s.last().cloned()).as_deref(),
            Some("a")
        );
        set_publishing("a", false);
        assert!(PUBLISHERS.with_borrow(Vec::is_empty));
    }

    #[test]
    fn glass_tints_with_the_color_and_takes_the_tint_label_and_share() {
        let props = HeaderProps {
            glass: true,
            ..header_props(Color::Primary.into())
        };
        let variables = header_variables(
            &props,
            Some(GlassTint {
                label: "#000000".into(),
                share: 90,
                sheen: 18,
            }),
        )
        .to_string();

        assert!(
            variables.contains(HEADER_BACKGROUND_VAR.name()),
            "{variables}"
        );
        assert!(
            variables.contains(&format!("{}:#000000;", HEADER_COLOR_VAR.name())),
            "{variables}"
        );
        assert!(
            variables.contains("--lsx-paper-tint-share:90%;"),
            "{variables}"
        );
    }

    /// The glass fill mixes the header's own fill, cues included.
    #[test]
    fn a_coloured_glass_mixes_the_header_fill() {
        let css = crate::css::Stylesheet::from(&*HEADER_BASE_SX);
        let css = css.as_str();

        assert!(
            css.contains(
                "color-mix(in srgb, var(--lsx-header-background) var(--lsx-paper-tint-share"
            ),
            "{css}"
        );
        assert!(css.contains("saturate(160%)"), "{css}");
    }

    /// Todo 2412: a literal fills and gets the black or white label that reads on it.
    #[test]
    fn a_literal_color_fills_and_takes_a_readable_label() {
        let variables = header_variables(&header_props("#1a1a2e".into()), None).to_string();

        assert!(
            variables.contains(&format!("{}:#1a1a2e;", HEADER_BACKGROUND_VAR.name())),
            "{variables}"
        );
        for var in [HEADER_COLOR_VAR.name(), ANCHOR_COLOR.name()] {
            assert!(
                variables.contains(&format!("{var}:#FFFFFF;")),
                "{var}: {variables}"
            );
        }
    }

    #[test]
    fn a_fixed_banner_pins_its_left_edge() {
        let css = Stylesheet::from(&HEADER_BASE_SX);
        let css = css.as_str();
        let (_, fixed) = css
            .split_once(r#"[data-state~="fixed"]"#)
            .unwrap_or_else(|| panic!("no fixed rule: {css}"));
        let fixed = &fixed[..fixed.find('}').unwrap()];
        assert!(fixed.contains("top:0;"), "{css}");
        assert!(fixed.contains("left:0;"), "{css}");
    }

    /// The colour is the gradient's first stop, not a flat fill under it.
    #[test]
    fn a_gradient_takes_the_color_as_its_first_stop() {
        let props = HeaderProps {
            gradient: Some(Gradient::default()),
            ..header_props(Color::Primary.into())
        };
        let variables = header_variables(&props, None).to_string();

        assert!(!variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(!variables.contains(HEADER_COLOR_VAR.name()));
    }

    /// Glass mixes the stops down after the fill; a ring inside reads the label.
    #[test]
    fn a_gradient_combines_with_glass() {
        let css = crate::css::Stylesheet::from(&*HEADER_BASE_SX);
        let css = css.as_str();

        assert!(
            css.contains("--lsx-focus-contrast:var(--lsx-gradient-contrast);"),
            "{css}"
        );
        assert!(
            css.contains("--lsx-anchor-color:var(--lsx-gradient-contrast);"),
            "{css}"
        );
        assert!(
            css.contains("color-mix(in srgb, var(--lsx-gradient-from)"),
            "{css}"
        );
    }

    /// Unset means the themed default applies, so neither var is pinned.
    #[test]
    fn no_color_emits_neither_variable() {
        let variables = header_variables(&header_props(Input::None), None).to_string();

        assert!(!variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(!variables.contains(HEADER_COLOR_VAR.name()));
        assert!(!variables.contains(NamedColorCss::FOCUS_CONTRAST.name()));
        assert!(!variables.contains(FOCUS_RING_HALO.name()));
        assert!(!variables.contains(ANCHOR_COLOR.name()));
        assert!(!variables.contains(SURFACE_LABEL.name()));
    }
}
