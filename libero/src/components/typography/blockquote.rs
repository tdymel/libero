use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, ScaleOrCss, States, Variables, attr, base_props,
            literal_contrast, parts_enum, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ANCHOR_COLOR, AnchorDefaults, BLOCKQUOTE_BACKGROUND, BLOCKQUOTE_BORDER_COLOR,
        BLOCKQUOTE_CITE_OPACITY, BLOCKQUOTE_COLOR, BlockquoteDefaults, Color, ColorShade,
        ColorValue, FOCUS_RING_HALO, NamedColorCss, SURFACE_LABEL, Size, SizeCss, TEXT_FONT_SIZE,
    },
};

/// Light enough to stay a tint, so the quote reads over it, as on `Mark`.
const TINT_SHADE: ColorShade = ColorShade::S1;

const ACCENT_SHADE: ColorShade = ColorShade::S6;

/// Tint, accent bar and text colour, resolved together so they cannot disagree.
struct Palette {
    background: ThemeAwareValue,
    border: ThemeAwareValue,
    /// `None` for a value `literal_contrast` cannot read.
    contrast: Option<ThemeAwareValue>,
}

/// A bare colour takes the tint shade, painted as `fill-N`, which `contrast-N` is
/// computed on (todo 605). A literal is both bar and fill, so it shows neither (todo 2486).
fn palette(value: Option<&ThemeAwareValue>, default_color: Color) -> Palette {
    let from_shade = |color: Color, shade: ColorShade| Palette {
        background: ThemeAwareValue::ColorValue(ColorValue::Fill(color, shade)),
        border: ThemeAwareValue::ColorValue(ColorValue::Shade(color, ACCENT_SHADE)),
        contrast: Some(ThemeAwareValue::ColorValue(ColorValue::Contrast(
            color, shade,
        ))),
    };

    match value {
        None => from_shade(default_color, TINT_SHADE),
        Some(ThemeAwareValue::Color(color)) => from_shade(*color, TINT_SHADE),
        Some(ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade))) => {
            from_shade(*color, *shade)
        }
        // Black or white off a hex, `contrast-color()` off a name: under the page's text navy was 2.02:1.
        Some(other) => Palette {
            background: other.clone(),
            border: other.clone(),
            contrast: literal_contrast(other).map(ThemeAwareValue::String),
        },
    }
}

/// Publishes `--lsx-focus-contrast`: `sx` infers it only from a literal, and the tint is a `var()`.
/// The tint is the ring's halo (todo 630).
fn blockquote_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let palette = palette(color, default_color);
    let focus_contrast =
        crate::theme::CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string());
    let background = palette.background.resolve(None);
    let contrast = palette
        .contrast
        .as_ref()
        .and_then(|contrast| contrast.resolve(None));

    variables()
        .with(BLOCKQUOTE_BACKGROUND, background.clone())
        .with(BLOCKQUOTE_BORDER_COLOR, palette.border.resolve(None))
        .with(BLOCKQUOTE_COLOR, contrast.clone())
        .with(FOCUS_RING_HALO, contrast.as_ref().and(background))
        .with(focus_contrast, contrast.clone())
        // The link colour was 3.53:1 on the `info` tint (todo 2521), as on `Mark` (762).
        .with(ANCHOR_COLOR, contrast.clone())
        .with(SURFACE_LABEL, contrast)
}

/// The UA gives both `<figure>` and `<blockquote>` a 40px inline margin.
static FIGURE_SX: StaticSx = StaticSx::new(|| sx().margin("0"));

static BLOCKQUOTE_SX: StaticSx = StaticSx::new(|| {
    BlockquoteDefaults::theme_vars()
        .margin("0")
        .background(BLOCKQUOTE_BACKGROUND.value())
        // The tint's own contrast twin, never an accent shade.
        .color(BLOCKQUOTE_COLOR.value())
        // A long word overflowed a 320px column (WCAG 1.4.10), as on `Title`.
        .overflow_wrap("break-word")
        // A link in the text's colour needs its underline (1.4.1).
        .and(AnchorDefaults::underline_at_rest())
});

/// Size per step: a sibling of the quote, an `em` here would resolve against the `<figure>`.
/// Dims the ink, not the layer: `opacity` took a link in `attribution` to 2.4:1 (todo 2484).
static FIGCAPTION_SX: StaticSx = StaticSx::new(|| {
    sx().margin_top("0.5rem")
        .color(format!(
            "color-mix(in srgb, currentColor calc({} * 100%), transparent)",
            BLOCKQUOTE_CITE_OPACITY.value()
        ))
        // A link in the caption's colour needs its underline too (1.4.1).
        .and(AnchorDefaults::underline_at_rest())
        .per_size(|size| sx().font_size(format!("calc({} * 0.85)", TEXT_FONT_SIZE.value(size))))
});

parts_enum! {
    /// [`Blockquote`]'s inner parts, for its `parts` prop. The root is the `<figure>`, a
    /// `<div>` without a caption.
    pub enum BlockquotePart {
        /// The tinted `<blockquote>`.
        Quote = "quote" => "& > [data-slot='quote']",
        /// The `<figcaption>`, with `attribution` or `work`.
        Caption = "caption" => "& > [data-slot='caption']",
        /// The `<cite>` holding `work`.
        Work = "work" => "& > [data-slot='caption'] > [data-slot='work']",
    }
}

