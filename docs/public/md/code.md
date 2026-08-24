# Code

Crate: `libero`
Import: `use libero::components::Code;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/code/code.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An inline `<code>` snippet, optionally syntax highlighted.

A `<code>` element for a snippet inside a sentence. Content is `source`, a
plain string; add `language` to syntax-highlight it. For a multi-line,
`<pre>`-wrapped block with line numbers, a copy button and diffs, reach for
[code_block.md](code_block.md) instead. The font and the syntax token colors
come from `Theme.code` and can be overridden per-app.

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

Without `language` the source renders as plain, unhighlighted text - that is
the default.

## Languages

Libero ships grammars for 30 languages, each behind its own `code-lang-*`
feature so a build only pays for what it highlights. The default set covers
`rust`, `bash`, `css` and a few more; enable the rest as you need them. An
unrecognized name - or a recognized one whose feature is off - falls back to
plain, unhighlighted text, which is also what leaving `language` off does.

The full set, spelled as the `code-lang-*` feature suffix (`language` also
takes aliases the feature names do not, such as `rs`, `py`, `c#`):

```
bash, c, cpp, csharp, css, dart, go, graphql, haskell, html, java,
javascript, json, kotlin, lua, markdown, objective-c, perl, php,
powershell, python, r, ruby, rust, scala, sql, swift, toml, typescript, yaml
```

The grammars are hand-ported from [Prism](https://prismjs.com), as is the
tokenizer that runs them. These 30 are where we started, not a closed set - if
you need one Prism has and we do not, it can be ported the same way.

## Accessibility

`Code` renders a real `<code>` element, so assistive technology announces the
content as code. Highlighting adds only `<span>`s with color classes, which
carry no semantics of their own, so a highlighted and an unhighlighted snippet
read the same.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `source` | `String` | required | The text to render, highlighted when `language` names a grammar this build compiles in. |
| `language` | `Language` | - | Unrecognized values fall back to no highlighting rather than a guess. |

Like every component, `Code` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`CodeDefaults` on the theme - the monospace stack plus one color per token
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

`Code` sets no state tokens of its own; a `states` prop is passed through
unchanged.
