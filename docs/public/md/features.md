# Features

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: Each component behind its own Cargo feature: the recommended setup, the default, the category features and what each component feature turns on.

Each component sits behind a Cargo feature, so an app compiles only the
components it uses. The default turns none on: libero alone compiles only the
layout components and the shared pieces below. Add one feature per component,
Text and Button included, or all-components to try everything.

## Recommended

Turn `default-features` off and list the components you use, one feature each.
A component's feature turns on the features it is built from, so `table` alone
gives a working `Table`. The compiler names a missing one: a component behind
an off feature is not in `libero::components`.

```toml
libero = { version = "0.3", default-features = false, features = [
    "table",
    "select",
    "date-time",
] }
```

Assistants writing code with libero should do the same: start from
`default-features = false` and add the feature of each component they use.

## Always compiled

The hooks (bar use_modal, use_drawer, use_floating_window, use_lightbox and the sortable hooks, which come with their component), the layout components (Box, Flex, Grid, Paper, ScrollArea, Header, Sidebar, Splitter and the rest), Pictogram and the icon provider, use_popover, FocusTrap and VisuallyHidden. Every component is built from them.

## Categories

A category feature turns on every component feature of its category, the ones
off by default too. `all-components` turns on every category.

| Category | Turns on |
|---|---|
| `buttons` | action-icon, button, button-group, copy, direction-toggle, repository, share, theme-switcher, tldr, toolbar |
| `data-display` | accordion, audio, avatar, badge, carousel, charts, data-list, icon, image, image-list, indicator, kanban, list, marquee, qr-code, sortable, table, timeline, video |
| `feedback` | alert, circular-progress, loader, notifications, progress-bar, skeleton |
| `form` | autocomplete, cascader, checkbox, chip, color, combobox, date-time, fieldset, file-field, form-root, image-cropper, native-select, number-field, password-field, phone-field, pin-field, radio, radio-group, rating, rich-text-editor, segmented-control, select, slider, switch, tags-field, text-field, textarea |
| `navigation` | anchor, burger, menubar, nav-link, navigation-bar, pagination, stepper, tabs, tree |
| `overlay` | dialog, drawer, floating-window, hover-card, lightbox, menu, modal, overlay-backdrop, shortcut-help, spotlight, tooltip, tour |
| `typography` | blockquote, code, kbd, mark, text, title |

## Components

Every component feature is off by default. Turning one on turns on the features
in its Turns on column.

Two names differ from their component's, as the category took it: `form-root`
is Form and use_form, `overlay-backdrop` is Overlay.

Three components use another feature only when it is on, and work without it:
`table`'s date filter picks from a DateField with `date-time` and from the
native date input without; FileField's `crop` needs `image-cropper` and Image's
`zoomable` needs `lightbox`, else a debug build warns and the file passes
uncropped, the picture unzoomable.

Cost is the wasm an app gains in a release build when it uses the feature's
first component, over a bare Text and Button app of 2.7 MB (before
wasm-bindgen); base marks what that app already has. Rows share code, so they do not add up. A feature nobody uses
costs build time, not wasm. Debug build from a cold cache, then after a libero
edit: no features 23 s and 3 s, `select` 25 s and 4 s, the former default set of 62 features 59 s and 13 s, `all-components` 105 s and 20 s.

