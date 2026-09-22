# libero - LLM documentation

This index lists every libero docs page as plain markdown, one file per
component. Each file is standalone: import line, source link, a lead, the
page's own sections with complete code, then props, theme defaults, CSS
variables and data attributes.

The docs site is a Dioxus app, so its HTML carries no content until the wasm
runs - these files are the readable source of the same pages. Start here, then
fetch only the file you need.

## About

- [Getting started](getting_started.md): Installing libero, wrapping an app in LiberoProvider, building for the web, natively, in a desktop WebView and for Android, and the feature flags.
- [Philosophy](philosophy.md): The four principles behind libero in order of priority (developer experience, accessibility, batteries included, simple yet modern) and what we do about each.
- [Styling](styling.md): The `sx` styling builder every component takes: theme values, states, selectors, responsive, media and container queries, cascade layers and `StaticSx`.
- [Theming](theming.md): How to customize a Libero theme and use it: colors, scales, per-component defaults, light and dark pairs, and reading the active theme.
- [Localization](localization.md): The words components say on their own, how dates and numbers are written, and the reading direction: `Localization`, `Formats` and the hooks that switch them.
- [Platform](platform.md): Every platform API (elements, timers, keys, scroll, the document, the colour scheme and the clock), what each makes possible, and how to use them where a renderer lacks one.

## Layout

- [Box](box.md): The primitive every other component is built on, rendered as any tag via `component` and styled through `sx`.
- [Paper](paper.md): The library's surface, with a background, a corner radius, an elevation and an optional hairline border, and no semantics of its own.
- [Flex](flex.md): A flexbox container with theme-aware direction, gap, alignment and wrapping.
- [Grid](grid.md): A layout matrix of named areas. `Grid` holds the shape, a `GridZone` is a twelve-column container with optional masonry, and a `GridItem` takes a fraction of it.
- [Center](center.md): Centers its child horizontally and vertically.
- [Container](container.md): Centers content and caps its width at a breakpoint.
- [AspectRatio](aspect_ratio.md): Enforces a width-to-height ratio on its child, cropping it to fill the box.
- [Divider](divider.md): A horizontal or vertical rule, with an optional label sitting in the line.
- [Collapse](collapse.md): Animates its children's height open and closed, and follows the content when its height changes.
- [Float](float.md): Anchors its child to a corner or edge of the nearest positioned ancestor, like a badge on an avatar.
- [Header](header.md): The page's banner landmark, a sticky, static or fixed `header` bar for nav and actions.
- [Sidebar](sidebar.md): An in-flow panel on one edge of its parent that scrolls its own content, like a nav rail or an inspector.
- [Splitter](splitter.md): Two panes split by a divider you can drag or move with the keyboard. Nest another `Splitter` in a pane for more than two.
- [ScrollArea](scroll_area.md): A scrollable region that fills its parent, with themed scrollbars, scroll positions in percent, edge events and row virtualization through `Virtualize`.
- [Scroller](scroller.md): A horizontal strip with a hidden scrollbar and a step control over each end, shown while there is more content that way.

## Buttons

- [Button](button.md): A clickable action, a toggle, or a router-aware link.
- [ActionIcon](action_icon.md): An icon-only button, rendered as a `button` or a link, with a required `aria_label`.
- [CopyButton](copy_button.md): An icon button that copies a value to the clipboard and confirms it with a check and a spoken "Copied".
- [DirectionToggle](direction_toggle.md): An icon button that turns the app's text between left to right and right to left.
- [RepoButton](repo_button.md): A link to a GitHub or GitLab repository with its star count beside the host's icon.
- [ThemeToggle](theme_toggle.md): An icon button that steps the colour scheme through system, dark and light, with an optional theme picker beside it.
- [Tldr](tldr.md): A menu of links that ask an AI assistant (ChatGPT, Google AI, Claude, Perplexity or your own) to summarize a page.

## Form

