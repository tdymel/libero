use dioxus::prelude::*;

use super::highlight::{Language, highlight};
use super::token_theme::use_token_theme;
use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    sx::{StaticSx, sx},
    theme::CODE_FONT_FAMILY,
};

static CODE_INLINE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline")
        .background("muted.2")
        .border_radius("4px")
        .padding("2px 6px")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875em")
        // An identifier has no break point; unbroken it runs out of a 320px column (WCAG 1.4.10).
        .with("overflow-wrap", "anywhere")
        // `anywhere` also lowers min-content, so a shrink-to-fit parent split
        // `#[derive(Options)]` after the `#` (todo 732). A short span fits 320px whole.
        .when("short", sx().white_space("nowrap"))
});

/// Up to this many characters a span stays on one line: about 170px of
/// monospace at 14px, under a 320px column's width.
const SHORT_CODE_CHARS: usize = 20;

base_props! {
    pub struct CodeProps {
        /// The text to render, highlighted when `language` names a grammar
        /// this build compiles in.
        #[props(into)]
        source: String,
        /// Unrecognized values fall back to no highlighting rather than a guess.
        #[props(default, into)]
        language: Input<Language>,
    }
}

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
                    for (text, class) in line.iter() {
                        span { class: *class, {text.as_str()} }
                    }
                }
            } else {
                {props.source.as_str()}
            }
        },
    )
}
