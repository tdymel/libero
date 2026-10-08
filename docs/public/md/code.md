# Code

Crate: `libero`
Import: `use libero::components::Code;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/code/code.rs>
Index: [index.md](index.md) lists every other page
Description: An inline `<code>` snippet, optionally syntax highlighted.

A `<code>` element for a snippet inside a sentence. Pass the text as `source`
and set `language` to highlight it. For a multi-line block with line numbers
and a copy button, use [CodeBlock](code_block.md). The font and the token
colors come from the theme.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Code, Text};

#[component]
fn Demo() -> Element {
    rsx! {
        Text {
            "Bind it with "
            Code { source: "let width: u32 = 320;", language: "rust" }
            " before the first draw."
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `source` | `String` | required | The text. Highlighted when `language` names a grammar this build compiles in. |
| `language` | `Language` | - | One of 30 grammars, each behind its own `code-lang-*` feature, so a build pays only for what it highlights: bash, c, cpp, csharp, css, dart, go, graphql, haskell, html, java, javascript, json, kotlin, lua, markdown, objective-c, perl, php, powershell, python, r, ruby, rust, scala, sql, swift, toml, typescript, yaml. Aliases such as `rs`, `py` and `c#` work too. The default features cover `rust`, `bash`, `css` and a few more. An unknown name, or one whose feature is off, renders plain text. |

Like every component, `Code` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- Each snippet is a real `<code>`. Highlighting only adds colored spans, so a
  screen reader reads the source as it is.
- A long identifier wraps at any character, so it fits a 320px column. A span
  of up to 20 characters stays on one line at any font size, so in a large
  heading or a narrow cell it can run wider than its column.

### Example

An inline `Code { source: "use_theme()" }` in a sentence: a screen reader
reads `use_theme()` as written, with or without highlighting.

## Theme defaults

`CodeDefaults` on the theme, the monospace stack plus one color per token
kind. Shared with [code_block.md](code_block.md).

| Field | Type | Description |
|---|---|---|
| `font_family` | `&'static str` | Monospace stack for inline code and code blocks. |
| `tok_keyword` | `&'static str` | Keyword color. |
| `tok_string` | `&'static str` | String literal color. |
| `tok_comment` | `&'static str` | Comment color; also rendered italic. |
| `tok_number` | `&'static str` | Number literal color. |
| `tok_constant` | `&'static str` | Constant color. |
| `tok_function` | `&'static str` | Function name color. |
| `tok_type` | `&'static str` | Type name color. |
| `tok_tag` | `&'static str` | Markup tag color. |
| `tok_attribute` | `&'static str` | Markup attribute color. |
| `tok_heading` | `&'static str` | Markdown heading color; also rendered bold. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-code-font-family` | Monospace stack for the `<code>` element. |
| `--lsx-code-tok-keyword` | Keyword color. |
| `--lsx-code-tok-string` | String literal color. |
| `--lsx-code-tok-comment` | Comment color. |
| `--lsx-code-tok-number` | Number literal color. |
| `--lsx-code-tok-constant` | Constant color. |
| `--lsx-code-tok-function` | Function name color. |
| `--lsx-code-tok-type` | Type name color. |
| `--lsx-code-tok-tag` | Markup tag color. |
| `--lsx-code-tok-attribute` | Markup attribute color. |
| `--lsx-code-tok-heading` | Markdown heading color. |

Highlighted spans carry fixed classes, one per token kind, each reading the
variable above it: `.lsx-tok-keyword`, `.lsx-tok-string`, `.lsx-tok-comment`,
`.lsx-tok-number`, `.lsx-tok-constant`, `.lsx-tok-function`, `.lsx-tok-type`,
`.lsx-tok-tag`, `.lsx-tok-attribute`, `.lsx-tok-heading`, plus `.lsx-tok-bold`
and `.lsx-tok-italic`.

## Data attributes

State tokens on the `<code>`'s `data-state`, beside any `states` you pass.

| Token | Condition |
|---|---|
| `short` | `source` is at most 20 characters: the span stays on one line (`white-space: nowrap`). A longer one breaks anywhere rather than run out of a narrow column. |
