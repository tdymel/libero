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
- [Styling](styling.md): The `sx` styling builder every component takes - theme values, states, selectors, responsive, media and container queries, cascade layers and `StaticSx`.
- [Theming](theming.md): How a Libero theme is defined - one plain struct of colors, scales and per-component defaults, emitted once as CSS custom properties.
- [Performance](performance.md): What a libero component actually costs per render, measured by ablation - scope and dynamic-node counts, memoization boundaries, and how to measure it yourself.

## A11y

- [Focus Trap](focus_trap.md): Confines Tab and Shift+Tab cycling to its children, for keeping keyboard focus inside an open overlay.
- [Visually Hidden](visually_hidden.md): A `span` whose content is read by screen readers but removed from sighted layout - extra context for something ambiguous on its own.

## Data Display

- [Avatar](avatar.md): A person as a fixed square, with a fallback chain from a picture down to a person glyph, and a group that collapses its overflow into a +N chip.
- [Badge](badge.md): A short status label - one uppercase pill, sized under a control, with no role and no interaction.
- [DataList](data_list.md): A `<dl>` of term/description pairs, where one term can carry several descriptions.
- [Icon](icon.md): A sized, colored badge around an svg child, whose `currentColor` fill inherits the badge's color.
- [Image](image.md): An img with a fallback source on load error, rounded corners, and an optional click-to-zoom overlay.
- [List](list.md): An unstyled `<ul>`/`<li>` pair with themed gaps and nested indent.
- [QrCode](qr_code.md): Encodes a string as a scalable QR code, rendered as an inline SVG.
- [Table](table.md): A sortable data table built from a row type and a list of column definitions.
- [Timeline](timeline.md): An ordered list of events drawn against a rail, with an `active` index colouring the bullets and connectors up to the current one.

## Feedback

- [Alert](alert.md): A tinted surface for something the reader has to know - a title that names it, an optional icon and close button, and `role="alert"` as a default you can replace.
- [ProgressBar](progress_bar.md): A determinate or indeterminate progress bar over any `min..=max` range, with `role="progressbar"` and the raw `aria-value*` set on its root.

## Form

- [Getting Started](form_getting_started.md): How libero forms are meant to be built - specialized fields, composed parts, validators at each layer, typed paths from `#[derive(Fields)]` and binding.
- [Form](form.md): A `<form>` that validates on submit - plain `Fn(&V) -> bool` rules, typed field paths from `#[derive(Fields)]`, and a focused error summary.
- [Fieldset](fieldset.md): Several fields that form one value under a `<legend>`, with composite rules over that value that land on the fields they name.
- [TextField](text_field.md): A single-line text field with the five field slots - label, description, control, helper text and validation message.
- [Textarea](textarea.md): A multi-line text field with the five field slots, sized by `rows` and resizable by the user.
- [PasswordField](password_field.md): A password field - a `TextField` whose `type` flips between `password` and `text`, with the reveal toggle in its trailing slot.
- [PhoneField](phone_field.md): A phone field - a country picker in front of a `tel` input, whose value is an E.164 string.
- [NumberField](number_field.md): A numeric field over the caller's own number type, with steppers in its trailing slot.
- [PinField](pin_field.md): A pin, one character per cell, with auto-advance, paste spreading and an `oncomplete` that fires the moment the last cell fills.
- [Autocomplete](autocomplete.md): A text field that offers completions - the value stays a `String`, and the suggestions are drawn from any `Options` type.
- [Cascader](cascader.md): A field whose value is a path through a tree - the ids from root to leaf, picked column by column.
- [Select](select.md): A listbox over an enum with libero's own rows, in the same field frame as every other input.
- [MultiSelect](multi_select.md): A listbox over an enum that holds any number of its options, drawn as chips in the trigger.
- [TagsField](tags_field.md): A field whose value is a `Vec<String>` of free-typed tags, drawn as chips with the editor between them.
- [NativeSelect](native_select.md): A styled native `<select>` over an enum, strictly controlled by `value` plus `onchange`.
- [Combobox](combobox.md): A virtualized listbox that hangs off a caller-supplied trigger, holding no state of its own.
- [Checkbox](checkbox.md): A checkbox with its label beside the box, the field slots under both, and an indeterminate state that lives in Rust rather than in the DOM.
- [Switch](switch.md): A strictly controlled on/off toggle - a visually hidden checkbox with `role="switch"`, drawn as a track and thumb, wearing the field slots.
- [RadioGroup](radio_group.md): A group of radios over an enum, exactly one selected - one tab stop, arrow-key selection, and the question announced as the group's name.
- [SegmentedControl](segmented_control.md): A connected strip of segments over an enum, exactly one of them selected, wearing the field slots.
- [Slider](slider.md): A value dragged along a track - continuous over `f64`, or discrete over an ordered enum that derives `SliderValue`.
- [RangeSlider](range_slider.md): Two thumbs on one track for a span rather than a point - the `Slider` engine, over a pair of values.
- [ColorField](color_field.md): A text field holding a `ColorCode`, with a preview swatch, an eyedropper and a `ColorPicker` in a dropdown.
- [ColorPicker](color_picker.md): A saturation panel and a hue slider, with an optional alpha slider and preset swatches, holding one `ColorCode` that converts to any CSS form. Also documents `HueSlider`, `AlphaSlider` and `ColorSwatch`.
- [DateField](date_field.md): One text field for every date and time value, typed leniently, with the matching `DatePicker` in a dropdown. Also lists `DayField`, `TimeField`, `DateTimeField`, `DateRangeField` and `DateTimeRangeField`.
- [DatePicker](date_picker.md): One picker for every date and time value - days, months, years, times, date-times and ranges of them. Also lists `DayPicker`, `MonthPicker`, `YearPicker`, `TimePicker` and `DateRangePicker`.
- [FileField](file_field.md): Files picked from the system dialog or dropped on the control, as a one-line input or a drop surface.

