use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, attr, base_props, variables},
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

/// Light enough to stay a tint rather than a fill, so the quote reads over it.
/// Which colour gets tinted is the caller's or the theme's; only the shade is
/// fixed here. `Mark` makes the same choice for the same reason.
const TINT_SHADE: ColorShade = ColorShade::S1;

/// The accent bar is the full-strength colour beside the tint.
const ACCENT_SHADE: ColorShade = ColorShade::S6;

/// What the caller's `color` resolves to: a tint to sit on, an accent bar, and
/// the text colour that reads against the tint.
///
/// Returned together rather than computed at three call sites, because the
/// three have to agree - a tint from one shade with contrast text for another
/// is exactly the unreadable combination `component-building-rules` warns
/// about.
struct Palette {
    background: ThemeAwareValue,
    border: ThemeAwareValue,
    /// `None` when the caller passed arbitrary CSS, from which no contrast
    /// twin can be derived. See [`blockquote_variables`].
    contrast: Option<ThemeAwareValue>,
}

/// A bare colour name carries no shade, so it takes the tint shade rather than
/// the `sx` pipeline's generic default. A palette tint paints in the fill role,
/// `fill-N`, the colour its `contrast-N` twin is computed on (todo 605). A
/// literal passes through; only a hex keeps a contrast twin, read off its own
/// value.
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

/// Four variables, one resolution, so they cannot disagree.
///
/// `--lsx-focus-contrast` is the one that is easy to forget and invisible when
/// missing. `sx`'s `background()` publishes it only when it recognises a
/// literal colour, and ours is a `var()`, which is opaque to that inference -
/// so a link inside a quote would draw its focus ring from the fallback,
/// `primary.6`, against a `primary.1` tint. `Paper` owes the same debt for the
/// same reason. The tint is the ring's halo beside it (todo 630).
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

/// Layout only. The frame lives on the `<blockquote>`; this exists to tie the
/// quote to its attribution.
///
/// The UA stylesheet gives **both** `<figure>` and `<blockquote>` a 40px
/// inline margin, so both reset it.
static FIGURE_SX: StaticSx = StaticSx::new(|| sx().margin("0"));

static BLOCKQUOTE_SX: StaticSx = StaticSx::new(|| {
    BlockquoteDefaults::theme_vars()
        .margin("0")
        .background(BLOCKQUOTE_BACKGROUND.value())
        // The tint's own contrast twin, never an accent shade - the rule three
        // components had to learn the hard way.
        .color(BLOCKQUOTE_COLOR.value())
});

/// Sits outside the quote, which is what makes the attribution attribution
/// rather than quoted words.
///
/// Being a sibling is also why the size is spelled out per step: an `em` here
/// resolves against the `<figure>`, which never sees the quote's font size, so
/// `0.85em` sat at 13.6px under every size of quote.
static FIGCAPTION_SX: StaticSx = StaticSx::new(|| {
    sx().margin_top("0.5rem")
        .opacity(BLOCKQUOTE_CITE_OPACITY.value())
        .per_size(|size| sx().font_size(format!("calc({} * 0.85)", TEXT_FONT_SIZE.value(size))))
});

base_props! {
    pub struct BlockquoteProps {
        /// Body font size, line height, padding and the accent bar's width.
        #[props(default, into)]
        size: Input<Size>,
        /// The accent bar, and the background tint derived from it.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Rounds the two corners away from the accent bar.
        #[props(default, into)]
        radius: Input<Size>,
        /// Who said it, rendered in a `<figcaption>` **outside** the quote.
        ///
        /// A person's name goes here as plain text. It deliberately does not
        /// become a `<cite>`: the HTML spec reserves that element for the
        /// title of a work and says it "must therefore not be used to mark up
        /// people's names".
        #[props(default)]
        attribution: Option<Element>,
        /// The title of the work quoted, rendered as a `<cite>` in the
        /// `<figcaption>` - a book, a talk, a film. **Not** a person.
        /// Follows `attribution` after a comma when both are set.
        #[props(default, into)]
        work: Option<String>,
        /// The `cite` **attribute** on `<blockquote>`: a URL naming the source
        /// document. Machine-readable only - no browser renders it. Unrelated
        /// to `work`, which is the visible `<cite>` element.
        #[props(default, into)]
        cite_url: Option<String>,
        children: Element,
    }
}

/// A quotation with its attribution.
///
/// Renders `<figure><blockquote/><figcaption/></figure>` - the shape the HTML
/// spec calls correct, where attribution sits *outside* the quote. Nesting the
/// attribution inside the `<blockquote>`, as most libraries do, makes assistive
/// technology and quote-extraction tools read it as quoted material.
///
/// Not interactive: no role, no `tabindex`, no ARIA. `<blockquote>` and
/// `<cite>` mean what they mean.
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
        .render(
            HtmlTag::Blockquote,
            // The one place `cite_url` appears: an attribute, not an element.
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

    // Both halves are optional and independent: a named speaker with no work,
    // a work with no speaker, or neither.
    let caption = match (&props.attribution, &props.work) {
        (None, None) => None,
        (attribution, work) => {
            let work = work.as_ref().map(|work| {
                let work = work.clone();
                cite_style
                    .clone()
                    .render(HtmlTag::Cite, Vec::new(), rsx! { "{work}" })
            });
            // Two adjacent inline nodes have nothing between them. The comma
            // is the spec's own worked example, and it belongs to neither
            // half - so it stays out of the `<cite>`.
            let separator = (attribution.is_some() && work.is_some()).then_some(", ");
            Some(caption_style.render(
                HtmlTag::Figcaption,
                Vec::new(),
                rsx! {
                    {attribution.clone()}
                    {separator}
                    {work}
                },
            ))
        }
    };

    let mut children = vec![quote];
    children.extend(caption);

    use_box()
        .framework_sx(&FIGURE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Figure, props.attributes, children)
}
