use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const CODE_BLOCK_BACKGROUND: CssVar = CssVar::new("--lsx-code-block-background");
pub const CODE_BLOCK_BORDER: CssVar = CssVar::new("--lsx-code-block-border");
pub const CODE_BLOCK_MUTED_TEXT: CssVar = CssVar::new("--lsx-code-block-muted-text");
pub const CODE_BLOCK_LINE_NUMBER: CssVar = CssVar::new("--lsx-code-block-line-number");
pub const CODE_BLOCK_COPY_HOVER_BACKGROUND: CssVar =
    CssVar::new("--lsx-code-block-copy-hover-background");
pub const CODE_BLOCK_COPY_HOVER_TEXT: CssVar = CssVar::new("--lsx-code-block-copy-hover-text");
/// The accent's share in a highlighted or diff row's wash, which the contrast test mixes too.
pub(crate) const MARKED_ROW_WASH_PERCENT: u32 = 4;

/// Theme defaults for `CodeBlock`, set on [`Theme`](crate::theme::Theme).
///
/// No highlight or diff row colours: they derive from primary/success/error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeBlockDefaults {
    pub background: &'static str,
    pub border: &'static str,
    pub muted_text: &'static str,
    pub line_number: &'static str,
    pub copy_hover_background: &'static str,
    pub copy_hover_text: &'static str,
    /// Draws the bar naming the language above the code.
    pub header: bool,
    /// Offers the copy button.
    pub copyable: bool,
    /// Draws the line-number gutter.
    pub line_numbers: bool,
}

impl CodeBlockDefaults {
    /// Every colour is a `muted` ramp step, so the block follows the scheme (todo 396).
    pub const DEFAULT: Self = Self {
        background: "var(--lsx-muted-1)",
        border: "var(--lsx-muted-4)",
        muted_text: "var(--lsx-muted-7)",
        // Rebased to 241's 4.27:1 where the dimmed role falls short (899).
        line_number: "var(--lsx-text-dimmed)",
        copy_hover_background: "var(--lsx-muted-3)",
        copy_hover_text: "var(--lsx-ink)",
        header: true,
        copyable: true,
        line_numbers: true,
    };
}

