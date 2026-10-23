# Features

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: Every component and hook behind its own Cargo feature: the recommended setup, the default, the groups, what each feature turns on, and the other features.

Every component and every hook sits behind its own Cargo feature. Turn on only
what you use: the wasm gets smaller and the builds faster. An assistant that
manages your Cargo.toml can keep the list exact.

## Recommended

Turn `default-features` off and list the components and hooks you use. A
feature turns on the features it is built from, so `table` alone gives a
working `Table`. The compiler names a missing one: an item behind an off
feature is not in `libero`.

```toml
libero = { version = "0.3", default-features = false, features = [
    "table",
    "select",
    "date-field",
    "use-clipboard",
] }
```

## Default

The default turns on the layout components, the common components and the
common hooks: layout, text, title, anchor, kbd, mark, blockquote, button, action-icon, icon, alert, loader, tooltip, dialog, modal, form-root, fieldset, text-field, checkbox, switch, select, use-media-query, use-color-scheme, use-clipboard, use-debounce, use-timers, use-hotkeys, use-local-storage. With `default-features = false` and
no features, libero compiles no component and no hook: only the provider,
styling, theme and platform layer.

## Groups

A group feature turns on every feature of its group. `all-components` turns on
every component group, `all-hooks` every hook.

| Group | Turns on |
|---|---|
| `accessibility` | focus-trap, visually-hidden |
| `buttons` | action-icon, button, button-group, copy, direction-toggle, repository, share, theme-switcher, tldr, toolbar |
| `data-display` | accordion, audio, avatar, badge, carousel, charts, data-list, icon, image, image-list, indicator, kanban, list, marquee, pictogram, qr-code, sortable, table, timeline, video |
| `feedback` | alert, circular-progress, loader, notifications, progress-bar, skeleton |
| `form` | autocomplete, cascader, checkbox, checkbox-group, chip, color, combobox, date-field, date-picker, fieldset, file-field, form-root, image-cropper, native-select, number-field, password-field, phone-field, pin-field, radio, radio-group, rating, rich-text-editor, segmented-control, select, slider, switch, switch-group, tags-field, text-field, textarea |
| `layout` | aspect-ratio, box, center, collapse, container, divider, flex, float, grid, header, paper, scroll-area, scroller, sidebar, splitter, transition |
| `navigation` | anchor, burger, menubar, nav-link, navigation-bar, pagination, stepper, tabs, tree |
| `overlay` | dialog, drawer, floating-window, hover-card, lightbox, loading-overlay, menu, modal, overlay-backdrop, shortcut-help, spotlight, tooltip, tour |
| `typography` | blockquote, code, kbd, mark, text, title |
| `all-hooks` | use-accessibility, use-back, use-clipboard, use-color-scheme, use-countdown, use-debounce, use-direction, use-drag, use-element, use-focus-return, use-fullscreen, use-geolocation, use-history, use-hotkeys, use-icon, use-id, use-indexed-db, use-intersection, use-long-press, use-media, use-media-query, use-popover, use-push-subscription, use-share, use-local-storage, use-swipe, use-system-notification, use-timers, use-user-media |

## Components

`form-root` is Form and use_form, `overlay-backdrop` is Overlay. `table`'s date
filter, FileField's `crop` and Image's `zoomable` use another feature only when
it is on (Notes).

