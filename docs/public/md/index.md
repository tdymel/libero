# libero docs - markdown index

Every docs page in this folder as plain markdown, one file per component:
title, lead, a full property table, and the page's own sections with code.
Start here, then fetch only the file you need.

The docs site is a Dioxus app, so its HTML carries no content until the wasm
runs - these files are the readable source of the same pages.

| Component | What it is |
|---|---|
| [Chip](chip.md) | A compact token: a plain `span`, a real checkbox (`onchange`), a button (`onclick`) or a link (`to`). Used for tags, filters and small inline actions. |

Component source lives at `libero/src/components/<group>/<component>/` in
<https://github.com/tdymel/libero>.
