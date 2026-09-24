# CodeBlock

Crate: `libero`
Import: `use libero::components::CodeBlock;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/code/code_block.rs>
Index: [index.md](index.md) lists every other page
Description: A multi-line code block with line numbers, a copy button, a language header, diffs and highlighted lines.

A multi-line code block with line numbers, a copy button and a header naming
the language. Pass the text as `source` and set `language` to highlight it.
For a snippet inside a sentence, use [Code](code.md).

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

`header`, `copyable` and `line_numbers` are on by default. `max_lines` caps the
height and `highlight_lines` emphasizes lines:

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

`diff: true` reads `source` as a unified diff. A leading `+` or `-` colors the
row and stays out of what is copied, so the copy button still yields
compilable code.

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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `source` | `String` | required | The text. Highlighted when `language` names a grammar this build compiles in. |
| `language` | `Language` | - | One of 30 grammars, each behind its own `code-lang-*` feature, so a build pays only for what it highlights: bash, c, cpp, csharp, css, dart, go, graphql, haskell, html, java, javascript, json, kotlin, lua, markdown, objective-c, perl, php, powershell, python, r, ruby, rust, scala, sql, swift, toml, typescript, yaml. Aliases such as `rs`, `py` and `c#` work too. The default features cover `rust`, `bash`, `css` and a few more. An unknown name, or one whose feature is off, renders plain text. |
| `header` | `bool` | `true` | A bar above the code naming the language, or saying it is unknown. |
| `copyable` | `bool` | `true` | Shows a [`CopyButton`](copy_button.md). Without `header`, it floats in the top-right corner. |
| `max_lines` | `u32` | - | Caps the height at about this many lines and scrolls the rest. Unset, the block grows to fit. |
| `line_numbers` | `bool` | `true` | Shows the line-number gutter. |
| `highlight_lines` | `String` | - | Lines to emphasize, counted from 1, such as `"1,5-7,10"`. Malformed parts are skipped. |
| `diff` | `bool` | `false` | Reads `source` as a unified diff. A leading `+` or `-` colors the row and stays out of what is copied. Wins over `highlight_lines`. |
| `label` | `String` | - | Names the block and describes its copy button, such as "The booking card, Rust code". Unset, the language, such as "Rust code". |
| `parts` | `Parts<CodeBlockPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `CodeBlock` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `CodeBlockPart::Header` | `header` | The bar above the code, with `header`. |
| `CodeBlockPart::Language` | `language` | The language name in the header. |
| `CodeBlockPart::Copy` | `copy` | The copy button, in the header or floating in the corner. |
| `CodeBlockPart::Scroll` | `scroll` | The scrolling box round the code. |

## Accessibility

### Libero handles

- The block is a `group` named by `label`. Its copy button is "Copy code",
  described by the same words, so a screen reader hears "Copy code, Rust code".
- In `diff` mode a screen reader hears "added" or "removed" before a changed
  line.
- A block that scrolls is a focusable region named after its language, such as
  "Rust code". The words come from the [localization](localization.md).

### You must

- With several blocks on a page, give each a `label` that says what the code
  is, such as "The booking form, Rust code".
- A line in `highlight_lines` is marked only by color and a bar, so say in the
  text why it matters.

## Theme defaults

`CodeBlockDefaults` on the theme; the font family and token colors come from
`CodeDefaults`. Highlight and diff row colors derive from the theme's primary,
success and error.

| Field | Type | Description |
|---|---|---|
| `background` | `&'static str` | Block background (`var(--lsx-muted-1)`). |
| `border` | `&'static str` | Block and header border (`var(--lsx-muted-4)`). |
| `muted_text` | `&'static str` | Header text and the copy button's resting color (`var(--lsx-muted-7)`). |
| `line_number` | `&'static str` | Gutter digits (`var(--lsx-text-dimmed)`). |
| `copy_hover_background` | `&'static str` | Copy button background on hover (`var(--lsx-muted-3)`). |
| `copy_hover_text` | `&'static str` | Copy button text on hover (`var(--lsx-ink)`). |
| `header` | `bool` | Default `header` when the prop is omitted (`true`). |
| `copyable` | `bool` | Default `copyable` when the prop is omitted (`true`). |
| `line_numbers` | `bool` | Default `line_numbers` when the prop is omitted (`true`). |

The defaults are steps of the theme's `muted` ramp, so the block follows the
palette's page. A token or text color that falls short of 4.5:1 there (3:1
for line numbers) is darkened, or lightened on a dark page, until it reads.

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
