use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, States, Variables, attr, base_props, parts_enum, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        BLOCKQUOTE_BACKGROUND, BLOCKQUOTE_BORDER_COLOR, BLOCKQUOTE_CITE_OPACITY, BLOCKQUOTE_COLOR,
        BlockquoteDefaults, Color, ColorShade, ColorValue, FOCUS_RING_HALO, NamedColorCss, Size,
        TEXT_FONT_SIZE,
    },
};

/// Light enough to stay a tint, so the quote reads over it, as on `Mark`.
const TINT_SHADE: ColorShade = ColorShade::S1;

const ACCENT_SHADE: ColorShade = ColorShade::S6;

/// Tint, accent bar and text colour, resolved together so they cannot disagree.
struct Palette {
    background: ThemeAwareValue,
    border: ThemeAwareValue,
    /// `None` for arbitrary CSS, which has no contrast twin.
    contrast: Option<ThemeAwareValue>,
}

/// A bare colour takes the tint shade, painted as `fill-N`, which `contrast-N` is
/// computed on (todo 605). Of the literals only a hex keeps a contrast twin.
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
        // Literal black or white off a hex: under the page's text a dark one was 2.02:1.
        Some(other) => Palette {
            background: other.clone(),
            border: other.clone(),
            contrast: match other {
                ThemeAwareValue::RawColor(_, hex) => {
                    Some(ThemeAwareValue::String(hex.contrast().to_string()))
                }
                _ => None,
            },
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
        .with(focus_contrast, contrast)
}

/// The UA gives both `<figure>` and `<blockquote>` a 40px inline margin.
static FIGURE_SX: StaticSx = StaticSx::new(|| sx().margin("0"));

static BLOCKQUOTE_SX: StaticSx = StaticSx::new(|| {
    BlockquoteDefaults::theme_vars()
        .margin("0")
        .background(BLOCKQUOTE_BACKGROUND.value())
        // The tint's own contrast twin, never an accent shade.
        .color(BLOCKQUOTE_COLOR.value())
});

/// Size per step: a sibling of the quote, an `em` here would resolve against the `<figure>`.
/// Dims the ink, not the layer: `opacity` took a link in `attribution` to 2.4:1 (todo 2484).
static FIGCAPTION_SX: StaticSx = StaticSx::new(|| {
    sx().margin_top("0.5rem")
        .color(format!(
            "color-mix(in srgb, currentColor calc({} * 100%), transparent)",
            BLOCKQUOTE_CITE_OPACITY.value()
        ))
        .per_size(|size| sx().font_size(format!("calc({} * 0.85)", TEXT_FONT_SIZE.value(size))))
});

parts_enum! {
    /// [`Blockquote`]'s inner parts, for its `parts` prop. The root is the `<figure>`.
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
        /// The accent bar, and the background tint derived from it. A CSS colour other
        /// than a hex leaves the quote text at the page's colour: check its contrast.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Rounds the two corners away from the accent bar.
        #[props(default, into)]
        radius: Input<Size>,
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

/// A quotation with its attribution, as `<figure><blockquote/><figcaption/></figure>`.
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
    let radius = props.radius.copied_or(theme.blockquote.radius);

    let quote_states: Input<States> = States::default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .into();
    let quote_variables: Input<Variables> =
        blockquote_variables(props.color.as_ref(), theme.blockquote.color).into();

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

    let mut children = vec![quote];
    children.extend(caption);

    use_box()
        .framework_sx(&FIGURE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Figure, props.attributes, children)
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
}
