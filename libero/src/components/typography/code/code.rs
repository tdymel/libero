use std::collections::HashSet;

use dioxus::prelude::*;

use super::highlight::{HighlightedLine, Language, highlight_lazy, plain_lines};
use super::token_theme::use_token_theme;
use crate::{
    components::{ActionIcon, Box, Input, States, common::base_props},
    hooks::{Clipboard, use_clipboard},
    sx::{StaticSx, Sx, sx},
    theme::{
        CODE_BACKGROUND, CODE_BORDER, CODE_FONT_FAMILY, CODE_LINE_NUMBER, CODE_MUTED_TEXT,
        ColorCss, ColorShade,
    },
};

const UNRECOGNIZED_LANGUAGE_LABEL: &str = "Unrecognized language";

// Shared with `max_lines`' height math below, so both stay in sync with
// `CODE_LINES_SX`/`CODE_PLAIN_PRE_SX`'s actual line-height and padding
// instead of a second set of hand-copied numbers.
const CODE_LINE_HEIGHT_PX: u32 = 20;
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

// display/align-items/justify-content/border/background(transparent)/cursor
// all match `ActionIcon`'s own base styling already - only what actually
// differs from it needs restating here.
static CODE_COPY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().border_radius("6px")
        .padding("5px")
        .color(CODE_MUTED_TEXT.value())
        .hover(sx().background("rgba(31, 35, 40, 0.08)").color("#1f2328"))
});

// Centered on the first code line specifically (not the container as a
// whole - would drift off-center against a single-line block otherwise):
// `CODE_LINES_SX`'s 12px top padding, plus half of its 20px line-height,
// minus half the button's own ~26px height (5px padding + 14px icon + 1px
// border, both edges).
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
        .hover(sx().background("rgba(31, 35, 40, 0.08)").color("#1f2328"))
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
        // Explicit rather than left to the font's own metrics - the
        // floating copy button's vertical centering, and `max_lines`' scroll
        // height, are both computed against this exact value.
        .line_height(format!("{CODE_LINE_HEIGHT_PX}px"))
});

// Light tint + a solid accent bar down the left edge (`box-shadow` rather
// than `border-left`, so the bar doesn't shift content relative to
// unmarked rows). Colors are the theme's actual primary/success/error, not
// independent theme fields - stay in sync with them automatically. The
// three states are mutually exclusive (see `code_lines`'s `row_state`), so
// a plain row simply carries none of them.
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
        .color(CODE_LINE_NUMBER.value())
});

static CODE_LINE_CONTENT_SX: StaticSx =
    StaticSx::new(|| sx().flex("1").white_space("pre").padding_right("16px"));

static CODE_PLAIN_PRE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .margin("0")
        .padding(format!("{}px 16px", CODE_LINES_VERTICAL_PADDING_PX / 2))
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.875rem")
        // Matches `CODE_LINES_SX` - keeps `max_lines`' scroll height correct
        // for the opaque-`children` path too, not just highlighted source.
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
        /// `block` only. Caps the visible height to roughly this many lines,
        /// scrolling vertically past it - unset (the default) grows to fit all
        /// content. Very long individual lines always scroll horizontally,
        /// regardless of this.
        #[props(default)]
        max_lines: Option<u32>,
        /// `block` only, and only takes effect with `source`. Toggles the
        /// line-number gutter.
        #[props(default = true)]
        line_numbers: bool,
        /// `block` only, and only takes effect with `source`. 1-indexed lines to
        /// visually emphasize, e.g. `"3"`, `"5-7"`, or `"1,5-7,10"`. Malformed
        /// segments are skipped rather than rejecting the whole value.
        #[props(default, into)]
        highlight_lines: Option<String>,
        /// `block` only, and only takes effect with `source`. Treats each line
        /// of `source` as a unified diff - a leading `+`/`-` colors that line's
        /// row (added/removed) and is itself stripped from what's displayed,
        /// highlighted, and copied. Other lines are left exactly as they are.
        /// Takes priority over `highlight_lines` on lines both would match.
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

