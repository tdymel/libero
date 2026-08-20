use std::collections::HashSet;

use dioxus::prelude::*;

use super::highlight::{HighlightedLine, Language, highlight_lazy, plain_lines};
use super::token_theme::use_token_theme;
use crate::{
    CssLayer,
    components::{
        ActionIcon, Box, HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    hooks::{Clipboard, use_clipboard, use_css},
    sx::{StaticSx, Sx, sx},
    theme::{
        CODE_BACKGROUND, CODE_BORDER, CODE_COPY_HOVER_BACKGROUND, CODE_COPY_HOVER_TEXT,
        CODE_FONT_FAMILY, CODE_LINE_NUMBER, CODE_MUTED_TEXT, ColorCss, ColorShade, CssVar,
    },
};

const UNRECOGNIZED_LANGUAGE_LABEL: &str = "Unrecognized language";

// Shared with `max_lines`' height math, so neither drifts from
// `CODE_LINES_SX`'s actual line-height and padding.
const CODE_LINE_HEIGHT_PX: u32 = 20;
// Set once on the lines container and inherited, so the gutter's width varies
// per block without every row carrying an `sx` of its own.
const CODE_GUTTER_WIDTH_VAR: CssVar = CssVar::new("--lsx-code-gutter-width");
const CODE_LINES_VERTICAL_PADDING_PX: u32 = 24;

static CODE_INLINE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline")
        .background("grey.2")
        .border_radius("4px")
        .padding("2px 6px")
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875em")
});

static CODE_BLOCK_CONTAINER_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .position("relative")
        .background(CODE_BACKGROUND.value())
        .border("1px solid")
        .border_color(CODE_BORDER.value())
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
        .border_color(CODE_BORDER.value())
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.75rem")
        .color(CODE_MUTED_TEXT.value())
});

// Only what differs from `ActionIcon`'s own base styling.
static CODE_COPY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().border_radius("6px")
        .padding("5px")
        .color(CODE_MUTED_TEXT.value())
        .hover(
            sx().background(CODE_COPY_HOVER_BACKGROUND.value())
                .color(CODE_COPY_HOVER_TEXT.value()),
        )
});

// Centered on the first code line, not the container, which would drift on a
// single-line block: 12px top padding + half of the 20px line-height, minus
// half the button's ~26px height.
static CODE_COPY_BUTTON_FLOATING_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("9px")
        .right("8px")
        .background(CODE_BACKGROUND.value())
        .border("1px solid")
        .border_color(CODE_BORDER.value())
        .border_radius("6px")
        .padding("5px")
        .color(CODE_MUTED_TEXT.value())
        .hover(
            sx().background(CODE_COPY_HOVER_BACKGROUND.value())
                .color(CODE_COPY_HOVER_TEXT.value()),
        )
});

static CODE_BLOCK_SCROLL_SX: StaticSx = StaticSx::new(|| sx().overflow("auto"));

static CODE_LINES_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .width("max-content")
        .min_width("100%")
        .padding(format!("{}px 0", CODE_LINES_VERTICAL_PADDING_PX / 2))
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875rem")
        // Explicit, not the font's metrics: the copy button's centering and
        // `max_lines`' scroll height are computed against this exact value.
        .line_height(format!("{CODE_LINE_HEIGHT_PX}px"))
});

// `box-shadow` rather than `border-left`, so the accent bar doesn't shift
// content relative to unmarked rows. The three states are mutually exclusive
// (see `code_lines`'s `row_state`).
static CODE_LINE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("row")
        .when(
            "highlighted",
            sx().background("primary.1").box_shadow(format!(
                "inset 3px 0 0 {}",
                ColorCss::PRIMARY.value(ColorShade::S5)
            )),
        )
        .when(
            "diff-add",
            sx().background("success.1").box_shadow(format!(
                "inset 3px 0 0 {}",
                ColorCss::SUCCESS.value(ColorShade::S5)
            )),
        )
        .when(
            "diff-remove",
            sx().background("error.1").box_shadow(format!(
                "inset 3px 0 0 {}",
                ColorCss::ERROR.value(ColorShade::S5)
            )),
        )
});

