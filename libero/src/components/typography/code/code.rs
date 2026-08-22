use dioxus::prelude::*;

use super::highlight::{Language, highlight};
use super::token_theme::use_token_theme;
use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    sx::{StaticSx, Sx, sx},
    theme::CODE_FONT_FAMILY,
};

static CODE_INLINE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline")
        .background("grey.2")
        .border_radius("4px")
        .padding("2px 6px")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875em")
});

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

    let boxed = use_box()
        .framework_sx(&CODE_INLINE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
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
