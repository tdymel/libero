use std::collections::HashSet;

use dioxus::prelude::*;

use super::highlight::{HighlightedLine, Language, highlight};
use super::token_theme::use_token_theme;
use crate::{
    CssLayer,
    components::{
        accessibility::VisuallyHidden,
        buttons::Copy,
        common::{
            HtmlTag, Input, Part, States, Variables, attr, base_props, inset_focus_ring_sx,
            names_itself, parts_enum, variables,
        },
        layout::{Box, BoxStyle, use_box},
    },
    hooks::{use_css, use_element, use_localization, use_theme},
    localization::{CodeBlockLabels, fill},
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{
        CODE_BLOCK_BACKGROUND, CODE_BLOCK_BORDER, CODE_BLOCK_COPY_HOVER_BACKGROUND,
        CODE_BLOCK_COPY_HOVER_TEXT, CODE_BLOCK_LINE_NUMBER, CODE_BLOCK_MUTED_TEXT,
        CODE_FONT_FAMILY, ColorCss, ColorShade, CssVar, MARKED_ROW_WASH_PERCENT, NamedColorCss,
        Size,
    },
    utils::warn,
};

// Shared with `max_lines`' height math, so neither drifts. Relative, so rows
// grow with a larger default font size, as the rem text does (2345).
const CODE_LINE_HEIGHT: &str = "1.25rem";
// Set once on the lines container and inherited by every row.
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
        .font_size(Size::Xs)
        .color(CODE_BLOCK_MUTED_TEXT.value())
});

// No `variant`/`color` on the `Copy`, so these fully control the look.
static CODE_COPY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().border_radius("6px")
        .color(CODE_BLOCK_MUTED_TEXT.value())
        .hover(
            sx().background(CODE_BLOCK_COPY_HOVER_BACKGROUND.value())
                .color(CODE_BLOCK_COPY_HOVER_TEXT.value()),
        )
});

// Measured on the scroll box; 0 under overlay scrollbars.
const CODE_SCROLLBAR_WIDTH_VAR: CssVar = CssVar::new("--lsx-code-block-scrollbar-width");

// Centred on the first code line: 12px padding + half a line, minus half the 26px button.
static CODE_COPY_BUTTON_FLOATING_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top(format!("calc({CODE_LINE_HEIGHT} / 2 - 1px)"))
        .right(format!(
            "calc(8px + {})",
            CODE_SCROLLBAR_WIDTH_VAR.value_or("0px")
        ))
        .background(CODE_BLOCK_BACKGROUND.value())
        .border("1px solid")
        .border_color(CODE_BLOCK_BORDER.value())
        .border_radius("6px")
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

static CODE_LINES_PRE_SX: StaticSx = StaticSx::new(|| sx().display("block").margin("0"));

// On the `code` inside the `pre`, keeping `pre > code` semantics (todo 595).
static CODE_LINES_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .width("max-content")
        .min_width("100%")
        .padding(format!("{}px 0", CODE_LINES_VERTICAL_PADDING_PX / 2))
        .font_family(CODE_FONT_FAMILY.value())
        .font_size(Size::Sm)
        // Explicit: the copy button's centring and `max_lines` compute against it.
        .line_height(CODE_LINE_HEIGHT)
});

// `box-shadow`, not `border-left`, so the accent bar doesn't shift content.
static CODE_LINE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("row")
        // A blank line has no gutter digit or text to give it height (todo 771).
        .min_height(CODE_LINE_HEIGHT)
        .when("highlighted", marked_row_sx(ColorCss::PRIMARY))
        .when("diff-add", marked_row_sx(ColorCss::SUCCESS))
        .when("diff-remove", marked_row_sx(ColorCss::ERROR))
});

