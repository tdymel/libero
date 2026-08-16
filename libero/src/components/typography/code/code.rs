use dioxus::prelude::*;

use super::highlight::{HighlightedLine, Language, highlight_lazy, plain_lines};
use super::token_theme::use_token_theme;
use crate::{
    components::{Box, Input, States},
    hooks::{Clipboard, use_clipboard},
    sx::{StaticSx, Sx, sx},
    theme::CODE_FONT_FAMILY,
};

// GitHub's own code-block palette reads much closer to white than this
// theme's `grey.1`/`grey.3` - close enough to matter for something meant to
// look like a code editor, so these are hardcoded rather than theme tokens.
const CODE_BG: &str = "#f6f8fa";
const CODE_BORDER: &str = "#d0d7de";
const CODE_MUTED_TEXT: &str = "#57606a";
const CODE_LINE_NUMBER: &str = "#8c959f";
const UNRECOGNIZED_LANGUAGE_LABEL: &str = "Unrecognized language";

static CODE_INLINE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline")
        .background("grey.1")
        .border_radius("4px")
        .padding("2px 6px")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875em")
});

static CODE_BLOCK_CONTAINER_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .position("relative")
        .background(CODE_BG)
        .border("1px solid")
        .border_color(CODE_BORDER)
        .border_radius("6px")
        .overflow("hidden")
});

static CODE_BLOCK_HEADER_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .justify_content("space-between")
        .gap("8px")
        .padding("4px 8px 4px 16px")
        .border_bottom("1px solid")
        .border_color(CODE_BORDER)
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.75rem")
        .color(CODE_MUTED_TEXT)
});

static CODE_COPY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .justify_content("center")
        .background("transparent")
        .border("none")
        .border_radius("6px")
        .padding("5px")
        .cursor("pointer")
        .color(CODE_MUTED_TEXT)
        .hover(sx().background("rgba(31, 35, 40, 0.08)").color("#1f2328"))
});

// Centered on the first code line specifically (not the container as a
// whole - would drift off-center against a single-line block otherwise):
// `CODE_LINES_SX`'s 12px top padding, plus half of its 20px line-height,
// minus half the button's own ~26px height (5px padding + 14px icon + 1px
// border, both edges).
static CODE_COPY_BUTTON_FLOATING_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .justify_content("center")
        .position("absolute")
        .top("9px")
        .right("8px")
        .background(CODE_BG)
        .border("1px solid")
        .border_color(CODE_BORDER)
        .border_radius("6px")
        .padding("5px")
        .cursor("pointer")
        .color(CODE_MUTED_TEXT)
        .hover(sx().background("rgba(31, 35, 40, 0.08)").color("#1f2328"))
});

static CODE_BLOCK_SCROLL_SX: StaticSx = StaticSx::new(|| sx().overflow("auto"));

static CODE_LINES_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .width("max-content")
        .min_width("100%")
        .padding("12px 0")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875rem")
        // Explicit rather than left to the font's own metrics - the
        // floating copy button's vertical centering is computed against
        // this exact value.
        .line_height("20px")
});

static CODE_LINE_ROW_SX: StaticSx = StaticSx::new(|| sx().display("flex").flex_direction("row"));

static CODE_LINE_NUMBER_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .user_select("none")
        .text_align("right")
        .padding("0 12px")
        .color(CODE_LINE_NUMBER)
});

static CODE_LINE_CONTENT_SX: StaticSx =
    StaticSx::new(|| sx().flex("1").white_space("pre").padding_right("16px"));

static CODE_PLAIN_PRE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .margin("0")
        .padding("12px 16px")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875rem")
});

fn copy_icon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            width: "14px",
            height: "14px",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

fn check_icon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            width: "14px",
            height: "14px",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CodeProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// Renders as a `pre`-wrapped, multi-line block instead of inline `code`.
    #[props(default)]
    block: bool,
    /// Runtime text to syntax-highlight - mutually exclusive with `children`.
    /// Also what enables line numbers and the copy button, since those need
    /// an actual string, not opaque `children`.
    #[props(default, into)]
    source: Option<String>,
    /// Unrecognized values fall back to no highlighting rather than a guess.
    #[props(default, into)]
    language: Input<Language>,
    /// `block` only. Shows the language (or "Unrecognized language" if the
    /// name isn't in libero's catalog, or isn't enabled via a `code-lang-*`
    /// feature) in a bar above the code.
    #[props(default = true)]
    header: bool,
    /// `block` only, and only takes effect with `source` (nothing to copy
    /// from `children`). Without `header`, floats in the top-right corner.
    #[props(default = true)]
    copyable: bool,
    children: Element,
}

