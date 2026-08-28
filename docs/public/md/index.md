# libero - LLM documentation

This index lists every libero docs page as plain markdown, one file per
component. Each file is standalone: import line, source link, a lead, the
page's own sections with complete code, then props, theme defaults, CSS
variables and data attributes.

The docs site is a Dioxus app, so its HTML carries no content until the wasm
runs - these files are the readable source of the same pages. Start here, then
fetch only the file you need.

## About

- [Getting Started](getting_started.md): Installing libero, wrapping an app in LiberoProvider, and the feature flags a web build wants.
- [Styling](styling.md): The `sx` styling builder every component takes - theme values, states, selectors, responsive and container queries, cascade layers and `StaticSx`.
- [Theming](theming.md): How a Libero theme is defined - one plain struct of colors, scales and per-component defaults, emitted once as CSS custom properties.
- [Performance](performance.md): What a libero component actually costs per render, measured by ablation - scope and dynamic-node counts, memoization boundaries, and how to measure it yourself.

## A11y

- [Focus Trap](focus_trap.md): Confines Tab and Shift+Tab cycling to its children, for keeping keyboard focus inside an open overlay.
- [Visually Hidden](visually_hidden.md): A `span` whose content is read by screen readers but removed from sighted layout - extra context for something ambiguous on its own.

## Data Display

- [DataList](data_list.md): A `<dl>` of term/description pairs, where one term can carry several descriptions.
- [Icon](icon.md): A sized, colored badge around an svg child, whose `currentColor` fill inherits the badge's color.
- [Image](image.md): An img with a fallback source on load error, rounded corners, and an optional click-to-zoom overlay.
- [List](list.md): An unstyled `<ul>`/`<li>` pair with themed gaps and nested indent.
- [QrCode](qr_code.md): Encodes a string as a scalable QR code, rendered as an inline SVG.
- [Table](table.md): A sortable data table built from a row type and a list of column definitions.

## Form

- [Autocomplete](autocomplete.md): A text field that offers completions - the value stays a `String`, and the suggestions are drawn from any `Options` type.
- [Checkbox](checkbox.md): A checkbox with its label beside the box, the field slots under both, and an indeterminate state that lives in Rust rather than in the DOM.
- [MultiSelect](multi_select.md): A listbox over an enum that holds any number of its options, drawn as chips in the trigger.
- [NativeSelect](native_select.md): A styled native `<select>` over an enum, strictly controlled by `value` plus `onchange`.
- [NumberField](number_field.md): A numeric field over the caller's own number type, with steppers in its trailing slot.
- [PasswordField](password_field.md): A password field - a `TextField` whose `type` flips between `password` and `text`, with the reveal toggle in its trailing slot.
- [RadioGroup](radio_group.md): A group of radios over an enum, exactly one selected - one tab stop, arrow-key selection, and the question announced as the group's name.
- [Select](select.md): A listbox over an enum with libero's own rows, in the same field frame as every other input.
- [Slider](slider.md): A value dragged along a track - continuous over `f64`, or discrete over an ordered enum that derives `SliderValue`.
- [Switch](switch.md): A strictly controlled on/off toggle - a visually hidden checkbox with `role="switch"`, drawn as a track and thumb, wearing the field slots.
- [TextField](text_field.md): A single-line text field with the five field slots - label, description, control, helper text and validation message.
- [Textarea](textarea.md): A multi-line text field with the five field slots, sized by `rows` and resizable by the user.

## Inputs

- [ActionIcon](action_icon.md): An icon-only button - `Icon`'s sizing, color and variant system rendered as a real `button` (or a link), with a required `aria_label`.
- [Button](button.md): A clickable action, a toggle, or a router-aware link.
- [Chip](chip.md): A compact token - a tag, a filter, or a small inline action.
- [Combobox](combobox.md): A virtualized listbox that hangs off a caller-supplied trigger, holding no state of its own.
- [SegmentedControl](segmented_control.md): A connected strip of segments over an enum, exactly one of them selected.

## Layout

- [AspectRatio](aspect_ratio.md): Enforces a width-to-height ratio on its child, cropping it to fill the box.
- [Box](box.md): The polymorphic primitive every other component is built on - renders as any tag via `component`, styled entirely through `sx`.
- [Center](center.md): Centers its child horizontally and vertically.
- [Container](container.md): Centers content and caps its width at a breakpoint.
- [Divider](divider.md): A horizontal or vertical rule, with an optional label sitting in the line.
- [Flex](flex.md): A flexbox container - direction, gap, align, justify and wrap, all theme-aware.
- [Float](float.md): Anchors its child to a corner or edge of the nearest positioned ancestor - a badge on an avatar, say.
- [Grid](grid.md): A named-area layout matrix - `Grid` holds the shape, a `GridZone` is a twelve-column packing container with optional masonry, and a `GridItem` takes a fraction of it.
- [Header](header.md): The page's banner landmark - a sticky, static or fixed `header` bar hosting nav and actions.
- [ScrollArea](scroll_area.md): A scrollable region that fills its parent, with themed scrollbars, percent-based scroll positions, per-edge events, and row virtualization through `Virtualize`.
- [Sidebar](sidebar.md): An in-flow panel bordering one edge of its parent and scrolling its own content - a nav rail or inspector.
- [Splitter](splitter.md): Two panes divided by a draggable, keyboard-resizable divider; nest another `Splitter` in a pane for more than two.

## Navigation

- [Anchor](anchor.md): A real link styled and sized like `Text`, router-aware through `to`.
- [NavLink](nav_link.md): A navigation list item - a link with a themed active/hover background and `aria-current`, for a sidebar or nav bar.
- [Tabs](tabs.md): One strip of tabs over an enum, with only the selected tab's panel built.
- [Tree](tree.md): A data-driven, keyboard-navigable tree view over your own node type.

## Overlay

- [Drawer](drawer.md): A dimmed, focus-trapped panel docked to one edge - `use_modal` with the docking around it, so it has the same handle, arguments and results.
- [Modal](modal.md): A modal is a hook, not a component - `use_modal` registers a render closure and returns a handle that opens it, with per-opening arguments, results and handlers.
- [Overlay](overlay.md): A full-viewport dim and blur layer with centred content - the backdrop behind a modal, or a loading screen.
- [Tooltip](tooltip.md): A CSS-only label that appears while its child is hovered or focused.

## Surface

- [Dialog](dialog.md): The dialog surface - padding, radius, shadow and the `role="dialog"` wiring - which inside a modal also names and closes itself.

## Typography

- [Code](code.md): An inline `<code>` snippet, optionally syntax highlighted.
- [CodeBlock](code_block.md): A `pre`-wrapped, multi-line code block with a line-number gutter, a copy button, a language header, optional diff rendering and line highlighting.
- [Kbd](kbd.md): A single keyboard key, rendered as a real `<kbd>` and styled entirely from the theme.
- [Mark](mark.md): A real `mark` element that highlights a chunk of text with a light tint of a theme color.
- [Text](text.md): Body copy, sized from the theme's text scale.
- [Title](title.md): A heading, `h1` through `h6`, whose visual size and semantic tag can be set apart.

---

Component source lives at `libero/src/components/<group>/<component>/` in
<https://github.com/tdymel/libero>.