impl ToCssDeclarations for CodeBlockDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CODE_BLOCK_BACKGROUND.declare(self.background),
            CODE_BLOCK_BORDER.declare(self.border),
            CODE_BLOCK_MUTED_TEXT.declare(self.muted_text),
            CODE_BLOCK_LINE_NUMBER.declare(self.line_number),
            CODE_BLOCK_COPY_HOVER_BACKGROUND.declare(self.copy_hover_background),
            CODE_BLOCK_COPY_HOVER_TEXT.declare(self.copy_hover_text),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;
    use crate::theme::{
        ANCHOR_CODE_COLOR, CODE_TOK_ATTRIBUTE, CODE_TOK_COMMENT, CODE_TOK_CONSTANT,
        CODE_TOK_FUNCTION, CODE_TOK_HEADING, CODE_TOK_KEYWORD, CODE_TOK_NUMBER, CODE_TOK_STRING,
        CODE_TOK_TAG, CODE_TOK_TYPE, ColorCss, ColorShade, KBD_BACKGROUND, KBD_COLOR,
        NamedColorCss, TOOLTIP_BACKGROUND, TOOLTIP_COLOR, Theme, ThemeSet,
    };
    use crate::tokens::HexColor;

    /// `value` as `:root` resolves it, following `var()`s to a hex.
    fn resolved(css: &str, value: &str) -> HexColor {
        let root = css
            .split_once(":root{")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(block, _)| block)
            .expect("a :root block");
        let mut value = value.to_string();
        while let Some(name) = value.strip_prefix("var(").and_then(|v| v.strip_suffix(')')) {
            value = root
                .split(';')
                .find_map(|declaration| declaration.strip_prefix(&format!("{name}:")))
                .unwrap_or_else(|| panic!("{name} is declared"))
                .to_string();
        }
        HexColor::parse(&value).unwrap_or_else(|| panic!("{value} is a hex"))
    }

    /// Every text on the block and its marked rows, inline `Code`, `Kbd` and `Tooltip`, as `:root` ships it.
    fn shortfalls(theme: &Theme) -> Vec<String> {
        let css = Stylesheet::from(theme).as_str().to_string();
        let on = |surface: &CssVar, what: &str, text: &CssVar, floor: f32| {
            let ratio =
                resolved(&css, &text.value()).contrast_ratio(resolved(&css, &surface.value()));
            (ratio < floor).then(|| format!("{what} on {} {ratio:.2}", surface.name()))
        };
        let inline = CssVar::new("--lsx-muted-fill-2");
        let mut short = Vec::new();
        // The marked rows' wash of the accent's `.5`, as `marked_row_sx` mixes it (todo 1543).
        let block = resolved(&css, &CODE_BLOCK_BACKGROUND.value());
        let share = MARKED_ROW_WASH_PERCENT;
        let washes = [
            ("highlighted", ColorCss::PRIMARY),
            ("diff-add", ColorCss::SUCCESS),
            ("diff-remove", ColorCss::ERROR),
        ]
        .map(|(row, accent)| {
            let accent = resolved(&css, &accent.value(ColorShade::S5));
            let channel =
                |a: u8, b: u8| (u32::from(a) * share + u32::from(b) * (100 - share) + 50) / 100;
            let wash = HexColor::new(
                (channel(accent.r(), block.r()) << 16)
                    | (channel(accent.g(), block.g()) << 8)
                    | channel(accent.b(), block.b()),
            );
            (row, wash)
        });
        let on_rows = |what: &str, text: &str, floor: f32| {
            let text = resolved(&css, text);
            washes
                .iter()
                .filter_map(|(row, wash)| {
                    let ratio = text.contrast_ratio(*wash);
                    (ratio < floor).then(|| format!("{what} on the {row} row {ratio:.2}"))
                })
                .collect::<Vec<_>>()
        };
        for (what, token) in [
            ("keyword", CODE_TOK_KEYWORD),
            ("string", CODE_TOK_STRING),
            ("comment", CODE_TOK_COMMENT),
            ("number", CODE_TOK_NUMBER),
            ("constant", CODE_TOK_CONSTANT),
            ("function", CODE_TOK_FUNCTION),
            ("type", CODE_TOK_TYPE),
            ("tag", CODE_TOK_TAG),
            ("attribute", CODE_TOK_ATTRIBUTE),
            ("heading", CODE_TOK_HEADING),
        ] {
            short.extend(on(&CODE_BLOCK_BACKGROUND, what, &token, 4.5));
            short.extend(on(&inline, what, &token, 4.5));
            short.extend(on_rows(what, &token.value(), 4.5));
        }
        short.extend(on(
            &CODE_BLOCK_BACKGROUND,
            "muted text",
            &CODE_BLOCK_MUTED_TEXT,
            4.5,
        ));
        // A marked row's line number and diff marker.
        short.extend(on_rows("ink", &NamedColorCss::INK.value(), 4.5));
        short.extend(on(&inline, "link in code", &ANCHOR_CODE_COLOR, 4.5));
        short.extend(on(
            &CODE_BLOCK_BACKGROUND,
            "line number",
            &CODE_BLOCK_LINE_NUMBER,
            4.27,
        ));
        short.extend(on(
            &CODE_BLOCK_COPY_HOVER_BACKGROUND,
            "copy icon",
            &CODE_BLOCK_COPY_HOVER_TEXT,
            3.0,
        ));
        short.extend(on(&KBD_BACKGROUND, "kbd", &KBD_COLOR, 4.5));
        short.extend(on(&TOOLTIP_BACKGROUND, "tooltip", &TOOLTIP_COLOR, 4.5));
        short
    }

    /// Todo 241 on every shipped palette (todo 396): GitHub's token contrast is
    /// measured on white and does not carry over to a tint.
    #[test]
    fn every_derived_surface_reads_on_every_shipped_palette() {
        let mut short = Vec::new();
        for set in ThemeSet::CATALOGUE {
            for theme in [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
            {
                let falling = shortfalls(theme);
                if !falling.is_empty() {
                    let scheme = theme.surface.color_scheme();
                    short.push(format!("{} {scheme}: {}", set.name(), falling.join(", ")));
                }
            }
        }
        assert_eq!(short, Vec::<String>::new());
    }
}