fn code_lines(lines: &[HighlightedLine]) -> Element {
    let gutter_width = format!("{}ch", lines.len().to_string().len());

    rsx! {
        Box {
            component: "div",
            framework_sx: &CODE_LINES_SX,
            for (index, line) in lines.iter().enumerate() {
                Box {
                    component: "div",
                    framework_sx: &CODE_LINE_ROW_SX,
                    Box {
                        component: "span",
                        framework_sx: &CODE_LINE_NUMBER_SX,
                        sx: sx().min_width(gutter_width.clone()),
                        {(index + 1).to_string()}
                    }
                    Box {
                        component: "span",
                        framework_sx: &CODE_LINE_CONTENT_SX,
                        for (text, class) in line.iter() {
                            span { class: *class, {text.as_str()} }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CopyButton(source: String, floating: bool) -> Element {
    let mut clipboard: Clipboard = use_clipboard();
    // Native `button`, not `Box` - needs `onmouseleave`, which isn't in
    // `Box`'s curated event set.
    let sx_ref: &'static StaticSx = if floating {
        &CODE_COPY_BUTTON_FLOATING_SX
    } else {
        &CODE_COPY_BUTTON_SX
    };
    let class = crate::hooks::use_css(sx_ref, crate::CssLayer::Framework);

    rsx! {
        button {
            r#type: "button",
            "aria-label": "Copy code",
            class,
            onclick: move |_| clipboard.copy(source.clone()),
            onmouseleave: move |_| clipboard.reset(),
            if clipboard.copied() {
                {check_icon()}
            } else {
                {copy_icon()}
            }
        }
    }
}

#[component]
pub fn Code(props: CodeProps) -> Element {
    use_token_theme();

    if props.block {
        let language = props.language.as_ref().copied();
        let source = props.source.clone();
        let highlighted = use_resource(use_reactive!(|source, language| async move {
            match (source, language) {
                (Some(source), Some(language)) => highlight_lazy(source, language).await,
                _ => None,
            }
        }));
        let lines = props.source.as_deref().map(|source| {
            highlighted
                .read()
                .clone()
                .flatten()
                .unwrap_or_else(|| plain_lines(source))
        });
        let label = language
            .filter(|language| language.is_available())
            .map(Language::label)
            .unwrap_or(UNRECOGNIZED_LANGUAGE_LABEL);
        let show_copy = props.copyable && props.source.is_some();

        return rsx! {
            Box {
                component: "div",
                class: props.class,
                sx: props.sx,
                states: props.states,
                framework_sx: &CODE_BLOCK_CONTAINER_SX,
                attributes: props.attributes,
                if props.header {
                    Box {
                        component: "div",
                        framework_sx: &CODE_BLOCK_HEADER_SX,
                        span { {label} }
                        if show_copy {
                            CopyButton { source: props.source.clone().unwrap(), floating: false }
                        }
                    }
                } else if show_copy {
                    CopyButton { source: props.source.clone().unwrap(), floating: true }
                }
                Box {
                    component: "div",
                    framework_sx: &CODE_BLOCK_SCROLL_SX,
                    if let Some(lines) = &lines {
                        {code_lines(lines)}
                    } else {
                        Box {
                            component: "pre",
                            framework_sx: &CODE_PLAIN_PRE_SX,
                            Box { component: "code", {props.children} }
                        }
                    }
                }
            }
        };
    }

    if let Some(source) = props.source.clone() {
        let language = props.language.as_ref().copied();
        let highlighted = use_resource(use_reactive!(|source, language| async move {
            match language {
                Some(language) => highlight_lazy(source, language).await,
                None => None,
            }
        }));

        return rsx! {
            Box {
                component: "code",
                class: props.class,
                sx: props.sx,
                states: props.states,
                framework_sx: &CODE_INLINE_SX,
                attributes: props.attributes,
                if let Some(lines) = highlighted.read().clone().flatten() {
                    for line in lines.iter() {
                        for (text, class) in line.iter() {
                            span { class: *class, {text.as_str()} }
                        }
                    }
                } else {
                    {source.as_str()}
                }
            }
        };
    }

    rsx! {
        Box {
            component: "code",
            class: props.class,
            sx: props.sx,
            states: props.states,
            framework_sx: &CODE_INLINE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
