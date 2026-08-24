# libero - LLM documentation

This index lists every libero docs page as plain markdown, one file per
component. Each file is standalone: import line, source link, a lead, the
page's own sections with complete code, then props, theme defaults, CSS
variables and data attributes.

The docs site is a Dioxus app, so its HTML carries no content until the wasm
runs - these files are the readable source of the same pages. Start here, then
fetch only the file you need.

## Inputs

- [Chip](chip.md): A compact token - a plain `span`, a real checkbox (`onchange`), a button (`onclick`) or a link (`to`). Used for tags, filters and small inline actions.

---

Component source lives at `libero/src/components/<group>/<component>/` in
<https://github.com/tdymel/libero>.