// A faint wash: the `.1` tint took the token colours under 4.5:1, 6% some (todo 1543).
fn marked_row_sx(color: ColorCss) -> Sx {
    let accent = color.value(ColorShade::S5);
    sx().background(format!(
        "color-mix(in srgb, {accent} {MARKED_ROW_WASH_PERCENT}%, {})",
        CODE_BLOCK_BACKGROUND.value()
    ))
    .box_shadow(format!("inset 3px 0 0 {accent}"))
    // The dimmed gutter and marker fell under 4.27:1 on the wash; ink marks the row too.
    .selector(
        "& > [data-slot='line-number'], & > [data-slot='marker']",
        sx().color(NamedColorCss::INK.value()),
    )
    // Forced colours drop both, so the bar becomes a border there (todo 594).
    .media(
        "(forced-colors: active)",
        sx().border_left("3px solid CanvasText"),
    )
}

// Visible `+`/`-`, drawing only: the hidden label beside it is what is read.
static CODE_DIFF_MARKER_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .user_select("none")
        .padding_right("8px")
        .color(CODE_BLOCK_MUTED_TEXT.value())
        .when("no-gutter", sx().padding_left("12px"))
});

// Kept out of a selection, like the marker it speaks for.
static CODE_DIFF_LABEL_SX: StaticSx = StaticSx::new(|| sx().user_select("none"));

static CODE_BLOCK_LINE_NUMBER_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .user_select("none")
        .text_align("right")
        .padding("0 12px")
        // `border-box` counts the padding in `min-width`, so add it back.
        .min_width(format!(
            "calc({} + 24px)",
            CODE_GUTTER_WIDTH_VAR.value_or("1ch")
        ))
        .color(CODE_BLOCK_LINE_NUMBER.value())
});

// `no-gutter` matches the gutter's 12px, so text isn't flush.
static CODE_LINE_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().flex("1")
        .white_space("pre")
        .padding_right("16px")
        .when("no-gutter", sx().padding_left("12px"))
        .when("copy-space", sx().padding_right(COPY_SPACE))
});

// The floating copy button's 8px inset and ~26px width, plus a gap: the
// first line scrolls out from under it.
const COPY_SPACE: &str = "40px";

static CODE_PLAIN_PRE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .margin("0")
        .padding(format!("{}px 16px", CODE_LINES_VERTICAL_PADDING_PX / 2))
        .font_family(CODE_FONT_FAMILY.value())
        .font_size(Size::Sm)
        // As `CODE_LINES_SX`, so `max_lines` holds while highlighting runs.
        .line_height(CODE_LINE_HEIGHT)
        .when("copy-space", sx().padding_right(COPY_SPACE))
});

parts_enum! {
    /// [`CodeBlock`]'s inner parts, for its `parts` prop. Each is matched by
    /// path, so a `CodeBlock` nested in another keeps its own styles.
    pub enum CodeBlockPart {
        /// The bar above the code, with `header`.
        Header = "header" => "& > [data-slot='header']",
        /// The language name in the header.
        Language = "language" => "& > [data-slot='header'] > [data-slot='language']",
        /// The copy button, in the header or floating in the corner.
        Copy = "copy" => "& > [data-slot='copy'], & > [data-slot='header'] > [data-slot='copy']",
        /// The scrolling box round the `<pre>`.
        Scroll = "scroll" => "& > [data-slot='scroll']",
    }
}

