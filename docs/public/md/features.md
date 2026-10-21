# Features

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: Each component behind its own Cargo feature: the recommended setup, the default, the category features and what each component feature turns on.

Each component sits behind a Cargo feature, so an app compiles only the
components it uses. The default turns none on: libero alone gives the
always-compiled core below. Add one feature per component, or all-components to
try everything.

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

The hooks (bar use_drawer, use_floating_window, use_lightbox and the sortable hooks, which come with their component), the layout components (Box, Flex, Grid, Paper, ScrollArea, Header, Sidebar, Splitter and the rest), Text, Title, Kbd, Mark, Blockquote, Button, ActionIcon, Icon, Pictogram, Alert, Loader, Anchor, Dialog, Modal, Overlay, Tooltip, ShortcutHelp, use_popover, FocusTrap, VisuallyHidden, Form and Fieldset. Most components are built from them.

## Categories

A category feature turns on every component feature of its category, the ones
off by default too. `all-components` turns on every category.

| Category | Turns on |
|---|---|
| `buttons` | button-group, copy, direction-toggle, repository, share, theme-switcher, tldr, toolbar |
| `data-display` | accordion, audio, avatar, badge, carousel, charts, data-list, image, image-list, indicator, kanban, list, marquee, qr-code, sortable, table, timeline, video |
| `feedback` | circular-progress, notifications, progress-bar, skeleton |
| `form` | autocomplete, cascader, checkbox, chip, color, combobox, date-time, file-field, image-cropper, native-select, number-field, password-field, phone-field, pin-field, radio, radio-group, rating, rich-text-editor, segmented-control, select, slider, switch, tags-field, text-field, textarea |
| `navigation` | bottom-navigation, burger, menubar, nav-link, pagination, stepper, tabs, tree |
| `overlay` | drawer, floating-window, hover-card, lightbox, menu, spotlight, tour |
| `typography` | code |

## Components

Every component feature is off by default. Turning one on turns on the features
in its Turns on column.

Three components use another feature only when it is on, and work without it:
`table`'s date filter picks from a DateField with `date-time` and from the
native date input without; FileField's `crop` needs `image-cropper` and Image's
`zoomable` needs `lightbox`, else a debug build warns and the file passes
uncropped, the picture unzoomable.

Cost is the wasm an app gains in a release build when it uses the feature's
first component, over a bare Text and Button app of 2.7 MB (before
wasm-bindgen). Rows share code, so they do not add up. A feature nobody uses
costs build time, not wasm. Debug build from a cold cache, then after a libero
edit: no features 23 s and 3 s, `select` 25 s and 4 s, the former default set of 62 features 59 s and 13 s, `all-components` 105 s and 20 s.