| Feature | Components | Turns on | Default | Adds | Cost |
|---|---|---|---|---|---|
| `action-icon` | ActionIcon | button, loader, tooltip, shortcut-help, anchor | off |  | +480 KB |
| `button` | Button | loader, anchor | off |  | base |
| `button-group` | ButtonGroup |  | off |  | +20 KB |
| `copy` | Copy | action-icon | off |  | +530 KB |
| `direction-toggle` | DirectionToggle | action-icon | off |  | +500 KB |
| `repository` | Repository | action-icon | off | The Simple Icons brand marks (pictogram-icons-simple). | +610 KB |
| `share` | Share | action-icon, icon, button | off |  | +590 KB |
| `theme-switcher` | ThemeSwitcher | menu, action-icon | off |  | +1.2 MB |
| `tldr` | Tldr | chip, menu, icon, action-icon | off | The Lobe Icons brand marks (pictogram-icons-lobe). | +1.2 MB |
| `toolbar` | Toolbar, ToolbarGroup, ToolbarSeparator |  | off |  | +290 KB |
| `accordion` | Accordion |  | off |  | +290 KB |
| `audio` | Audio | slider, menu, button, modal, action-icon, shortcut-help | off |  | +2.1 MB |
| `avatar` | Avatar, AvatarGroup | tooltip | off |  | +100 KB |
| `badge` | Badge |  | off |  | +10 KB |
| `carousel` | Carousel | action-icon | off |  | +1.3 MB |
| `charts` | LineChart, BarChart, PieChart and the other charts | skeleton, mark | off | charts-rs, about 240 KB brotli on the web. | +1.6 MB |
| `data-list` | DataList |  | off |  | +7 KB |
| `icon` | Icon |  | off |  | +24 KB |
| `image` | Image |  | off | Its zoomable prop needs lightbox on as well; without it the picture renders unzoomable. | +2.0 MB |
| `image-list` | ImageList | image | off |  | +130 KB |
| `indicator` | Indicator |  | off |  | +10 KB |
| `kanban` | Kanban | sortable, menu | off |  | +900 KB |
| `list` | List |  | off |  | +10 KB |
| `marquee` | Marquee | action-icon | off |  | +570 KB |
| `qr-code` | QrCode |  | off | The fast_qr encoder. | +110 KB |
| `sortable` | Sortable, use_sortable |  | off |  | +580 KB |
| `table` | Table, column and the table toolbar buttons | badge, progress-bar, skeleton, checkbox, native-select, select, text-field, pagination, menu, sortable, icon, action-icon, button | off | The largest component: its filters and editors bring the select and text fields. With date-time on, a date filter uses DateField. | +6.5 MB |
| `timeline` | Timeline |  | off |  | +40 KB |
| `video` | Video | slider, menu, button, modal, action-icon, shortcut-help | off |  | +2.7 MB |
| `alert` | Alert | action-icon | off |  | +550 KB |
| `circular-progress` | CircularProgress |  | off |  | +30 KB |
| `loader` | Loader |  | off |  | base |
| `notifications` | Notifications, use_notifications | alert, button | off |  | +410 KB |
| `progress-bar` | ProgressBar | slider | off |  | +50 KB |
| `skeleton` | Skeleton |  | off |  | +8 KB |
| `autocomplete` | Autocomplete | combobox, action-icon | off |  | +1.7 MB |
| `cascader` | Cascader | combobox, action-icon | off |  | +1.5 MB |
| `checkbox` | Checkbox |  | off |  | +280 KB |
| `chip` | Chip | action-icon, anchor | off |  | +210 KB |
| `color` | ColorField, ColorPicker, ColorSwatch, HueSlider, AlphaSlider | slider, action-icon | off |  | +1.6 MB |
| `combobox` | Combobox, use_combobox | loader | off |  | +1.1 MB |
| `date-time` | ChronoField, ChronoPicker, DateField, DatePicker, TimeField, TimePicker and the other date and time fields and pickers | tabs, action-icon | off |  | +1.1 MB |
| `fieldset` | Fieldset | form-root | off |  | +210 KB |
| `form-root` | Form, use_form | alert | off |  | +830 KB |
| `file-field` | FileField | chip, select, alert, modal, action-icon, button, dialog, loader | off | Its crop prop needs image-cropper on as well; without it the file passes uncropped. | +2.4 MB |
| `image-cropper` | ImageCropper | slider, action-icon | off |  | +1.4 MB |
| `native-select` | NativeSelect | combobox, select | off |  | +400 KB |
| `number-field` | NumberField | action-icon | off |  | +900 KB |
| `password-field` | PasswordField | text-field, action-icon | off |  | +760 KB |
| `phone-field` | PhoneField | combobox | off |  | +1.5 MB |
| `pin-field` | PinField |  | off |  | +310 KB |
| `radio` | Radio |  | off |  | +230 KB |
| `radio-group` | RadioGroup | radio | off |  | +330 KB |
| `rating` | Rating |  | off |  | +380 KB |
| `rich-text-editor` | RichTextEditor, RichTextView | toolbar, text-field, menu, code, mark, modal, action-icon, button, dialog, shortcut-help, text | off |  | +3.1 MB |
| `segmented-control` | SegmentedControl | button | off |  | +340 KB |
| `select` | Select, MultiSelect | combobox, chip, action-icon | off |  | +1.8 MB |
| `slider` | Slider, RangeSlider | tooltip | off |  | +1.0 MB |
| `switch` | Switch |  | off |  | +300 KB |
| `tags-field` | TagsField | combobox, chip, select, action-icon | off |  | +1.7 MB |
| `text-field` | TextField |  | off |  | +310 KB |
| `textarea` | Textarea |  | off |  | +390 KB |
| `anchor` | Anchor |  | off |  | +5 KB |
| `burger` | Burger | action-icon | off |  | +500 KB |
| `menubar` | Menubar | menu | off |  | +870 KB |
| `nav-link` | NavLink | tree, anchor | off |  | +220 KB |
| `navigation-bar` | NavigationBar | nav-link | off |  | +30 KB |
| `pagination` | Pagination | action-icon | off |  | +560 KB |
| `stepper` | Stepper |  | off |  | +410 KB |
| `tabs` | Tabs |  | off |  | +210 KB |
| `tree` | Tree | list, icon | off |  | +300 KB |
| `dialog` | Dialog | action-icon, title | off |  | +590 KB |
| `drawer` | use_drawer | dialog, modal | off |  | +880 KB |
| `floating-window` | use_floating_window | menu, action-icon, button, title | off |  | +1.5 MB |
| `hover-card` | HoverCard | tooltip | off |  | +570 KB |
| `lightbox` | use_lightbox | carousel, modal, action-icon, dialog | off |  | +1.9 MB |
| `menu` | Menu, use_menu | anchor, kbd | off |  | +800 KB |
| `modal` | Modal, use_modal | overlay-backdrop | off |  | +180 KB |
| `overlay-backdrop` | Overlay |  | off |  | +9 KB |
| `shortcut-help` | ShortcutHelp | dialog, kbd | off |  | +660 KB |
| `spotlight` | use_spotlight | combobox, dialog, modal, kbd, loader | off |  | +1.5 MB |
| `tooltip` | Tooltip |  | off |  | +390 KB |
| `tour` | use_tour | dialog, button | off |  | +1.1 MB |
| `blockquote` | Blockquote |  | off |  | +23 KB |
| `code` | Code, CodeBlock | copy | off |  | +120 KB |
| `kbd` | Kbd |  | off |  | +6 KB |
| `mark` | Mark |  | off |  | +4 KB |
| `text` | Text |  | off |  | base |
| `title` | Title | text | off |  | +7 KB |