| Feature | Components | Turns on | Default | Notes |
|---|---|---|---|---|
| `focus-trap` | FocusTrap | use-focus-return, visually-hidden | on |  |
| `visually-hidden` | VisuallyHidden | box | on |  |
| `action-icon` | ActionIcon | button, loader, tooltip, shortcut-help, anchor, paper, pictogram, use-hotkeys, use-id | on |  |
| `button` | Button | loader, anchor | on |  |
| `button-group` | ButtonGroup | box | off |  |
| `copy` | Copy | action-icon, use-clipboard, use-timers | off |  |
| `direction-toggle` | DirectionToggle | action-icon, use-direction | off |  |
| `repository` | Repository | action-icon, use-local-storage | off | The Simple Icons brand marks (pictogram-icons-simple). |
| `share` | Share | action-icon, icon, button, use-share | off |  |
| `theme-switcher` | ThemeSwitcher | menu, action-icon, use-color-scheme | off |  |
| `tldr` | Tldr | chip, menu, icon, action-icon | off | The Lobe Icons brand marks (pictogram-icons-lobe). |
| `toolbar` | Toolbar, ToolbarGroup, ToolbarSeparator | box, use-hotkeys | off |  |
| `accordion` | Accordion | collapse, use-icon, use-id | off |  |
| `audio` | Audio | slider, menu, button, modal, action-icon, shortcut-help, use-fullscreen, use-media | off |  |
| `avatar` | Avatar, AvatarGroup | tooltip, use-icon, use-id, visually-hidden | off |  |
| `badge` | Badge | box | off |  |
| `carousel` | Carousel | action-icon, scroll-area | off |  |
| `charts` | LineChart, BarChart, PieChart and the other charts | skeleton, mark, focus-trap, use-accessibility, use-popover, use-debounce, use-id, use-icon | off | charts-rs, about 240 KB brotli on the web. |
| `data-list` | DataList | box | off |  |
| `icon` | Icon | pictogram, visually-hidden | on |  |
| `image` | Image | box | off | Its zoomable prop needs lightbox on as well; without it the picture renders unzoomable. |
| `image-list` | ImageList | image, grid | off |  |
| `indicator` | Indicator | box | off |  |
| `kanban` | Kanban | sortable, menu, focus-trap | off |  |
| `list` | List | box | off |  |
| `marquee` | Marquee | action-icon | off |  |
| `pictogram` | Pictogram | use-icon | on |  |
| `qr-code` | QrCode | box | off | The fast_qr encoder. |
| `sortable` | Sortable, use_sortable | use-drag, use-icon, use-id, use-media-query, use-timers, visually-hidden | off |  |
| `table` | Table, column and the table toolbar buttons | badge, progress-bar, skeleton, checkbox, native-select, select, text-field, pagination, menu, sortable, icon, action-icon, button, collapse, use-debounce | off | The largest component: its filters and editors bring the select and text fields. With date-field on, a date filter uses DateField. |
| `timeline` | Timeline | visually-hidden | off |  |
| `video` | Video | slider, menu, button, modal, action-icon, shortcut-help, use-fullscreen, use-media | off |  |
| `alert` | Alert | action-icon | on |  |
| `circular-progress` | CircularProgress | box, use-accessibility | off |  |
| `loader` | Loader | box | on |  |
| `notifications` | Notifications, use_notifications | alert, button, float | off |  |
| `progress-bar` | ProgressBar | slider | off |  |
| `skeleton` | Skeleton | box | off |  |
| `autocomplete` | Autocomplete | combobox, action-icon | off |  |
| `cascader` | Cascader | combobox, action-icon, use-media-query | off |  |
| `checkbox` | Checkbox | use-icon, use-id, visually-hidden | on |  |
| `checkbox-group` | CheckboxGroup | checkbox | off |  |
| `chip` | Chip | action-icon, anchor | on |  |
| `color` | ColorField, ColorPicker, ColorSwatch, HueSlider, AlphaSlider | slider, action-icon, use-media-query, use-swipe, use-timers | off |  |
| `combobox` | Combobox, use_combobox | loader, paper, scroll-area, use-id, use-popover, visually-hidden, use-icon, use-swipe, use-media-query, use-timers | on |  |
| `date-picker` | ChronoPicker, DatePicker, DateRangePicker, MonthPicker, YearPicker, TimePicker, DateRange | tabs, action-icon, use-drag, use-media-query, use-swipe, use-timers | off |  |
| `date-field` | ChronoField, DateField, DateRangeField, DateTimeField, DateTimeRangeField, TimeField | date-picker, button | off | The typed text's formatting and parsing. date-time turns on both date features. |
| `fieldset` | Fieldset | form-root | on |  |
| `file-field` | FileField | chip, select, alert, modal, action-icon, button, dialog, loader | off | Its crop prop needs image-cropper on as well; without it the file passes uncropped. |
| `form-root` | Form, use_form | alert | on |  |
| `image-cropper` | ImageCropper | slider, action-icon | off |  |
| `native-select` | NativeSelect | combobox, select | off |  |
| `number-field` | NumberField | action-icon | off |  |
| `password-field` | PasswordField | text-field, action-icon | off |  |
| `phone-field` | PhoneField | combobox | off |  |
| `pin-field` | PinField | use-icon, visually-hidden, use-id | off |  |
| `radio` | Radio | use-accessibility, use-icon, visually-hidden, use-id | off |  |
| `radio-group` | RadioGroup | radio | off |  |
| `rating` | Rating | use-icon, visually-hidden, use-id, use-drag | off |  |
| `rich-text-editor` | RichTextEditor, RichTextView | toolbar, text-field, menu, code, mark, modal, action-icon, button, dialog, shortcut-help, text, flex, use-history | off |  |
| `segmented-control` | SegmentedControl | button, use-id | off |  |
| `select` | Select, MultiSelect | combobox, chip, action-icon | on |  |
| `slider` | Slider, RangeSlider | tooltip, use-icon, visually-hidden, use-id, use-drag | off |  |
| `switch` | Switch | use-accessibility, use-icon, visually-hidden, use-id | on |  |
| `switch-group` | SwitchGroup | switch | off |  |
| `tags-field` | TagsField | combobox, chip, select, action-icon | off |  |
| `text-field` | TextField | use-icon, visually-hidden, use-id | on |  |
| `textarea` | Textarea | use-debounce, use-icon, visually-hidden, use-id | off |  |
| `aspect-ratio` | AspectRatio | box | on |  |
| `box` | Box | use-element | on |  |
| `center` | Center | box | on |  |
| `collapse` | Collapse | box, use-timers | on |  |
| `container` | Container | box | on |  |
| `divider` | Divider | use-id, use-icon, visually-hidden | on |  |
| `flex` | Flex | box | on |  |
| `float` | Float | box | on |  |
| `grid` | Grid, GridItem, GridZone | box | on |  |
| `header` | Header | paper, use-id, use-icon, visually-hidden | on |  |
| `paper` | Paper | box | on |  |
| `scroll-area` | ScrollArea, use_scroll_area | box, use-drag | on |  |
| `scroller` | Scroller, use_scroller | scroll-area, use-icon, use-id, visually-hidden | on |  |
| `sidebar` | Sidebar | scroll-area | on |  |
| `splitter` | Splitter | use-drag, use-id, use-icon, visually-hidden | on |  |
| `transition` | Transition | box, use-timers | on |  |
| `anchor` | Anchor | use-icon, visually-hidden | on |  |
| `burger` | Burger | action-icon | off |  |
| `menubar` | Menubar | menu | off |  |
| `nav-link` | NavLink | tree, anchor, collapse, use-id | off |  |
| `navigation-bar` | NavigationBar | nav-link, float | off |  |
| `pagination` | Pagination | action-icon | off |  |
| `stepper` | Stepper | collapse, use-icon, use-id, visually-hidden | off |  |
| `tabs` | Tabs | use-id, use-icon, visually-hidden | off |  |
| `tree` | Tree | list, icon, use-popover, checkbox | off |  |
| `dialog` | Dialog | action-icon, title | on |  |
| `drawer` | use_drawer | dialog, modal, float, use-swipe, use-timers | off |  |
| `floating-window` | use_floating_window | menu, action-icon, button, title, float, use-drag | off |  |
| `hover-card` | HoverCard | tooltip, focus-trap, paper, scroll-area | off |  |
| `lightbox` | use_lightbox | carousel, modal, action-icon, dialog | off |  |
| `menu` | Menu, use_menu | anchor, kbd, divider, paper, use-popover, use-timers | off |  |
| `modal` | Modal, use_modal | overlay-backdrop, focus-trap, use-back | on |  |
| `overlay-backdrop` | Overlay | box | on |  |
| `loading-overlay` | LoadingOverlay | overlay-backdrop, loader, visually-hidden | off |  |
| `shortcut-help` | ShortcutHelp | dialog, kbd | on |  |
| `spotlight` | use_spotlight | combobox, dialog, modal, kbd, loader | off |  |
| `tooltip` | Tooltip | use-timers, box, use-popover | on |  |
| `tour` | use_tour | dialog, button, focus-trap, use-accessibility, use-local-storage, use-back | off |  |
| `blockquote` | Blockquote | box | on |  |
| `code` | Code, CodeBlock | copy | off |  |
| `kbd` | Kbd | box | on |  |
| `mark` | Mark, Highlight | box | on |  |
| `text` | Text | box | on |  |
| `title` | Title | text | on |  |

