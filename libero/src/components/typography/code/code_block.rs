use std::collections::HashSet;

use dioxus::prelude::*;

use super::highlight::{HighlightedLine, Language, highlight};
use super::token_theme::use_token_theme;
use crate::{
    CssLayer,
    components::{
        ActionIcon, Box, HtmlTag, Input, States, Variables, VisuallyHidden,
        common::{
            CopiedIcon, CopyFailedIcon, CopyIcon, base_props, inset_focus_ring_sx, variables,
        },
        layout::use_box,
    },
    hooks::{Clipboard, use_clipboard, use_css, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{
        CODE_BLOCK_BACKGROUND, CODE_BLOCK_BORDER, CODE_BLOCK_COPY_HOVER_BACKGROUND,
        CODE_BLOCK_COPY_HOVER_TEXT, CODE_BLOCK_LINE_NUMBER, CODE_BLOCK_MUTED_TEXT,
        CODE_FONT_FAMILY, ColorCss, ColorShade, CssVar,
    },
    utils::warn,
};

const UNRECOGNIZED_LANGUAGE_LABEL: &str = "Unrecognized language";

// Shared with `max_lines`' height math, so neither drifts from
// `CODE_LINES_SX`'s actual line-height and padding.
const CODE_LINE_HEIGHT_PX: u32 = 20;
// Set once on the lines container and inherited, so the gutter's width varies
// per block without every row carrying an `sx` of its own.
const CODE_GUTTER_WIDTH_VAR: CssVar = CssVar::new("--lsx-code-block-gutter-width");
const CODE_LINES_VERTICAL_PADDING_PX: u32 = 24;

static CODE_BLOCK_CONTAINER_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .position("relative")
        .background(CODE_BLOCK_BACKGROUND.value())
        .border("1px solid")
        .border_color(CODE_BLOCK_BORDER.value())
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
        .border_color(CODE_BLOCK_BORDER.value())
        .font_family(CODE_FONT_FAMILY.value())
        .font_size("0.75rem")
        .color(CODE_BLOCK_MUTED_TEXT.value())
});

// Only what differs from `ActionIcon`'s own base styling.
static CODE_COPY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().border_radius("6px")
        .padding("5px")
        .color(CODE_BLOCK_MUTED_TEXT.value())
        .hover(
            sx().background(CODE_BLOCK_COPY_HOVER_BACKGROUND.value())
                .color(CODE_BLOCK_COPY_HOVER_TEXT.value()),
        )
});

// Centered on the first code line, not the container, which would drift on a
// single-line block: 12px top padding + half of the 20px line-height, minus
// half the button's ~26px height.
static CODE_COPY_BUTTON_FLOATING_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("9px")
        .right("8px")
        .background(CODE_BLOCK_BACKGROUND.value())
        .border("1px solid")
        .border_color(CODE_BLOCK_BORDER.value())
        .border_radius("6px")
        .padding("5px")
        .color(CODE_BLOCK_MUTED_TEXT.value())
        .hover(
            sx().background(CODE_BLOCK_COPY_HOVER_BACKGROUND.value())
                .color(CODE_BLOCK_COPY_HOVER_TEXT.value()),
        )
});

// Inset: the container clips, so an outset ring would be cut away.
static CODE_BLOCK_SCROLL_SX: StaticSx = StaticSx::new(|| {
    sx().overflow("auto")
        .focus_visible(inset_focus_ring_sx("-2px"))
});

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

static CODE_BLOCK_LINE_NUMBER_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .user_select("none")
        .text_align("right")
        .padding("0 12px")
        // Global `border-box` counts the padding in `min-width`, so add it back
        // or every gutter sizes to its own digits and `9` sits left of `10`.
        .min_width(format!(
            "calc({} + 24px)",
            CODE_GUTTER_WIDTH_VAR.value_or("1ch")
        ))
        .color(CODE_BLOCK_LINE_NUMBER.value())
});

// No gutter means nothing reserves a left inset; `no-gutter` matches
// `CODE_BLOCK_LINE_NUMBER_SX`'s 12px so text isn't flush.
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
        // Matches `CODE_LINES_SX`, so `max_lines` is right while highlighting
        // is still in flight.
        .line_height(format!("{CODE_LINE_HEIGHT_PX}px"))
});

