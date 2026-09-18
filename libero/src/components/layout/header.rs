use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_props, fill_color, input_from_str, variables,
        },
        layout::use_box,
    },
    css::Stylesheet,
    hooks::{use_css, use_id},
    platform::{document, when_laid_out},
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ColorShade, ColorValue, CssVar, FOCUS_RING_HALO, HEADER_HEIGHT, HEADER_HEIGHT_VAR,
        NamedColorCss, PAPER_BACKGROUND, Size, Z_INDEX_HEADER,
    },
};

str_enum! {
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

// Unlike `Icon`/`Button`, an unset `color` keeps the neutral default rather
// than falling back to a theme color. A bare color name takes the shade
// above; everything else passes through.
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

/// This Header's height and the scroll padding on `:root`, live while the root
/// names it. Focus moved under a stuck banner is then scrolled clear (2.4.11).
fn publish_css(id: &str, height: &str) -> String {
    format!(
        ":root[{PUBLISHER_ATTRIBUTE}=\"{id}\"]{{{}:{height};scroll-padding-top:{};}}",
        HEADER_HEIGHT_VAR.name(),
        HEADER_HEIGHT_VAR.value()
    )
}

static HEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .width("100%")
        .with(
            HEADER_HEIGHT_VAR.name(),
            HEADER_HEIGHT.overridable(Size::Md),
        )
        // Content that wraps (200% text, 320 px) grows the banner instead of spilling.
        .min_height(HEADER_HEIGHT_VAR.value())
        .padding_left("md")
        .padding_right("md")
        .background(HEADER_BACKGROUND_VAR.value_or(PAPER_BACKGROUND.value()))
        .color(HEADER_COLOR_VAR.value_or("inherit"))
        .border_bottom("1px solid")
        .border_bottom_color("muted.4")
        .z_index(Z_INDEX_HEADER.overridable())
        .position("sticky")
        .top("0")
        .when("static", sx().position("static"))
        .when("fixed", sx().position("fixed").top("0"))
});

fn header_variables(props: &HeaderProps) -> Variables {
    let base = header_base_color(props.color.as_ref());
    let contrast = base
        .as_ref()
        .and_then(header_contrast_color)
        .and_then(|v| v.resolve(None));
    let fill = base.as_ref().and_then(fill_color);

    variables()
        .with(
            HEADER_HEIGHT.override_var(),
            props.size.resolve(Some(HEADER_HEIGHT)),
        )
        // The banner is a fill under `HEADER_COLOR_VAR`, so it resolves
        // through the fill ramp: a `primary` header used to be `blue.6` with
        // white text on it, 3.56:1 (todo 239).
        .with(HEADER_BACKGROUND_VAR, fill.clone())
        .with(HEADER_COLOR_VAR, contrast.clone())
        // The background comes through a var, so `sx` cannot publish the
        // focus contrast from it (`codebase/sx`): without this every ring in a
        // coloured header is the primary shade on a primary banner. Only with
        // a `color`, so an uncoloured header inherits the page's.
        .with(FOCUS_RING_HALO, contrast.as_ref().and(fill))
        .with(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            contrast,
        )
        .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
}

base_props! {
    pub struct HeaderProps {
        /// `Sticky` (default) needs no offset; `Fixed` is viewport-relative:
        /// offset your content by `var(--lsx-header-height)` (see `publish_height`).
        #[props(default, into)]
        position: Input<HeaderPosition>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Publishes this sticky/fixed Header's height as `--lsx-header-height`
        /// and `scroll-padding-top` on `:root`, so focus scrolls clear of it.
        /// Set it on the page's own banner only; the last one mounted wins.
        #[props(default)]
        publish_height: bool,
        children: Element,
    }
}

/// The page's `banner` landmark, always a `<header>`. Hosts nav and actions
/// as children rather than being scoped to either.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Header;
/// # fn app() -> Element {
/// rsx! {
///     // The page's banner: focus moved under it is scrolled clear.
///     Header { publish_height: true, "Libero" }
/// }
/// # }
/// ```
#[component]
pub fn Header(props: HeaderProps) -> Element {
    let position = props.position.copied_or_default();
    let variables: Input<Variables> = header_variables(&props).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("static", position == HeaderPosition::Static)
        .with("fixed", position == HeaderPosition::Fixed)
        .into();

    // An opted-in sticky or fixed Header publishes its height on `:root`, the
    // last one mounted winning. The size's own CSS, so nothing is measured.
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
        .unwrap_or_else(|| HEADER_HEIGHT.value(Size::Md));
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
            children: rsx! {},
        }
    }

    #[test]
    fn a_theme_color_brings_its_own_contrast_along() {
        let variables = header_variables(&header_props(Color::Primary.into())).to_string();

        assert!(variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(variables.contains(HEADER_COLOR_VAR.name()));
        assert!(variables.contains(NamedColorCss::FOCUS_CONTRAST.name()));
        assert!(variables.contains(&format!(
            "{}:{};",
            FOCUS_RING_HALO.name(),
            ColorValue::Fill(Color::Primary, HEADER_DEFAULT_SHADE).value()
        )));
    }

    #[test]
    fn a_publisher_sets_its_height_and_scroll_padding_on_the_root() {
        assert_eq!(
            publish_css("lsx-7", "var(--lsx-header-height-lg)"),
            ":root[data-lsx-header=\"lsx-7\"]{--lsx-header-height:var(--lsx-header-height-lg);\
             scroll-padding-top:var(--lsx-header-height);}"
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

    /// Unset means the themed default applies, so neither var is pinned.
    #[test]
    fn no_color_emits_neither_variable() {
        let variables = header_variables(&header_props(Input::None)).to_string();

        assert!(!variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(!variables.contains(HEADER_COLOR_VAR.name()));
        assert!(!variables.contains(NamedColorCss::FOCUS_CONTRAST.name()));
        assert!(!variables.contains(FOCUS_RING_HALO.name()));
    }
}