## Inputs

- [ActionIcon](action_icon.md): An icon-only button - `Icon`'s sizing, color and variant system rendered as a real `button` (or a link), with a required `aria_label`.
- [Button](button.md): A clickable action, a toggle, or a router-aware link.
- [Chip](chip.md): A compact token - a tag, a filter, or a small inline action.

## Layout

- [AspectRatio](aspect_ratio.md): Enforces a width-to-height ratio on its child, cropping it to fill the box.
- [Box](box.md): The polymorphic primitive every other component is built on - renders as any tag via `component`, styled entirely through `sx`.
- [Center](center.md): Centers its child horizontally and vertically.
- [Collapse](collapse.md): Animates its children's height open and closed, over a grid row rather than a measured pixel height.
- [Container](container.md): Centers content and caps its width at a breakpoint.
- [Divider](divider.md): A horizontal or vertical rule, with an optional label sitting in the line.
- [Flex](flex.md): A flexbox container - direction, gap, align, justify and wrap, all theme-aware.
- [Float](float.md): Anchors its child to a corner or edge of the nearest positioned ancestor - a badge on an avatar, say.
- [Grid](grid.md): A named-area layout matrix - `Grid` holds the shape, a `GridZone` is a twelve-column packing container with optional masonry, and a `GridItem` takes a fraction of it.
- [Header](header.md): The page's banner landmark - a sticky, static or fixed `header` bar hosting nav and actions.
- [ImageList](image-list.md): A gallery of pictures with optional caption bars, rendered as a ul/li list over a GridZone - so cols is a span of the library's own twelve tracks and masonry is that zone's measuring engine.
- [ScrollArea](scroll_area.md): A scrollable region that fills its parent, with themed scrollbars, percent-based scroll positions, per-edge events, and row virtualization through `Virtualize`.
- [Sidebar](sidebar.md): An in-flow panel bordering one edge of its parent and scrolling its own content - a nav rail or inspector.
- [Splitter](splitter.md): Two panes divided by a draggable, keyboard-resizable divider; nest another `Splitter` in a pane for more than two.

## Navigation

- [Burger](burger.md): Three bars that morph into an X - an `ActionIcon` carrying the glyph and the three ARIA facts a nav toggle needs.
- [Anchor](anchor.md): A real link styled and sized like `Text`, router-aware through `to`.
- [Carousel](carousel.md): A scroll-snap strip of slides that knows which one it is on, with controls, indicators, optional autoplay and no JavaScript carousel library underneath.
- [NavLink](nav_link.md): A navigation list item - a link with a themed active/hover background and `aria-current`, for a sidebar or nav bar.
- [Pagination](pagination.md): A row of page controls in a named nav landmark, with an ellipsis range that never reflows as you click through it.
- [Tabs](tabs.md): One strip of tabs over an enum, with only the selected tab's panel built.
- [Tree](tree.md): A data-driven, keyboard-navigable tree view over your own node type.

## Overlay

- [Drawer](drawer.md): A dimmed, focus-trapped panel docked to one edge - `use_modal` with the docking around it, so it has the same handle, arguments and results.
- [Modal](modal.md): A modal is a hook, not a component - `use_modal` registers a render closure and returns a handle that opens it, with per-opening arguments, results and handlers.
- [Overlay](overlay.md): A full-viewport dim and blur layer with centred content - the backdrop behind a modal, or a loading screen.
- [Popover](popover.md): A popover is a hook, not a component - `use_popover` portals a box to the document root and anchors it, flipping and shifting to stay on screen.
- [Tooltip](tooltip.md): A CSS-only label that appears while its child is hovered or focused.

## Surface

- [Paper](paper.md): The library's surface - a background, a corner radius, an elevation and an optional hairline border, with no semantics of its own.
- [Dialog](dialog.md): The dialog surface - padding, radius, shadow and the `role="dialog"` wiring - which inside a modal also names and closes itself.

## Typography

- [Blockquote](blockquote.md): A quotation in a tinted frame with an accent bar, rendered as `figure` + `blockquote` + `figcaption` so the attribution sits outside the quote.
- [Code](code.md): An inline `<code>` snippet, optionally syntax highlighted.
- [CodeBlock](code_block.md): A `pre`-wrapped, multi-line code block with a line-number gutter, a copy button, a language header, optional diff rendering and line highlighting.
- [Kbd](kbd.md): A single keyboard key, rendered as a real `<kbd>` and styled entirely from the theme.
- [Mark](mark.md): A real `mark` element that highlights a chunk of text with a light tint of a theme color.
- [Text](text.md): Body copy, sized from the theme's text scale.
- [Title](title.md): A heading, `h1` through `h6`, whose visual size and semantic tag can be set apart.

---

Component source lives at `libero/src/components/<group>/<component>/` in
<https://github.com/tdymel/libero>.