base_props! {
    pub struct CodeBlockProps {
        /// The text to render, highlighted when `language` names a grammar
        /// this build compiles in. Line numbers and the copy button need a
        /// real string, so this is the only way to pass content.
        #[props(into)]
        source: String,
        /// Unrecognized values fall back to no highlighting rather than a guess.
        #[props(default, into)]
        language: Input<Language>,
        /// A bar above the code naming the language, or "Unrecognized
        /// language" if it isn't in the catalog or its `code-lang-*` feature
        /// is off.
        #[props(default)]
        header: Option<bool>,
        /// Without `header`, floats in the top-right corner.
        #[props(default)]
        copyable: Option<bool>,
        /// Caps the visible height to roughly this many lines and scrolls
        /// past it; unset grows to fit. Long lines always scroll horizontally
        /// regardless.
        #[props(default)]
        max_lines: Option<u32>,
        /// Toggles the line-number gutter.
        #[props(default)]
        line_numbers: Option<bool>,
        /// 1-indexed lines to emphasize, e.g. `"1,5-7,10"`. Malformed
        /// segments are skipped, not rejected. A range past the last line
        /// stops at it.
        #[props(default, into)]
        highlight_lines: Option<String>,
        /// Reads `source` as a unified diff: a leading `+`/`-` colors the row
        /// and is stripped from what's shown, highlighted and copied. Wins
        /// over `highlight_lines`.
        #[props(default)]
        diff: bool,
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

/// `"3,5-7,10"` into the 1-indexed lines it names, plus the first range that
/// ran past the last line. Malformed segments are skipped, not rejected. A
/// range stops at `line_count`, so a typo like `"1-1000000000"` cannot fill
/// the set with a billion entries.
fn parse_highlighted_lines(spec: &str, line_count: usize) -> (HashSet<usize>, Option<String>) {
    let mut lines = HashSet::new();
    let mut overrun = None;
    for segment in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match segment.split_once('-') {
            Some((start, end)) => {
                if let (Ok(start), Ok(end)) =
                    (start.trim().parse::<usize>(), end.trim().parse::<usize>())
                {
                    if end > line_count && overrun.is_none() {
                        overrun = Some(format!(
                            "CodeBlock: highlight_lines range `{segment}` runs past the last line ({line_count})"
                        ));
                    }
                    lines.extend(start..=end.min(line_count));
                }
            }
            None => {
                if let Ok(line) = segment.parse::<usize>() {
                    lines.insert(line);
                }
            }
        }
    }
    (lines, overrun)
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
                    framework_sx: &CODE_BLOCK_LINE_NUMBER_SX,
                    // Read aloud, the numbers interleave with the code.
                    aria_hidden: "true",
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
            // Reset first, so a second copy empties the status and fills it
            // again rather than leaving the same text a reader skips.
            onclick: move |_| {
                clipboard.reset();
                clipboard.copy(source.clone());
            },
            onmouseleave: move |_| clipboard.reset(),
            // A keyboard or touch user never leaves with a mouse.
            onblur: move |_| clipboard.reset(),
            if clipboard.copied() {
                CopiedIcon {}
            } else if clipboard.failed() {
                CopyFailedIcon {}
            } else {
                CopyIcon {}
            }
        }
        // Always mounted, so a reader is already watching it when the text
        // arrives - the check icon alone says nothing.
        VisuallyHidden { role: "status",
            if clipboard.copied() {
                "Copied"
            } else if clipboard.failed() {
                "Copy failed"
            }
        }
    }
}