| Feature | Components | Turns on | Default | Adds | Cost |
|---|---|---|---|---|---|
| `button-group` | ButtonGroup |  | off |  | +20 KB |
| `copy` | Copy |  | off |  | +530 KB |
| `direction-toggle` | DirectionToggle |  | off |  | +500 KB |
| `repository` | Repository |  | off | The Simple Icons brand marks (pictogram-icons-simple). | +610 KB |
| `share` | Share |  | off |  | +590 KB |
| `theme-switcher` | ThemeSwitcher | menu | off |  | +1.2 MB |
| `tldr` | Tldr | chip, menu | off | The Lobe Icons brand marks (pictogram-icons-lobe). | +1.2 MB |
| `toolbar` | Toolbar, ToolbarGroup, ToolbarSeparator |  | off |  | +290 KB |
| `accordion` | Accordion |  | off |  | +290 KB |
| `audio` | Audio | slider, menu | off |  | +2.1 MB |
| `avatar` | Avatar, AvatarGroup |  | off |  | +100 KB |
| `badge` | Badge |  | off |  | +10 KB |
| `carousel` | Carousel |  | off |  | +1.3 MB |
| `charts` | LineChart, BarChart, PieChart and the other charts | skeleton | off | charts-rs, about 240 KB brotli on the web. | +1.6 MB |
| `data-list` | DataList |  | off |  | +7 KB |
| `image` | Image |  | off | Its zoomable prop needs lightbox on as well; without it the picture renders unzoomable. | +2.0 MB |
| `image-list` | ImageList | image | off |  | +130 KB |
| `indicator` | Indicator |  | off |  | +10 KB |
| `kanban` | Kanban | sortable, menu | off |  | +900 KB |
| `list` | List |  | off |  | +10 KB |
| `marquee` | Marquee |  | off |  | +570 KB |
| `qr-code` | QrCode |  | off | The fast_qr encoder. | +110 KB |
| `sortable` | Sortable, use_sortable |  | off |  | +580 KB |
| `table` | Table, column and the table toolbar buttons | badge, progress-bar, skeleton, checkbox, native-select, select, text-field, pagination, menu, sortable | off | The largest component: its filters and editors bring the select and text fields. With date-time on, a date filter uses DateField. | +6.5 MB |
| `timeline` | Timeline |  | off |  | +40 KB |
| `video` | Video | slider, menu | off |  | +2.7 MB |
| `circular-progress` | CircularProgress |  | off |  | +30 KB |
| `notifications` | Notifications, use_notifications |  | off |  | +410 KB |
| `progress-bar` | ProgressBar | slider | off |  | +50 KB |
| `skeleton` | Skeleton |  | off |  | +8 KB |
| `autocomplete` | Autocomplete | combobox | off |  | +1.7 MB |
| `cascader` | Cascader | combobox | off |  | +1.5 MB |
| `checkbox` | Checkbox |  | off |  | +280 KB |
| `chip` | Chip |  | off |  | +210 KB |
| `color` | ColorField, ColorPicker, ColorSwatch, HueSlider, AlphaSlider | slider | off |  | +1.6 MB |
| `combobox` | Combobox, use_combobox |  | off |  | +1.1 MB |
| `date-time` | ChronoField, ChronoPicker, DateField, DatePicker, TimeField, TimePicker and the other date and time fields and pickers | tabs | off |  | +1.1 MB |
| `file-field` | FileField | chip, select | off | Its crop prop needs image-cropper on as well; without it the file passes uncropped. | +2.4 MB |
| `image-cropper` | ImageCropper | slider | off |  | +1.4 MB |
| `native-select` | NativeSelect | combobox, select | off |  | +400 KB |
| `number-field` | NumberField |  | off |  | +900 KB |
| `password-field` | PasswordField | text-field | off |  | +760 KB |
| `phone-field` | PhoneField | combobox | off |  | +1.5 MB |
| `pin-field` | PinField |  | off |  | +310 KB |
| `radio` | Radio |  | off |  | +230 KB |
| `radio-group` | RadioGroup | radio | off |  | +330 KB |
| `rating` | Rating |  | off |  | +380 KB |
| `rich-text-editor` | RichTextEditor, RichTextView | toolbar, text-field, menu, code | off |  | +3.1 MB |
| `segmented-control` | SegmentedControl |  | off |  | +340 KB |
| `select` | Select, MultiSelect | combobox, chip | off |  | +1.8 MB |
| `slider` | Slider, RangeSlider |  | off |  | +1.0 MB |
| `switch` | Switch |  | off |  | +300 KB |
| `tags-field` | TagsField | combobox, chip, select | off |  | +1.7 MB |
| `text-field` | TextField |  | off |  | +310 KB |
| `textarea` | Textarea |  | off |  | +390 KB |
| `bottom-navigation` | BottomNavigation | nav-link | off |  | +30 KB |
| `burger` | Burger |  | off |  | +500 KB |
| `menubar` | Menubar | menu | off |  | +870 KB |
| `nav-link` | NavLink | tree | off |  | +220 KB |
| `pagination` | Pagination |  | off |  | +560 KB |
| `stepper` | Stepper |  | off |  | +410 KB |
| `tabs` | Tabs |  | off |  | +210 KB |
| `tree` | Tree | list | off |  | +300 KB |
| `drawer` | use_drawer |  | off |  | +880 KB |
| `floating-window` | use_floating_window | menu | off |  | +1.5 MB |
| `hover-card` | HoverCard |  | off |  | +570 KB |
| `lightbox` | use_lightbox | carousel | off |  | +1.9 MB |
| `menu` | Menu, use_menu |  | off |  | +800 KB |
| `spotlight` | use_spotlight | combobox | off |  | +1.5 MB |
| `tour` | use_tour |  | off |  | +1.1 MB |
| `code` | Code, CodeBlock | copy | off |  | +120 KB |
