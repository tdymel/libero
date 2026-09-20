use dioxus::prelude::*;

use super::highlight::{Language, highlight};
use super::token_theme::use_token_theme;
use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    platform::paints_outer_inline_backgrounds,
    sx::{StaticSx, sx},
    theme::CODE_FONT_FAMILY,
};

static CODE_INLINE_SX: StaticSx = StaticSx::new(|| {
    let base = match paints_outer_inline_backgrounds() {
        true => sx(),
        // Blitz fills a token's text from the token span alone.
        false => sx().selector("& > span", sx().background("inherit")),
    };
    base.display("inline")
        .background("muted.2")
        .border_radius("4px")
        .padding("2px 6px")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875em")
        // An identifier has no break point; unbroken it runs out of a 320px column (WCAG 1.4.10).
        .with("overflow-wrap", "anywhere")
        // `anywhere` lowers min-content, so shrink-to-fit parents split short spans (todo 732).
        .when("short", sx().white_space("nowrap"))
});

/// Up to this many characters a span stays on one line: ~170px, under a 320px column.
const SHORT_CODE_CHARS: usize = 20;

/// Splits the whitespace off each classed token into bare text (`None`): Blitz
/// trims the spaces an element starts or ends with, so `let width` read "letwidth".
fn spaces_outside_spans(line: &[Token]) -> Vec<Token> {
    let mut pieces = Vec::with_capacity(line.len());
    for (text, class) in line {
        let word = text.trim();
        if class.is_none() || word.is_empty() {
            pieces.push((text.clone(), None));
            continue;
        }
        let lead = text.len() - text.trim_start().len();
        let trail = lead + word.len();
        pieces.extend([
            (text[..lead].to_string(), None),
            (word.to_string(), *class),
            (text[trail..].to_string(), None),
        ]);
    }
    pieces.retain(|(text, _)| !text.is_empty());
    pieces
}

type Token = (String, Option<&'static str>);

base_props! {
    pub struct CodeProps {
        #[props(into)]
        source: String,
        /// Unset or unknown, no highlighting.
        #[props(default, into)]
        language: Input<Language>,
    }
}

/// Inline code, highlighted when `language` is set.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Code;
/// # fn app() -> Element {
/// rsx! {
///     Code { source: "let x = 1;", language: "rust" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/code>
#[component]
pub fn Code(props: CodeProps) -> Element {
    use_token_theme();

    let language = props.language.as_ref().copied();
    let source = props.source.clone();
    let highlighted = use_resource(use_reactive!(|source, language| async move {
        language.map(|language| highlight(&source, language))
    }));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("short", props.source.chars().count() <= SHORT_CODE_CHARS)
        .into();
    let boxed = use_box()
        .framework_sx(&CODE_INLINE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare();

    boxed.render(
        HtmlTag::Code,
        props.attributes,
        rsx! {
            if let Some(lines) = highlighted.read().clone().flatten() {
                for line in lines.iter() {
                    for (text, class) in spaces_outside_spans(line) {
                        if class.is_some() {
                            span { class, {text} }
                        } else {
                            {text}
                        }
                    }
                }
            } else {
                {props.source.as_str()}
            }
        },
    )
}