static CODE_LINE_NUMBER_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .user_select("none")
        .text_align("right")
        .padding("0 12px")
        .min_width(CODE_GUTTER_WIDTH_VAR.value_or("1ch"))
        .color(CODE_LINE_NUMBER.value())
});

// No gutter means nothing reserves a left inset; `no-gutter` matches
// `CODE_LINE_NUMBER_SX`'s 12px so text isn't flush.
static CODE_LINE_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().flex("1")
        .white_space("pre")
        .padding_right("16px")
        .when("no-gutter", sx().padding_left("12px"))
});

static CODE_PLAIN_PRE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .margin("0")
        .padding(format!("{}px 16px", CODE_LINES_VERTICAL_PADDING_PX / 2))
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875rem")
        // Matches `CODE_LINES_SX`, so `max_lines` is right for the opaque
        // `children` path too.
        .line_height(format!("{CODE_LINE_HEIGHT_PX}px"))
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

base_props! {
    pub struct CodeProps {
        /// Renders as a `pre`-wrapped, multi-line block instead of inline `code`.
        #[props(default)]
        block: bool,
        /// Text to syntax-highlight, mutually exclusive with `children`. Line
        /// numbers and the copy button need a real string, so they need this.
        #[props(default, into)]
        source: Option<String>,
        /// Unrecognized values fall back to no highlighting rather than a guess.
        #[props(default, into)]
        language: Input<Language>,
        /// `block` only. A bar above the code naming the language, or
        /// "Unrecognized language" if it isn't in the catalog or its
        /// `code-lang-*` feature is off.
        #[props(default = true)]
        header: bool,
        /// `block` only, and only takes effect with `source` (nothing to copy
        /// from `children`). Without `header`, floats in the top-right corner.
        #[props(default = true)]
        copyable: bool,
        /// `block` only. Caps the visible height to roughly this many lines
        /// and scrolls past it; unset grows to fit. Long lines always scroll
        /// horizontally regardless.
        #[props(default)]
        max_lines: Option<u32>,
        /// `block` only, and only takes effect with `source`. Toggles the
        /// line-number gutter.
        #[props(default = true)]
        line_numbers: bool,
        /// `block` + `source` only. 1-indexed lines to emphasize, e.g.
        /// `"1,5-7,10"`. Malformed segments are skipped, not rejected.
        #[props(default, into)]
        highlight_lines: Option<String>,
        /// `block` + `source` only. Reads `source` as a unified diff: a
        /// leading `+`/`-` colors the row and is stripped from what's shown,
        /// highlighted and copied. Wins over `highlight_lines`.
        #[props(default)]
        diff: bool,
        children: Element,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffStatus {
    Added,
    Removed,
}

/// From the raw leading byte, not the tokenized spans, so the grammar's
/// scoping of `+`/`-` can't affect it.
fn diff_status(line: &str) -> Option<DiffStatus> {
    match line.as_bytes().first() {
        Some(b'+') => Some(DiffStatus::Added),
        Some(b'-') => Some(DiffStatus::Removed),
        _ => None,
    }
}

/// Drops a leading `+`/`-`, but not a leading space, so unchanged lines need
/// no strict unified-diff form. Used by highlighting, plain rendering and the
/// copy button alike, so shown and copied always match.
fn strip_diff_markers(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.as_bytes().first() {
            Some(b'+') | Some(b'-') => &line[1..],
            _ => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `"3,5-7,10"` into the 1-indexed lines it names. Malformed segments are
/// skipped, not rejected.
fn parse_highlighted_lines(spec: &str) -> HashSet<usize> {
    let mut lines = HashSet::new();
    for segment in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match segment.split_once('-') {
            Some((start, end)) => {
                if let (Ok(start), Ok(end)) =
                    (start.trim().parse::<usize>(), end.trim().parse::<usize>())
                {
                    lines.extend(start..=end);
                }
            }
            None => {
                if let Ok(line) = segment.parse::<usize>() {
                    lines.insert(line);
                }
            }
        }
    }
    lines
}

fn code_line_row(
    index: usize,
    line: &HighlightedLine,
    line_numbers: bool,
    row_state: Option<&'static str>,
) -> Element {
    let states = row_state
        .map(|s| States::new().active(s))
        .unwrap_or_default();
    rsx! {
        Box {
            component: "div",
            framework_sx: &CODE_LINE_ROW_SX,
            states,
            if line_numbers {
                Box {
                    component: "span",
                    framework_sx: &CODE_LINE_NUMBER_SX,
                    {(index + 1).to_string()}
                }
            }
            Box {
                component: "span",
                framework_sx: &CODE_LINE_CONTENT_SX,
                states: States::new().with("no-gutter", !line_numbers),
                for (text, class) in line.iter() {
                    span { class: *class, {text.as_str()} }
                }
            }
        }
    }
}

fn gutter_variables(gutter_width: String) -> Variables {
    variables().with(CODE_GUTTER_WIDTH_VAR, gutter_width)
}

fn code_lines(
    lines: &[HighlightedLine],
    line_numbers: bool,
    highlighted_lines: &HashSet<usize>,
    diff_statuses: &[Option<DiffStatus>],
) -> Element {
    let gutter_width = format!("{}ch", lines.len().to_string().len());

    rsx! {
        Box {
            component: "div",
            framework_sx: &CODE_LINES_SX,
            variables: gutter_variables(gutter_width),
            for (index, line) in lines.iter().enumerate() {
                {
                    let row_state = match diff_statuses.get(index).copied().flatten() {
                        Some(DiffStatus::Added) => Some("diff-add"),
                        Some(DiffStatus::Removed) => Some("diff-remove"),
                        None if highlighted_lines.contains(&(index + 1)) => Some("highlighted"),
                        None => None,
                    };
                    code_line_row(index, line, line_numbers, row_state)
                }
            }
        }
    }
}

#[component]
fn CopyButton(source: String, floating: bool) -> Element {
    let mut clipboard: Clipboard = use_clipboard();
    // No `variant`/`color`, so `ActionIcon` adds no background of its own and
    // these fully control the look. Passed as `Input::Static`, so the CSS is
    // built once for the process rather than per copy button.
    let button_sx: &'static StaticSx = if floating {
        &CODE_COPY_BUTTON_FLOATING_SX
    } else {
        &CODE_COPY_BUTTON_SX
    };

    rsx! {
        ActionIcon {
            aria_label: "Copy code",
            sx: button_sx,
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

    let language = props.language.as_ref().copied();
    // Everything shown or copied uses the stripped version, so the two match.
    // Only `block` has markers; inline code is verbatim.
    let display_source = props.source.as_ref().map(|source| {
        if props.diff && props.block {
            strip_diff_markers(source)
        } else {
            source.clone()
        }
    });
    // Hoisted above both branches: hook slots are positional, so a
    // `use_resource` inside `if props.block` would swap slots when it flips.
    let source = display_source.clone();
    let highlighted = use_resource(use_reactive!(|source, language| async move {
        match (source, language) {
            (Some(source), Some(language)) => Some(highlight_lazy(source, language).await),
            _ => None,
        }
    }));

    // Every path roots in one element, so one `prepare()` above the branch
    // covers them all - only which framework style it carries differs.
    let boxed = use_box()
        .framework_sx(match props.block {
            true => &CODE_BLOCK_CONTAINER_SX,
            false => &CODE_INLINE_SX,
        })
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare();
    // `None` registers nothing, so the inline paths pay only the hook slot.
    let header_class = use_css(
        (props.block && props.header).then_some(&CODE_BLOCK_HEADER_SX),
        CssLayer::Framework,
    );

    if props.block {
        // Diff statuses need the markers still present.
        let diff_statuses: Vec<Option<DiffStatus>> = if props.diff {
            props
                .source
                .as_deref()
                .map(|source| source.lines().map(diff_status).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let lines = display_source.as_deref().map(|source| {
            highlighted
                .read()
                .clone()
                .flatten()
                .unwrap_or_else(|| plain_lines(source))
        });
        let label = language
            .map(Language::label)
            .unwrap_or(UNRECOGNIZED_LANGUAGE_LABEL);
        let copy_source = display_source.clone().filter(|_| props.copyable);
        let highlighted_lines = props
            .highlight_lines
            .as_deref()
            .map(parse_highlighted_lines)
            .unwrap_or_default();
        let scroll_sx = match props.max_lines {
            Some(max_lines) => sx().max_height(format!(
                "{}px",
                max_lines * CODE_LINE_HEIGHT_PX + CODE_LINES_VERTICAL_PADDING_PX
            )),
            None => sx(),
        };

        return boxed.render(
            HtmlTag::Div,
            props.attributes,
            vec![
                rsx! {
                    if props.header {
                        div { class: header_class,
                            span { {label} }
                            if let Some(copy_source) = copy_source.clone() {
                                CopyButton { source: copy_source, floating: false }
                            }
                        }
                    } else if let Some(copy_source) = copy_source.clone() {
                        CopyButton { source: copy_source, floating: true }
                    }
                },
                rsx! {
                    Box {
                        component: "div",
                        framework_sx: &CODE_BLOCK_SCROLL_SX,
                        sx: scroll_sx,
                        if let Some(lines) = &lines {
                            {code_lines(lines, props.line_numbers, &highlighted_lines, &diff_statuses)}
                        } else {
                            Box {
                                component: "pre",
                                framework_sx: &CODE_PLAIN_PRE_SX,
                                Box { component: "code", {props.children} }
                            }
                        }
                    }
                },
            ],
        );
    }

    if let Some(source) = props.source.clone() {
        return boxed.render(
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
                    {source.as_str()}
                }
            },
        );
    }

    boxed.render(HtmlTag::Code, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_highlighted_lines_accepts_singles_and_ranges() {
        assert_eq!(
            parse_highlighted_lines("1,5-7,10"),
            HashSet::from([1, 5, 6, 7, 10])
        );
    }

    #[test]
    fn parse_highlighted_lines_skips_malformed_segments() {
        assert_eq!(
            parse_highlighted_lines("1,,abc,5-,3"),
            HashSet::from([1, 3])
        );
    }

    #[test]
    fn parse_highlighted_lines_trims_whitespace() {
        assert_eq!(
            parse_highlighted_lines(" 1 , 3 - 4 "),
            HashSet::from([1, 3, 4])
        );
    }

    #[test]
    fn parse_highlighted_lines_empty_spec_is_empty() {
        assert_eq!(parse_highlighted_lines(""), HashSet::new());
    }

    #[test]
    fn diff_status_detects_added_and_removed() {
        assert_eq!(diff_status("+ new line"), Some(DiffStatus::Added));
        assert_eq!(diff_status("- old line"), Some(DiffStatus::Removed));
        assert_eq!(diff_status("  unchanged"), None);
        assert_eq!(diff_status(""), None);
    }

    #[test]
    fn strip_diff_markers_removes_leading_plus_and_minus_only() {
        let source = "fn greet() {\n-    old();\n+    new();\n}";
        assert_eq!(
            strip_diff_markers(source),
            "fn greet() {\n    old();\n    new();\n}"
        );
    }

    #[test]
    fn strip_diff_markers_leaves_unmarked_lines_untouched() {
        assert_eq!(
            strip_diff_markers(" leading space stays"),
            " leading space stays"
        );
        assert_eq!(strip_diff_markers("no marker at all"), "no marker at all");
    }
}