#[component]
pub fn CodeBlock(props: CodeBlockProps) -> Element {
    use_token_theme();
    let theme = use_theme();
    let header = props.header.unwrap_or(theme.code_block.header);
    let copyable = props.copyable.unwrap_or(theme.code_block.copyable);
    let line_numbers = props.line_numbers.unwrap_or(theme.code_block.line_numbers);

    let language = props.language.as_ref().copied();
    // Everything shown or copied uses the stripped version, so the two match.
    let display_source = if props.diff {
        strip_diff_markers(&props.source)
    } else {
        props.source.clone()
    };
    let source = display_source.clone();
    let highlighted = use_resource(use_reactive!(|source, language| async move {
        language.map(|language| highlight(&source, language))
    }));

    let boxed = use_box()
        .framework_sx(&CODE_BLOCK_CONTAINER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare();
    // `None` registers nothing, so a headerless block pays only the hook slot.
    let header_class = use_css(header.then_some(&CODE_BLOCK_HEADER_SX), CssLayer::Framework);

    // Diff statuses need the markers still present.
    let diff_statuses: Vec<Option<DiffStatus>> = if props.diff {
        props.source.lines().map(diff_status).collect()
    } else {
        Vec::new()
    };
    // Only the highlighted rows. While highlighting is in flight the block
    // renders as one `pre` below rather than a full row tree that is thrown
    // away the moment it resolves - ~330 ns/line of wasted build on every
    // block.
    let lines = highlighted.read().clone().flatten();
    let label = language
        .map(Language::label)
        .unwrap_or(UNRECOGNIZED_LANGUAGE_LABEL);
    let copy_source = copyable.then(|| display_source.clone());
    let (highlighted_lines, overrun) = props
        .highlight_lines
        .as_deref()
        .map(|spec| parse_highlighted_lines(spec, display_source.lines().count()))
        .unwrap_or_default();
    // Once per mount: the highlight resource resolving is already a second render.
    use_hook(move || {
        if let Some(message) = overrun {
            warn(&message);
        }
    });
    let scroll_sx: Input<Sx> = match props.max_lines {
        Some(max_lines) => sx().max_height(format!(
            "{}px",
            max_lines * CODE_LINE_HEIGHT_PX + CODE_LINES_VERTICAL_PADDING_PX
        )),
        None => sx(),
    }
    .into();

    // A box that scrolls must be reachable from the keyboard (Safari does not
    // make scrollers focusable), but one that does not is only a tab stop in
    // the way - so it is measured.
    let scroll_element = use_element();
    let mut overflows = use_signal(|| false);
    let measure = move || {
        if !scroll_element.is_mounted() {
            return;
        }
        let (content, size) = (scroll_element.scroll_size(), scroll_element.dimensions());
        spawn(async move {
            if let (Ok(content), Ok(size)) = (content.await, size.await) {
                // `scrollWidth` is rounded, the rect is not.
                let next = content.width > size.width + 1.0 || content.height > size.height + 1.0;
                if next != *overflows.peek() {
                    overflows.set(next);
                }
            }
        });
    };
    // `ResizeObserver` reports the box, not its content, so the rows that
    // replace the plain `pre` once highlighting resolves are measured here.
    use_effect(move || {
        let _ = highlighted.read();
        measure();
    });
    let scrolls = overflows();
    let scroll_label = match language {
        Some(language) => format!("{} code", Language::label(language)),
        None => "Code".to_string(),
    };
    let scroll_box = use_box()
        .framework_sx(&CODE_BLOCK_SCROLL_SX)
        .sx(&scroll_sx)
        .prepare()
        .element(&scroll_element)
        .event("onresize", move |_: Event<ResizeData>| measure())
        .attr("tabindex", scrolls.then_some("0"))
        .attr("role", scrolls.then_some("region"))
        .attr("aria-label", scrolls.then_some(scroll_label));
    let code = match &lines {
        Some(lines) => code_lines(lines, line_numbers, &highlighted_lines, &diff_statuses),
        None => rsx! {
            Box {
                component: "pre",
                framework_sx: &CODE_PLAIN_PRE_SX,
                Box {
                    component: "code",
                    {display_source.as_str()}
                }
            }
        },
    };

    boxed.render(
        HtmlTag::Div,
        props.attributes,
        rsx! {
            if header {
                div { class: header_class,
                    span { {label} }
                    if let Some(copy_source) = copy_source.clone() {
                        CopyButton { source: copy_source, floating: false }
                    }
                }
            } else if let Some(copy_source) = copy_source.clone() {
                CopyButton { source: copy_source, floating: true }
            }
            {scroll_box.render(HtmlTag::Div, Vec::new(), code)}
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_highlighted_lines_accepts_singles_and_ranges() {
        assert_eq!(
            parse_highlighted_lines("1,5-7,10", 10).0,
            HashSet::from([1, 5, 6, 7, 10])
        );
    }

    #[test]
    fn parse_highlighted_lines_skips_malformed_segments() {
        assert_eq!(
            parse_highlighted_lines("1,,abc,5-,3", 10).0,
            HashSet::from([1, 3])
        );
    }

    #[test]
    fn parse_highlighted_lines_trims_whitespace() {
        assert_eq!(
            parse_highlighted_lines(" 1 , 3 - 4 ", 10).0,
            HashSet::from([1, 3, 4])
        );
    }

    #[test]
    fn parse_highlighted_lines_empty_spec_is_empty() {
        assert_eq!(parse_highlighted_lines("", 10).0, HashSet::new());
    }

    #[test]
    fn parse_highlighted_lines_clamps_a_range_to_the_line_count() {
        let (lines, overrun) = parse_highlighted_lines("2-1000000000", 4);
        assert_eq!(lines, HashSet::from([2, 3, 4]));
        assert!(overrun.is_some_and(|message| message.contains("2-1000000000")));
    }

    #[test]
    fn a_range_past_the_last_line_warns_once_per_mount() {
        crate::utils::take_warnings();
        let mut dom = VirtualDom::new(|| {
            let mut copyable = use_signal(|| true);
            use_hook(move || copyable.set(false));
            rsx! {
                crate::LiberoProvider {
                    CodeBlock { source: "a\nb", highlight_lines: "1-9", copyable: copyable() }
                }
            }
        });
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        let count = crate::utils::take_warnings()
            .iter()
            .filter(|warning| warning.contains("`1-9`"))
            .count();
        assert_eq!(count, 1);
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
