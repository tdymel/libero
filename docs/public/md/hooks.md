# Hooks

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: Every public libero hook in one table, with what it is for and the page that shows it.

Libero's components are built from these hooks, and they are public for yours.
They are positional like every dioxus hook, so call them unconditionally, in the
same order every render. The primitives and the theme-set and stylesheet hooks
have a page of their own. The rest are shown on the component or guide page they
belong to.

## Every public hook

| Hook | What it is for | Page |
|---|---|---|
| `use_id` | A unique id for the aria wiring between one instance's elements. | [Unique ID](use_id.md) |
| `use_element` | A handle to one of your component's elements, to focus, scroll or measure it. | [Element handle](use_element.md) |
| `use_focus_return` | Puts focus back on the trigger when a panel closes. | [Focus return](use_focus_return.md) |
| `use_drag` | Pointer capture and deltas for a drag. | [Drag](use_drag.md) |
| `use_intersection` | How much of an element is visible inside a root. | [Intersection](use_intersection.md) |
| `use_in_viewport` | Whether an element is in the viewport. | [Intersection](use_intersection.md) |
| `use_long_press` | Handlers that call back once a pointer stays down. | [Long press](use_long_press.md) |
| `use_timeout` | Runs a callback once, a while after you start it. | [Timers](use_timers.md) |
| `use_interval` | Runs a callback repeatedly, with start, stop and toggle. | [Timers](use_timers.md) |
| `use_debounced_value` | A signal that follows another once it stops changing. | [Debounce and throttle](use_debounce.md) |
| `use_debounced_callback` | A callback that runs after its last call, with the last argument. | [Debounce and throttle](use_debounce.md) |
| `use_throttled_value` | A signal that follows another at most once per period. | [Debounce and throttle](use_debounce.md) |
| `use_throttled_callback` | A callback that runs at once, then at most once per period. | [Debounce and throttle](use_debounce.md) |
| `use_hotkeys` | Runs a handler on a keyboard shortcut, from anywhere in the page. | [Hotkeys](use_hotkeys.md) |
| `use_media_query` | Whether a CSS media query matches, live. | [Media query](use_media_query.md) |
| `use_is_mobile` | Whether the viewport is narrower than 768px, live. | [Media query](use_media_query.md) |
| `use_theme` | The active theme, for values CSS cannot carry. | [Theming](theming.md) |
| `use_theme_set` | Reads and swaps the active theme set. | [Theme set](use_theme_set.md) |
| `use_localization` | The words libero's components say, in the active language. | [Localization](localization.md) |
| `use_localization_handle` | Switches the language at runtime. | [Localization](localization.md) |
| `use_formats` | The active date, time and number formats. | [Localization](localization.md) |
| `use_formats_handle` | Switches the formats at runtime. | [Localization](localization.md) |
| `use_stylesheet` | Registers a stylesheet of your own, above every libero layer. | [Stylesheet](use_stylesheet.md) |
| `use_accessibility` | The reader's motion, contrast and transparency settings; forces reduced motion. | [Accessibility settings](use_accessibility.md) |
| `use_scroll_area` | Scrolls a ScrollArea from code. | [ScrollArea](scroll_area.md) |
| `use_scroller` | Steps a Scroller from controls of your own. | [Scroller](scroller.md) |
| `use_form` | Controls a Form: validity, check, submit and reset. | [Form](form.md) |
| `use_form_context` | The handle of the Form it is called inside. | [Form](form.md) |
| `use_combobox` | Keeps a Combobox's open state in your scope. | [Combobox](combobox.md) |
| `use_modal` | Registers a modal and returns the handle that opens it. | [Modal](modal.md) |
| `use_modal_close` | Closes the modal it is rendered in. | [Modal](modal.md) |
| `use_drawer` | Registers a drawer and returns the handle that opens it. | [Drawer](drawer.md) |
| `use_popover` | Places a floating box next to an anchor. | [Popover](popover.md) |
| `use_menu` | Keeps a Menu's open state in your scope. | [Menu](menu.md) |
| `use_spotlight` | Registers a command palette and its hotkey. | [Spotlight](spotlight.md) |
| `use_lightbox` | Opens a picture viewer over the page. | [Lightbox](lightbox.md) |
| `use_floating_window` | Opens a movable, non-modal window. | [FloatingWindow](floating_window.md) |
| `use_notifications` | Shows notifications drawn as an Alert. | [Notifications](notifications.md) |
| `use_notifications_with` | Notifications of your own data type and template. | [Notifications](notifications.md) |
