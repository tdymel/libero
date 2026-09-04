# CodeBlock

Crate: `libero`
Import: `use libero::components::CodeBlock;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/code/code_block.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A `pre`-wrapped, multi-line code block with a line-number gutter, a copy button, a language header, optional diff rendering and line highlighting.

A `pre`-wrapped, multi-line block with a line-number gutter, a copy button and a
header naming the language. Content is `source`, a plain string - line numbers
and the copy button need one to split. Add `language` to syntax-highlight it. For
a snippet inside a sentence, use [`Code`](code.md).

Background, border, line numbers and the copy button come from
`Theme.code_block`, the font and token colors from `Theme.code`; the diff and
highlight tints derive from the theme's success, error and primary colors.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::CodeBlock;

const RUST_EXAMPLE: &str = r#"fn shout(word: &str) -> String {
    // Rust
    format!("{}!", word.to_uppercase())
}"#;

#[component]
fn Demo() -> Element {
    rsx! {
        CodeBlock { source: RUST_EXAMPLE, language: "rust" }
    }
}
```

`header`, `copyable` and `line_numbers` are on by default; `max_lines` caps the
visible height and scrolls past it, and `highlight_lines` emphasizes 1-indexed
rows:

```rust
use dioxus::prelude::*;
use libero::components::CodeBlock;

const RUST_EXAMPLE: &str = r#"fn shout(word: &str) -> String {
    // Rust
    format!("{}!", word.to_uppercase())
}"#;

#[component]
fn Demo() -> Element {
    rsx! {
        CodeBlock {
            source: RUST_EXAMPLE,
            language: "rust",
            header: false,
            line_numbers: false,
            highlight_lines: "2,3",
            max_lines: 3,
        }
    }
}
```

`diff: true` reads `source` as a unified diff: a leading `+`/`-` colors the row
and is stripped from what is shown, highlighted and copied - so the copy button
still yields compilable code. It wins over `highlight_lines`.

```rust
use dioxus::prelude::*;
use libero::components::CodeBlock;

const RUST_DIFF: &str = r#"fn shout(word: &str) -> String {
    // Rust
-    format!("{}", word)
+    format!("{}!", word.to_uppercase())
}"#;

#[component]
fn Demo() -> Element {
    rsx! {
        CodeBlock { source: RUST_DIFF, language: "rust", diff: true }
    }
}
```

## Languages

Libero ships grammars for 30 languages, each behind its own `code-lang-*` feature
so a build only pays for what it highlights. The default set covers `rust`,
`bash`, `css` and a few more; enable the rest as you need them. An unrecognized
name - or a recognized one whose feature is off - falls back to plain,
unhighlighted text, which is also what leaving `language` off does.

```
bash, c, cpp, csharp, css, dart, go, graphql, haskell, html, java, javascript,
json, kotlin, lua, markdown, objective-c, perl, php, powershell, python, r,
ruby, rust, scala, sql, swift, toml, typescript, yaml
```

`language` also takes aliases the feature names don't - `rs`, `py`, `c#`. The
grammars are hand-ported from Prism (<https://prismjs.com>), as is the tokenizer
that runs them; these 30 are where we started, not a closed set.

## Accessibility

In `diff` mode added and removed lines are told apart by their tint alone - add
a caption or surrounding prose when the distinction has to survive without
color.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `source` | `String` | required | The text to render, highlighted when `language` names a grammar this build compiles in. Line numbers and the copy button need a real string, so this is the only way to pass content. |
| `language` | `Language` | - | Unrecognized values fall back to no highlighting rather than a guess. |
| `header` | `bool` | `true` | A bar above the code naming the language, or "Unrecognized language" if it isn't in the catalog or its `code-lang-*` feature is off. |
| `copyable` | `bool` | `true` | Without `header`, floats in the top-right corner. |
| `max_lines` | `Option<u32>` | - | Caps the visible height to roughly this many lines and scrolls past it; unset grows to fit. Long lines always scroll horizontally regardless. |
| `line_numbers` | `bool` | `true` | Toggles the line-number gutter. |
| `highlight_lines` | `Option<String>` | - | 1-indexed lines to emphasize, e.g. `"1,5-7,10"`. Malformed segments are skipped, not rejected. |
| `diff` | `bool` | `false` | Reads `source` as a unified diff: a leading `+`/`-` colors the row and is stripped from what's shown, highlighted and copied. Wins over `highlight_lines`. |

Like every component, `CodeBlock` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`CodeBlockDefaults` on the theme; the font family and token colors come from
`CodeDefaults`. Highlight and diff row colors are deliberately not fields - they
derive from the theme's primary, success and error.

| Field | Type | Description |
|---|---|---|
| `background` | `&'static str` | Block background (`#f6f8fa`). |
| `border` | `&'static str` | Block and header border (`#d0d7de`). |
| `muted_text` | `&'static str` | Header text and the copy button's resting color (`#57606a`). |
| `line_number` | `&'static str` | Gutter digits (`#8c959f`). |
| `copy_hover_background` | `&'static str` | Copy button background on hover. |
| `copy_hover_text` | `&'static str` | Copy button text on hover. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-code-block-background` | Block background. |
| `--lsx-code-block-border` | Block and header border color. |
| `--lsx-code-block-muted-text` | Header and copy-button text. |
| `--lsx-code-block-line-number` | Gutter digit color. |
| `--lsx-code-block-copy-hover-background` | Copy button hover background. |
| `--lsx-code-block-copy-hover-text` | Copy button hover text. |
| `--lsx-code-block-gutter-width` | Gutter width in `ch`, computed from the line count. |

## Data attributes

State tokens on the rows and cells the block builds, not on its root.

| Token | On | Condition |
|---|---|---|
| `diff-add` / `diff-remove` | a line row | `diff` is set and the line starts with `+` / `-`. |
| `highlighted` | a line row | The line is named by `highlight_lines` (and `diff` is off). |
| `no-gutter` | a line's content cell | `line_numbers` is off, so the content takes the gutter's padding. |