- [Getting started](form_getting_started.md): How to build libero forms, with specialized fields, composed parts, validators at each layer, typed paths from `#[derive(Fields)]` and binding.
- [Form](form.md): A `<form>` that validates on submit, with plain `Fn(&V) -> bool` rules, typed field paths from `#[derive(Fields)]` and a focused error summary.
- [Fieldset](fieldset.md): Several fields that form one value under a `<legend>`, with rules over that value that land on the fields they name.
- [TextField](text_field.md): A single-line text field with a label, a description, helper text and a validation message.
- [Textarea](textarea.md): A multi-line text field, sized by `rows` and resizable by the user.
- [PasswordField](password_field.md): A `TextField` for secrets, with a button in its trailing slot that shows the text.
- [PhoneField](phone_field.md): A country picker in front of a `tel` input, whose value is an E.164 string.
- [NumberField](number_field.md): A numeric field over your own number type, with optional steppers in its trailing slot.
- [PinField](pin_field.md): A pin, one character per cell, with auto-advance, paste spreading and an `oncomplete` that fires the moment the last cell fills.
- [Autocomplete](autocomplete.md): A text field that offers completions from any `Options` type, while the value stays a `String`.
- [Cascader](cascader.md): A field that picks one option from a tree, one level at a time, and shows the path in the trigger.
- [Select](select.md): A listbox over an enum with rows you can draw yourself, in the same field frame as every other input.
- [MultiSelect](multi_select.md): A listbox over an enum that holds any number of its options, drawn as chips in the trigger.
- [TagsField](tags_field.md): A field whose value is a `Vec<String>` of typed tags, drawn as chips around the input.
- [NativeSelect](native_select.md): A styled native `<select>` over an enum, with the field slots.
- [Combobox](combobox.md): A listbox that hangs off a trigger you supply, holding no state of its own.
- [Checkbox](checkbox.md): A checkbox with its label beside the box, the field slots under both, and an indeterminate state.
- [Chip](chip.md): A compact token: a tag, a filter, a small action or a link.
- [Switch](switch.md): An on/off toggle drawn as a track and thumb, announced as a switch, with the field slots.
- [RadioGroup](radio_group.md): A group of radios over an enum, exactly one selected, with one tab stop, arrow-key selection and the question as the group's name.
- [SegmentedControl](segmented_control.md): A connected strip of segments over an enum, exactly one of them selected, with the field slots.
- [Slider](slider.md): A value dragged along a track, continuous over `f64` or discrete over an ordered enum that derives `SliderValue`.
- [RangeSlider](range_slider.md): Two thumbs on one track, for a span instead of a point, over the same values as `Slider`.
- [ColorField](color_field.md): A text field holding a `ColorCode`, with a preview swatch, an eyedropper and a `ColorPicker` in a dropdown.
- [ColorPicker](color_picker.md): A saturation panel and a hue slider, with an optional alpha slider and preset swatches, over one `ColorCode`. Also documents `HueSlider`, `AlphaSlider` and `ColorSwatch`.
- [ChronoField](chrono_field.md): A text field for every date and time value, typed leniently, with the matching `ChronoPicker` in a dropdown.
- [ChronoPicker](chrono_picker.md): One picker for every date and time value, from days, months and years to times, date-times and ranges of them.
- [FileField](file_field.md): Files picked from the system dialog or dropped on the control, as a one-line input or a drop surface.

## Navigation

- [Anchor](anchor.md): A real link styled and sized like `Text`, router-aware through `to`.
- [NavLink](nav_link.md): A navigation list item for a sidebar or nav bar, a link that marks the current page with `aria-current`.
- [Burger](burger.md): Three bars that morph into an X, an `ActionIcon` with the ARIA a nav toggle needs.
- [Tabs](tabs.md): One strip of tabs over an enum, with only the selected tab's panel built.
- [Menubar](menubar.md): A row of menus. Each menu is a `Menu`, and the bar is a single tab stop with one menu open at most.
- [Pagination](pagination.md): A row of page buttons in a named nav landmark, with an ellipsis that keeps the row the same width.
- [Stepper](stepper.md): The stages of a process over an enum, horizontal or vertical, with the current step's content.
- [Tree](tree.md): A data-driven, keyboard-navigable tree view over your own node type.

## Overlay

- [Overlay](overlay.md): A layer that dims and blurs the page behind it, with centred content. The backdrop behind a modal, or a loading screen.
- [Modal](modal.md): A hook that opens a render closure as a modal, with arguments and a result per opening.
- [Dialog](dialog.md): The dialog surface with a header and `role="dialog"`, which inside a modal also names and closes itself.
- [Drawer](drawer.md): A dimmed, focus-trapped panel docked to one edge, `use_modal` with the docking around it.
- [Popover](popover.md): A hook that anchors a portaled box to a trigger, flipping and shifting it to stay on screen.
- [Tooltip](tooltip.md): A label that appears while its child is hovered or focused by keyboard, portaled so nothing clips it.
- [HoverCard](hover_card.md): An interactive card that opens while its trigger is hovered or focused, a named, dismissible dialog on a paper surface.
- [Menu](menu.md): A list of commands that drops from a trigger, with groups, separators and submenus.
- [Spotlight](spotlight.md): A command palette. A modal search box over your actions, with groups, arrow-key highlight and a Ctrl/Cmd+K hotkey.
- [Lightbox](lightbox.md): A modal image viewer, `use_modal` with a gallery around it, with zoom, pan, captions and a thumbnail strip.
- [FloatingWindow](floating_window.md): A non-modal window over the page that drags, moves by keyboard and resizes from a corner, opened through a hook.

