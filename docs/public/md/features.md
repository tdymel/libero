# Features

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: Each component behind its own Cargo feature: the recommended setup, the default, the category features and what each component feature turns on.

Each component sits behind a Cargo feature, so an app compiles only the
components it uses. The default turns the common ones on, so an app that names
no feature still builds.

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

Off by default are the rare or heavy ones: charts, media players, the kanban
board, the rich text editor, QR codes and the brand-mark buttons among them. A
feature on by default may be there only because another default one turns it
on.

| Feature | Components | Turns on | Default | Cost |
|---|---|---|---|---|
| `button-group` | ButtonGroup |  | on |  |
| `copy` | Copy |  | on |  |
| `direction-toggle` | DirectionToggle |  | on |  |
| `repository` | Repository |  | off | The Simple Icons brand marks (pictogram-icons-simple). |
| `share` | Share |  | on |  |
| `theme-switcher` | ThemeSwitcher | menu | on |  |
| `tldr` | Tldr | chip, menu | off | The Lobe Icons brand marks (pictogram-icons-lobe). |
| `toolbar` | Toolbar, ToolbarGroup, ToolbarSeparator |  | on |  |
| `accordion` | Accordion |  | on |  |
| `audio` | Audio | slider, menu | off |  |
| `avatar` | Avatar, AvatarGroup |  | on |  |
| `badge` | Badge |  | on |  |
| `carousel` | Carousel |  | on |  |
| `charts` | LineChart, BarChart, PieChart and the other charts | skeleton | off | charts-rs, about 240 KB brotli on the web. |
| `data-list` | DataList |  | on |  |
| `image` | Image | lightbox | on |  |
| `image-list` | ImageList | image | on |  |
| `indicator` | Indicator |  | on |  |
| `kanban` | Kanban | sortable, menu | off |  |
| `list` | List |  | on |  |
| `marquee` | Marquee |  | off |  |
| `qr-code` | QrCode |  | off | The fast_qr encoder. |
| `sortable` | Sortable, use_sortable |  | on |  |
| `table` | Table, column and the table toolbar buttons | badge, progress-bar, skeleton, checkbox, native-select, select, text-field, pagination, menu, sortable, date-time | on | The largest component: its filters and editors bring the date, select and text fields. |
| `timeline` | Timeline |  | on |  |
| `video` | Video | slider, menu | off |  |
| `circular-progress` | CircularProgress |  | on |  |
| `notifications` | Notifications, use_notifications |  | on |  |
| `progress-bar` | ProgressBar | slider | on |  |
| `skeleton` | Skeleton |  | on |  |
| `autocomplete` | Autocomplete | combobox | on |  |
| `cascader` | Cascader | combobox | off |  |
| `checkbox` | Checkbox |  | on |  |
| `chip` | Chip |  | on |  |
| `color` | ColorField, ColorPicker, ColorSwatch, HueSlider, AlphaSlider | slider | off |  |
| `combobox` | Combobox, use_combobox |  | on |  |
| `date-time` | ChronoField, ChronoPicker, DateField, DatePicker, TimeField, TimePicker and the other date and time fields and pickers | tabs | on |  |
| `file-field` | FileField | chip, image-cropper, select | on |  |
| `image-cropper` | ImageCropper | slider | on |  |
| `native-select` | NativeSelect | combobox, select | on |  |
| `number-field` | NumberField |  | on |  |
| `password-field` | PasswordField | text-field | on |  |
| `phone-field` | PhoneField | combobox | off |  |
| `pin-field` | PinField |  | on |  |
| `radio` | Radio |  | on |  |
| `radio-group` | RadioGroup | radio | on |  |
| `rating` | Rating |  | on |  |
| `rich-text-editor` | RichTextEditor, RichTextView | toolbar, text-field, menu, code | off |  |
| `segmented-control` | SegmentedControl |  | on |  |
| `select` | Select, MultiSelect | combobox, chip | on |  |
| `slider` | Slider, RangeSlider |  | on |  |
| `switch` | Switch |  | on |  |
| `tags-field` | TagsField | combobox, chip, select | on |  |
| `text-field` | TextField |  | on |  |
| `textarea` | Textarea |  | on |  |
| `bottom-navigation` | BottomNavigation | nav-link | on |  |
| `burger` | Burger |  | on |  |
| `menubar` | Menubar | menu | on |  |
| `nav-link` | NavLink | tree | on |  |
| `pagination` | Pagination |  | on |  |
| `stepper` | Stepper |  | on |  |
| `tabs` | Tabs |  | on |  |
| `tree` | Tree | list | on |  |
| `drawer` | use_drawer |  | on |  |
| `floating-window` | use_floating_window | menu | off |  |
| `hover-card` | HoverCard |  | on |  |
| `lightbox` | use_lightbox | carousel | on |  |
| `menu` | Menu, use_menu |  | on |  |
| `spotlight` | use_spotlight | combobox | off |  |
| `tour` | use_tour |  | off |  |
| `code` | Code, CodeBlock | copy | on |  |