/// Based on the raw line's leading byte, not its tokenized spans - keeps
/// diff-status detection independent of (and unaffected by) how the
/// grammar happened to split `+`/`-` into scopes.
fn diff_status(line: &str) -> Option<DiffStatus> {
    match line.as_bytes().first() {
        Some(b'+') => Some(DiffStatus::Added),
        Some(b'-') => Some(DiffStatus::Removed),
        _ => None,
    }
}

/// Drops a line's leading `+`/`-` (not a leading space - only lines
/// actually marked added/removed are touched, so `source` doesn't need to
/// follow strict unified-diff conventions for its unchanged lines). Used
/// for highlighting, plain rendering, and the copy button alike, so what's
/// shown and what's copied always match.
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

/// Parses a `"3,5-7,10"`-style spec into the 1-indexed line numbers it
/// names. Malformed segments (empty, non-numeric, backwards ranges) are
/// skipped rather than rejecting the whole spec.
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
    gutter_width: &str,
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
                    sx: sx().min_width(gutter_width.to_string()),
                    {(index + 1).to_string()}
                }
            }
            Box {
                component: "span",
                framework_sx: &CODE_LINE_CONTENT_SX,
                // Without the gutter there's nothing reserving left inset -
                // match `CODE_LINE_NUMBER_SX`'s own 12px so the block still
                // has breathing room instead of text flush on the edge.
                sx: if line_numbers { sx() } else { sx().padding_left("12px") },
                for (text, class) in line.iter() {
                    span { class: *class, {text.as_str()} }
                }
            }
        }
    }
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
            for (index, line) in lines.iter().enumerate() {
                {
                    let row_state = match diff_statuses.get(index).copied().flatten() {
                        Some(DiffStatus::Added) => Some("diff-add"),
                        Some(DiffStatus::Removed) => Some("diff-remove"),
                        None if highlighted_lines.contains(&(index + 1)) => Some("highlighted"),
                        None => None,
                    };
                    code_line_row(index, line, line_numbers, &gutter_width, row_state)
                }
            }
        }
    }
}

#[component]
fn CopyButton(source: String, floating: bool) -> Element {
    let mut clipboard: Clipboard = use_clipboard();
    // Neither `variant` nor `color` is set, so `ActionIcon` contributes no
    // background/color of its own - this `sx` (hover swap included) is the
    // only thing controlling the button's look.
    let button_sx = if floating {
        CODE_COPY_BUTTON_FLOATING_SX.clone()
    } else {
        CODE_COPY_BUTTON_SX.clone()
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
    // Everything actually shown or copied - the highlighter, the plain
    // fallback, the copy button - uses the marker-stripped version, so what
    // you read and what you copy always match. Only `block` has diff
    // markers to strip; inline code renders its source verbatim.
    let display_source = props.source.as_ref().map(|source| {
        if props.diff && props.block {
            strip_diff_markers(source)
        } else {
            source.clone()
        }
    });
    // Hoisted above both branches on purpose: hook slots are positional, so
    // a `use_resource` inside `if props.block` would hand its slot to the
    // inline branch's own the moment `block` flips.
    let source = display_source.clone();
    let highlighted = use_resource(use_reactive!(|source, language| async move {
        match (source, language) {
            (Some(source), Some(language)) => Some(highlight_lazy(source, language).await),
            _ => None,
        }
    }));

    if props.block {
        // Diff statuses come from the raw source - the markers must still
        // be there to detect.
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
        let show_copy = props.copyable && props.source.is_some();
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
                            CopyButton { source: display_source.clone().unwrap(), floating: false }
                        }
                    }
                } else if show_copy {
                    CopyButton { source: display_source.clone().unwrap(), floating: true }
                }
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
            }
        };
    }

    if let Some(source) = props.source.clone() {
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