base_props! {
    parts(CodeBlockPart);
    pub struct CodeBlockProps {
        #[props(into)]
        source: String,
        /// Unset or unknown, no highlighting.
        #[props(default, into)]
        language: Input<Language>,
        /// A bar above the code naming the language.
        #[props(default)]
        header: Option<bool>,
        /// Without `header`, floats in the top-right corner.
        #[props(default)]
        copyable: Option<bool>,
        /// Caps the height at about this many lines and scrolls past it.
        #[props(default)]
        max_lines: Option<u32>,
        #[props(default)]
        line_numbers: Option<bool>,
        /// 1-indexed lines to emphasize, e.g. `"1,5-7,10"`.
        #[props(default, into)]
        highlight_lines: Option<String>,
        /// A leading `+`/`-` marks an added/removed line and is dropped; other lines take no
        /// prefix, so not a unified diff. Wins over `highlight_lines`.
        #[props(default)]
        diff: bool,
        /// Names the block, its copy button and its scroll region, e.g. "The booking card, Rust code".
        /// Unset, the language, as "Rust code".
        #[props(default, into)]
        label: Option<String>,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffStatus {
    Added,
    Removed,
}

/// From the raw leading byte, so the grammar's scoping of `+`/`-` can't affect it.
fn diff_status(line: &str) -> Option<DiffStatus> {
    match line.as_bytes().first() {
        Some(b'+') => Some(DiffStatus::Added),
        Some(b'-') => Some(DiffStatus::Removed),
        _ => None,
    }
}

/// Drops a leading `+`/`-`, not a space.
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

/// What a diff copies: the unmarked and `+` lines, so the result compiles.
fn new_side(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.starts_with('-'))
        .map(|line| line.strip_prefix('+').unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One classless span per line, split like `highlight` splits.
fn plain_lines(source: &str) -> Vec<HighlightedLine> {
    source
        .lines()
        .map(|line| match line.is_empty() {
            true => Vec::new(),
            false => vec![(line.to_string(), None)],
        })
        .collect()
}

/// `"3,5-7,10"` into lines, plus the first overrunning range. Malformed segments are skipped;
/// a range stops at `line_count`, so `"1-1000000000"` cannot fill the set.
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

/// `None` outside a diff; a diff draws a marker column on every row.
type DiffCell = Option<Option<DiffStatus>>;

/// A row's parts, resolved once per block: a `Box` scope each was three per line.
#[derive(Clone)]
struct LineStyles {
    row: BoxStyle,
    number: BoxStyle,
    marker: BoxStyle,
    content: BoxStyle,
}

fn use_line_styles() -> LineStyles {
    LineStyles {
        row: use_box().framework_sx(&CODE_LINE_ROW_SX).prepare(),
        number: use_box().framework_sx(&CODE_BLOCK_LINE_NUMBER_SX).prepare(),
        marker: use_box().framework_sx(&CODE_DIFF_MARKER_SX).prepare(),
        content: use_box().framework_sx(&CODE_LINE_CONTENT_SX).prepare(),
    }
}

#[allow(clippy::too_many_arguments)]
fn code_line_row(
    index: usize,
    line: &HighlightedLine,
    line_numbers: bool,
    diff: DiffCell,
    row_state: Option<&'static str>,
    copy_space: bool,
    labels: &CodeBlockLabels,
    styles: &LineStyles,
) -> Element {
    let states = row_state
        .map(|s| States::new().active(s))
        .unwrap_or_default();
    let (marker, spoken) = match diff.flatten() {
        Some(DiffStatus::Added) => ("+", Some(labels.added)),
        Some(DiffStatus::Removed) => ("-", Some(labels.removed)),
        None => (" ", None),
    };
    // Read aloud, the numbers interleave with the code.
    let hidden = |slot: &'static str| vec![attr("aria-hidden", "true"), attr("data-slot", slot)];
    let number = line_numbers.then(|| {
        let text = rsx! { {(index + 1).to_string()} };
        styles
            .number
            .clone()
            .render(HtmlTag::Span, hidden("line-number"), text)
    });
    let marker = diff.is_some().then(|| {
        let text = rsx! { {marker} };
        styles
            .marker
            .clone()
            .with_states(&States::new().with("no-gutter", !line_numbers))
            .render(HtmlTag::Span, hidden("marker"), text)
    });
    let content = styles
        .content
        .clone()
        .with_states(
            &States::new()
                .with("no-gutter", !line_numbers && diff.is_none())
                .with("copy-space", copy_space),
        )
        .render(
            HtmlTag::Span,
            Vec::new(),
            rsx! {
                for (text, class) in line.iter() {
                    span { class: *class, {text.as_str()} }
                }
            },
        );
    // A `span`: a row sits inside `code`, which holds phrasing content only.
    styles.row.clone().with_states(&states).render(
        HtmlTag::Span,
        Vec::new(),
        rsx! {
            if let Some(number) = number {
                {number}
            }
            if let Some(marker) = marker {
                {marker}
            }
            if let Some(spoken) = spoken {
                VisuallyHidden { sx: &CODE_DIFF_LABEL_SX, "{spoken} " }
            }
            {content}
        },
    )
}

fn max_lines_height(max_lines: u32) -> String {
    format!("calc({max_lines} * {CODE_LINE_HEIGHT} + {CODE_LINES_VERTICAL_PADDING_PX}px)")
}

fn gutter_variables(gutter_width: String) -> Variables {
    variables().with(CODE_GUTTER_WIDTH_VAR, gutter_width)
}

fn code_lines(
    lines: &[HighlightedLine],
    line_numbers: bool,
    highlighted_lines: &HashSet<usize>,
    diff_statuses: Option<&[Option<DiffStatus>]>,
    floating_copy: bool,
    labels: &CodeBlockLabels,
    styles: &LineStyles,
) -> Element {
    let gutter_width = format!("{}ch", lines.len().to_string().len());

    rsx! {
        Box {
            component: "pre",
            framework_sx: &CODE_LINES_PRE_SX,
            Box {
                component: "code",
                framework_sx: &CODE_LINES_SX,
                variables: gutter_variables(gutter_width),
                for (index, line) in lines.iter().enumerate() {
                    {
                        let diff = diff_statuses.map(|statuses| statuses.get(index).copied().flatten());
                        let row_state = match diff.flatten() {
                            Some(DiffStatus::Added) => Some("diff-add"),
                            Some(DiffStatus::Removed) => Some("diff-remove"),
                            None if highlighted_lines.contains(&(index + 1)) => Some("highlighted"),
                            None => None,
                        };
                        let copy_space = floating_copy && index == 0;
                        code_line_row(index, line, line_numbers, diff, row_state, copy_space, labels, styles)
                    }
                }
            }
        }
    }
}

/// A block of highlighted code, with optional header, copy button, line numbers and diff marks.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::CodeBlock;
/// # fn app() -> Element {
/// rsx! {
///     CodeBlock { source: "let x = 1;", language: "rust", line_numbers: true }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/code-block>
#[component]
pub fn CodeBlock(props: CodeBlockProps) -> Element {
    use_token_theme();
    let theme = use_theme();
    let labels = use_localization().code_block;
    let copyable = props.copyable.unwrap_or(theme.code_block.copyable);
    let line_numbers = props.line_numbers.unwrap_or(theme.code_block.line_numbers);

    let language = props.language.as_ref().copied();
    // Nothing to name and nothing to copy: no empty bar (todo 1252).
    let header =
        props.header.unwrap_or(theme.code_block.header) && (language.is_some() || copyable);
    let display_source = if props.diff {
        strip_diff_markers(&props.source)
    } else {
        props.source.clone()
    };
    let source = display_source.clone();
    let highlighted = use_resource(use_reactive!(|source, language| async move {
        language.map(|language| highlight(&source, language))
    }));

    let scroll_label = match language {
        Some(language) => fill(labels.code_named, &[("language", &language.name(&labels))]),
        None => labels.code.to_string(),
    };
    let group_label = props.label.clone().unwrap_or(scroll_label);
    // A classic vertical scrollbar's width, which the floating copy button moves clear of.
    let mut scrollbar = use_signal(|| 0u32);
    let width = scrollbar();
    // A group, so the copy button is read as this block's (todo 1025).
    let boxed = use_box()
        .framework_sx(&CODE_BLOCK_CONTAINER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .style((width > 0).then(|| format!("{}:{width}px;", CODE_SCROLLBAR_WIDTH_VAR.name())))
        .prepare()
        .attr("role", Some("group"))
        .attr(
            "aria-label",
            (!names_itself(&props.attributes)).then(|| group_label.clone()),
        );
    let header_class = use_css(header.then_some(&CODE_BLOCK_HEADER_SX), CssLayer::Framework);
    let line_styles = use_line_styles();

    // Diff statuses need the markers still present.
    let diff_statuses: Vec<Option<DiffStatus>> = if props.diff {
        props.source.lines().map(diff_status).collect()
    } else {
        Vec::new()
    };
    // Unmarked, one plain `pre` in flight, not a row tree thrown away on resolve (~330 ns/line).
    // Marking asked for: unstyled rows until the grammar lands, so the marks show (todos 668, 2827).
    let marked = props.diff || props.highlight_lines.is_some();
    let lines = highlighted
        .read()
        .clone()
        .flatten()
        .or_else(|| marked.then(|| plain_lines(&display_source)));
    // No language, no label: the empty span keeps the copy button at the end.
    let label = language.map(|language| language.name(&labels));
    let copy_source = copyable.then(|| {
        if props.diff {
            new_side(&props.source)
        } else {
            display_source.clone()
        }
    });
    let floating_copy = copyable && !header;
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
        Some(max_lines) => sx().max_height(max_lines_height(max_lines)),
        None => sx(),
    }
    .into();

    // Focusable only while it scrolls; Safari does not make scrollers focusable itself.
    let scroll_element = use_element();
    let mut overflows = use_signal(|| false);
    let measure = move || {
        if !scroll_element.is_mounted() {
            return;
        }
        let (content, size) = (scroll_element.scroll_size(), scroll_element.dimensions());
        // The `pre` is a block: as wide as the box less its scrollbar.
        let pre = scroll_element
            .query_selector("pre")
            .ok()
            .map(|pre| pre.dimensions());
        spawn(async move {
            if let (Ok(content), Ok(size)) = (content.await, size.await) {
                // `scrollWidth` is rounded, the rect is not.
                let next = content.width > size.width + 1.0 || content.height > size.height + 1.0;
                if next != *overflows.peek() {
                    overflows.set(next);
                }
                if let Some(Ok(pre)) = match pre {
                    Some(pre) => Some(pre.await),
                    None => None,
                } {
                    let width = (size.width - pre.width).round().max(0.0) as u32;
                    if width != *scrollbar.peek() {
                        scrollbar.set(width);
                    }
                }
            }
        });
    };
    // `ResizeObserver` misses a content change, so measure again once highlighting lands.
    use_effect(move || {
        let _ = highlighted.read();
        measure();
    });
    let scrolls = overflows();
    let scroll_box = use_box()
        .framework_sx(&CODE_BLOCK_SCROLL_SX)
        .sx(&scroll_sx)
        .prepare()
        .element(&scroll_element)
        .attr("data-slot", Some(CodeBlockPart::Scroll.slot()))
        .event("onresize", move |_: Event<ResizeData>| measure())
        // Code is LTR: bidi would reorder its operators on an RTL page (todo 735).
        .attr("dir", Some("ltr"))
        .attr("tabindex", scrolls.then_some("0"))
        .attr("role", scrolls.then_some("region"))
        // `label` tells two blocks' regions apart in the landmark list (todo 2349).
        .attr("aria-label", scrolls.then(|| group_label.clone()));
    let code = match &lines {
        Some(lines) => code_lines(
            lines,
            line_numbers,
            &highlighted_lines,
            props.diff.then_some(&diff_statuses[..]),
            floating_copy,
            &labels,
            &line_styles,
        ),
        None => rsx! {
            Box {
                component: "pre",
                framework_sx: &CODE_PLAIN_PRE_SX,
                states: States::new().with("copy-space", floating_copy),
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
                div { class: header_class, "data-slot": CodeBlockPart::Header.slot(),
                    span { "data-slot": CodeBlockPart::Language.slot(), {label} }
                    if let Some(copy_source) = copy_source.clone() {
                        Copy {
                            "data-slot": CodeBlockPart::Copy.slot(),
                            value: copy_source,
                            aria_label: labels.copy,
                            label: group_label.clone(),
                            size: "xs",
                            sx: &CODE_COPY_BUTTON_SX,
                        }
                    }
                }
            } else if let Some(copy_source) = copy_source.clone() {
                Copy {
                    "data-slot": CodeBlockPart::Copy.slot(),
                    value: copy_source,
                    aria_label: labels.copy,
                    label: group_label.clone(),
                    size: "xs",
                    sx: &CODE_COPY_BUTTON_FLOATING_SX,
                }
            }
            {scroll_box.render(HtmlTag::Div, Vec::new(), code)}
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<CodeBlockPart>(),
            [
                ("header", "& > [data-slot='header']"),
                (
                    "language",
                    "& > [data-slot='header'] > [data-slot='language']"
                ),
                (
                    "copy",
                    "& > [data-slot='copy'], & > [data-slot='header'] > [data-slot='copy']"
                ),
                ("scroll", "& > [data-slot='scroll']"),
            ]
        );
    }

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

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// The copy button's description points at an element holding the group's name.
    fn described_text(html: &str) -> Option<String> {
        let id = html
            .split("aria-describedby=\"")
            .nth(1)?
            .split('"')
            .next()?;
        let text = html.split(&format!("id=\"{id}\"")).nth(1)?;
        Some(text.split('>').nth(1)?.split('<').next()?.to_string())
    }

    #[test]
    fn a_block_is_a_group_named_by_its_label() {
        let html = render(|| {
            rsx! {
                crate::LiberoProvider {
                    CodeBlock { source: "let a = 1;", language: "rust", label: "The setup, Rust code" }
                }
            }
        });
        assert!(html.contains(r#"role="group""#), "{html}");
        assert!(
            html.contains(r#"aria-label="The setup, Rust code""#),
            "{html}"
        );
        assert_eq!(
            described_text(&html).as_deref(),
            Some("The setup, Rust code")
        );
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn an_unlabelled_block_is_named_after_its_language() {
        let html = render(|| {
            rsx! {
                crate::LiberoProvider { CodeBlock { source: "x", language: "rust" } }
            }
        });
        assert!(html.contains(r#"aria-label="Rust code""#), "{html}");
        assert_eq!(described_text(&html).as_deref(), Some("Rust code"));
    }

    #[test]
    fn a_callers_name_wins_over_the_label() {
        let html = render(|| {
            rsx! {
                crate::LiberoProvider {
                    CodeBlock { source: "x", "aria-label": "Mine", copyable: false }
                }
            }
        });
        assert_eq!(html.matches("aria-label=").count(), 1, "{html}");
        assert!(html.contains(r#"aria-label="Mine""#), "{html}");
    }

    /// Todo 2345: a 200% default font size grew the text but not a px row pitch.
    #[test]
    fn the_row_pitch_follows_the_root_font_size() {
        assert!(CODE_LINE_HEIGHT.ends_with("rem"));
        assert_eq!(max_lines_height(3), "calc(3 * 1.25rem + 24px)");
        let css = crate::css::Stylesheet::from(&CODE_LINE_ROW_SX);
        assert!(css.as_str().contains("min-height:1.25rem"), "{css:?}");
        let css = crate::css::Stylesheet::from(&CODE_COPY_BUTTON_FLOATING_SX);
        assert!(
            css.as_str().contains("top:calc(1.25rem / 2 - 1px)"),
            "{css:?}"
        );
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
    fn a_diff_copies_the_new_side_only() {
        let source = "fn greet() {\n-    old();\n+    new();\n}";
        assert_eq!(new_side(source), "fn greet() {\n    new();\n}");
    }

    /// Todo 2827: the server render, and the frames before highlighting resolves, kept no marks.
    #[test]
    fn a_diff_with_a_language_shows_its_marks_before_highlighting_resolves() {
        let html = render(|| {
            rsx! {
                crate::LiberoProvider {
                    CodeBlock { source: "-let a = 1;\n+let a = 2;", language: "rust", diff: true }
                }
            }
        });
        assert_eq!(html.matches(r#"data-slot="marker""#).count(), 2, "{html}");
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