## Feedback

- [Alert](alert.md): A tinted surface for something the reader has to know, with a title, an optional icon and close button, and a role that follows its color.
- [Notifications](notifications.md): A hook plus a host. Render `Notifications {}` once, and `use_notifications()` shows messages from anywhere, as an `Alert` or as your own template over your own data.
- [Loader](loader.md): An indeterminate busy indicator, as a ring, bars or dots. It stays silent, so a status region says the wait. `Button`, `Combobox` and `FileField` show it while loading.
- [ProgressBar](progress_bar.md): A determinate or indeterminate progress bar over any `min..=max` range.
- [Skeleton](skeleton.md): A placeholder for loading content, as a standalone grey shape or a wrapper that hides the real content until it is ready.

## Data display

- [Icon](icon.md): A sized, colored box around an svg, which takes the box's color through `currentColor`.
- [Badge](badge.md): A short status label, one uppercase pill with no role and no interaction.
- [Indicator](indicator.md): A dot or a small capped count pinned to something else with a `Float`, never read out itself.
- [Avatar](avatar.md): A person as a fixed square, with a fallback chain from a picture down to a person glyph, and a group that collapses its overflow into a +N chip.
- [Image](image.md): An `<img>` with a fallback source on load error, rounded corners and an optional click-to-zoom overlay.
- [ImageList](image_list.md): A gallery of pictures with optional caption bars, laid out on a `GridZone`, so `cols` counts the library's twelve tracks.
- [Carousel](carousel.md): A strip of slides that snaps as it scrolls and knows which one it is on, with controls, indicators and optional autoplay.
- [List](list.md): A `<ul>` of `<li>` items without the browser's list styling, with themed gaps and nested indent.
- [DataList](data_list.md): A `<dl>` of term/description pairs, where one term can carry several descriptions.
- [Table](table.md): A sortable data table built from a row type and a list of column definitions.
- [Timeline](timeline.md): An ordered list of events drawn against a rail, with an `active` index colouring the bullets and connectors up to the current one.
- [Accordion](accordion.md): Sections over an enum, each a heading whose button opens its panel, with one or many open.
- [Marquee](marquee.md): Content that scrolls on its own in an endless loop, with a pause toggle and a still fallback under reduced motion.
- [QrCode](qr_code.md): Encodes a string as a scalable QR code, rendered as an inline SVG.

## Typography

- [Title](title.md): A heading, `h1` through `h6`, whose visual size and semantic tag can be set apart.
- [Text](text.md): Body copy, sized from the theme's text scale.
- [Mark](mark.md): A `mark` element that highlights text with a light tint of a theme color.
- [Code](code.md): An inline `<code>` snippet, optionally syntax highlighted.
- [Kbd](kbd.md): A single keyboard key, rendered as a real `<kbd>` and styled from the theme.
- [CodeBlock](code_block.md): A multi-line code block with line numbers, a copy button, a language header, diffs and highlighted lines.
- [Blockquote](blockquote.md): A quotation in a tinted frame with an accent bar, with the attribution outside the quote.

## Accessibility

- [Overview](accessibility.md): What libero's accessibility support covers across the library and what it does not, from on and disabled states to forced colors, which is covered only in part.
- [FocusTrap](focus_trap.md): Confines Tab and Shift+Tab cycling to its children, for keeping keyboard focus inside an open overlay.
- [VisuallyHidden](visually_hidden.md): A `span` read by screen readers but hidden from sighted layout, for extra context on something vague on its own.

## Hooks

- [Overview](hooks.md): Every public libero hook in one table, with what it is for and the page that shows it.
- [use_id](use_id.md): A process-unique id, stable for the component's lifetime, for the aria wiring between one instance's elements.
- [use_element](use_element.md): A handle to one of your component's own elements, to focus, scroll and measure it on every renderer.
- [use_focus_return](use_focus_return.md): Puts focus back on the element that opened a panel or popup once it closes, with fallbacks for a trigger that is gone.
- [use_drag](use_drag.md): Pointer plumbing for a drag. Capture, a start point, deltas against it, and one end path for release and cancel.
- [use_theme_set](use_theme_set.md): Reads and swaps the active theme set, which a theme picker is built on.
- [use_stylesheet](use_stylesheet.md): Registers a stylesheet of your own above every libero layer and returns its class.
- [use_accessibility](use_accessibility.md): Reads the reader's accessibility settings and lets an app force reduced motion.

---

Component source lives at `libero/src/components/<group>/<component>/` in
<https://github.com/tdymel/libero>.