base_props! {
    parts(BlockquotePart);
    pub struct BlockquoteProps {
        /// Body font size, line height, padding and the accent bar's width.
        #[props(default, into)]
        size: Input<Size>,
        /// The accent bar, and the background tint derived from it. A literal colour, or a
        /// shade from 6 up, is a solid fill with no separate bar or tint.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Rounds the corners away from the bar: a size word or any CSS, as `radius: "0"`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Who said it, in a `<figcaption>` outside the quote. Plain text, not a `<cite>`.
        #[props(default)]
        attribution: Option<Element>,
        /// The title of the quoted work, as a `<cite>`. Not a person.
        #[props(default, into)]
        work: Option<String>,
        /// The `cite` attribute: the source's URL, machine-readable only.
        #[props(default, into)]
        cite_url: Option<String>,
        children: Element,
    }
}

/// A quotation with its attribution, as `<figure><blockquote/><figcaption/></figure>`;
/// without `attribution` or `work` the root is a `<div>`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Blockquote;
/// # fn app() -> Element {
/// # rsx! {
/// Blockquote {
///     color: "info",
///     attribution: rsx! { "Albert Einstein" },
///     work: "Letter to his son",
///     cite_url: "https://example.org/quotes/42",
///     "Life is like riding a bicycle. To keep your balance, you must keep moving."
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/blockquote>
#[component]
pub fn Blockquote(props: BlockquoteProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.blockquote.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.blockquote.radius);

    let quote_states: Input<States> = States::default()
        .with(size.state_name(), true)
        .with(radius.size.radius_state_name(), true)
        .into();
    let quote_variables: Input<Variables> =
        blockquote_variables(props.color.as_ref(), theme.blockquote.color)
            .with(
                SizeCss::RADIUS.override_var(),
                radius.custom_css(SizeCss::RADIUS),
            )
            .into();

    let quote = use_box()
        .framework_sx(&BLOCKQUOTE_SX)
        .states(&quote_states)
        .variables(&quote_variables)
        .prepare()
        .attr("data-slot", BlockquotePart::Quote.slot())
        .render(
            HtmlTag::Blockquote,
            props
                .cite_url
                .map(|url| vec![attr("cite", url)])
                .unwrap_or_default(),
            props.children,
        );

    let caption_states: Input<States> = States::default().with(size.state_name(), true).into();
    let caption_style = use_box()
        .framework_sx(&FIGCAPTION_SX)
        .states(&caption_states)
        .prepare();
    let cite_style = use_box().prepare();

    let caption = match (&props.attribution, &props.work) {
        (None, None) => None,
        (attribution, work) => {
            let work = work.as_ref().map(|work| {
                let work = work.clone();
                cite_style
                    .clone()
                    .attr("data-slot", BlockquotePart::Work.slot())
                    .render(HtmlTag::Cite, Vec::new(), rsx! { "{work}" })
            });
            // The spec's own example; the comma stays out of the `<cite>`.
            let separator = (attribution.is_some() && work.is_some()).then_some(", ");
            Some(
                caption_style
                    .attr("data-slot", BlockquotePart::Caption.slot())
                    .render(
                        HtmlTag::Figcaption,
                        Vec::new(),
                        rsx! {
                            {attribution.clone()}
                            {separator}
                            {work}
                        },
                    ),
            )
        }
    };

    // A `<figure>` with no caption is announced as a nameless "figure" (todo 2487).
    let root = match caption {
        Some(_) => HtmlTag::Figure,
        None => HtmlTag::Div,
    };
    let mut children = vec![quote];
    children.extend(caption);

    use_box()
        .framework_sx(&FIGURE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .prepare()
        .render(root, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<BlockquotePart>(),
            [
                ("quote", "& > [data-slot='quote']"),
                ("caption", "& > [data-slot='caption']"),
                ("work", "& > [data-slot='caption'] > [data-slot='work']"),
            ]
        );
    }

    /// Todo 2520: navy under the page text was about 1:1; the browser picks black or white.
    #[test]
    fn a_named_literal_takes_the_browsers_contrast_colour() {
        let navy = ThemeAwareValue::String("navy".to_string());
        let css = blockquote_variables(Some(&navy), Color::Primary).to_string();

        for pair in [
            format!("{}:contrast-color(navy);", BLOCKQUOTE_COLOR.name()),
            format!("{}:contrast-color(navy);", ANCHOR_COLOR.name()),
            format!("{}:navy;", FOCUS_RING_HALO.name()),
        ] {
            assert!(css.contains(&pair), "{pair} in {css}");
        }
    }

    /// Todo 2521: a link on the `info` tint was 3.53:1 in the theme's link colour.
    #[test]
    fn a_link_inside_takes_the_tints_twin() {
        let css = blockquote_variables(None, Color::Info).to_string();
        let twin = ColorValue::Contrast(Color::Info, TINT_SHADE).value();

        for var in [ANCHOR_COLOR, SURFACE_LABEL] {
            assert!(css.contains(&format!("{}:{twin};", var.name())), "{css}");
        }
    }
}