## Hooks

One feature per hook module. The hooks of a component (use_modal, use_drawer,
use_lightbox, use_sortable, ...) come with its feature.

| Feature | Hooks | Turns on | Default | Notes |
|---|---|---|---|---|
| `use-accessibility` | use_accessibility |  | on |  |
| `use-back` | use_back | use-popover | on |  |
| `use-clipboard` | use_clipboard |  | on |  |
| `use-color-scheme` | use_color_scheme |  | on |  |
| `use-countdown` | use_countdown, format_duration |  | off |  |
| `use-debounce` | use_debounced_value, use_debounced_callback, use_throttled_value, use_throttled_callback | use-timers | on |  |
| `use-direction` | use_direction |  | off |  |
| `use-drag` | use_drag | use-element | on |  |
| `use-element` | use_element |  | on |  |
| `use-focus-return` | use_focus_return | use-element | on |  |
| `use-fullscreen` | use_fullscreen | use-back | off |  |
| `use-geolocation` | use_geolocation |  | off |  |
| `use-history` | use_history | use-timers | off |  |
| `use-hotkeys` | use_hotkeys | use-popover | on |  |
| `use-icon` | use_icon |  | on |  |
| `use-id` | use_id |  | on |  |
| `use-indexed-db` | use_indexed_db | use-local-storage | off |  |
| `use-intersection` | use_intersection, use_in_viewport | use-element | off |  |
| `use-long-press` | use_long_press | use-timers | off |  |
| `use-media` | use_media | use-element | off |  |
| `use-media-query` | use_media_query, use_is_mobile |  | on |  |
| `use-popover` | use_popover | use-focus-return | on |  |
| `use-push-subscription` | use_push_subscription |  | off |  |
| `use-share` | use_share |  | off |  |
| `use-local-storage` | use_local_storage, use_session_storage |  | on |  |
| `use-swipe` | use_swipe, use_edge_swipe | use-direction, use-drag | off |  |
| `use-system-notification` | use_system_notification |  | off |  |
| `use-timers` | use_timeout, use_interval |  | on |  |
| `use-user-media` | use_user_media, use_user_media_devices |  | off |  |

## Other features

| Feature | Does |
|---|---|
| `native` | Reaches elements through Blitz under dioxus-native: subtree queries, focus, the clipboard, file dialogs. |
| `desktop` | The desktop WebView on Linux: system notifications over the session bus, the save dialog. |
| `fcm` | use_fcm, Firebase Cloud Messaging on Android. Ships firebase-messaging into the app. |
| `collation` | Table sorts text by the app's language off the web too, as the browser's Intl.Collator does. |
| `full-polymorphism` | The 28 rarer HTML tags for a component's component prop (head, iframe, video, template, ...). |
| `icons-bootstrap, icons-material, icons-phosphor, icons-tabler` | One pictogram set each for IconProvider. |
| `code-lang-*` | One grammar each for Code and CodeBlock, from code-lang-bash to code-lang-yaml. |
| `held-clock, fake-file-dialog` | Test harnesses only: a held timer clock and a scripted file dialog. Never in an app. |
