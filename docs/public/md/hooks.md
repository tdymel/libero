# Hooks

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: Every public libero hook in one table, with what it is for and a link to its page.

Libero's components are built from these hooks, and they are public for yours.
They are positional like every dioxus hook, so call them unconditionally, in the
same order every render. Each has a page with a small demo. A hook that belongs
to a component links to that component's page for the rest.

## Every public hook

| Hook | What it is for |
|---|---|
| [`use_id`](use_id.md) | A unique id for the aria wiring between one instance's elements. |
| [`use_element`](use_element.md) | A handle to one of your component's elements, to focus, scroll or measure it. |
| [`use_focus_return`](use_focus_return.md) | Puts focus back on the trigger when a panel closes. |
| [`use_drag`](use_drag.md) | Pointer capture and deltas for a drag. |
| [`use_clipboard`](use_clipboard.md) | Copies text and reports whether the write worked. |
| [`use_theme`](use_theme.md) | The active theme, for values CSS cannot carry. |
| [`use_theme_set`](use_theme_set.md) | Reads and swaps the active theme set. |
| [`use_color_scheme`](use_color_scheme.md) | Reads and sets light or dark. |
| [`use_localization`](use_localization.md) | The words libero's components say, in the active language. |
| [`use_localization_handle`](use_localization_handle.md) | Switches the language at runtime. |
| [`use_formats`](use_formats.md) | The active date, time and number formats. |
| [`use_formats_handle`](use_formats_handle.md) | Switches the formats at runtime. |
| [`use_stylesheet`](use_stylesheet.md) | Registers a stylesheet of your own, above every libero layer. |
| [`use_scroll_area`](use_scroll_area.md) | Scrolls a ScrollArea from code. |
| [`use_scroller`](use_scroller.md) | Steps a Scroller from controls of your own. |
| [`use_form`](use_form.md) | Controls a Form: validity, check, submit and reset. |
| [`use_form_context`](use_form_context.md) | The handle of the Form it is called inside. |
| [`use_combobox`](use_combobox.md) | Keeps a Combobox's open state in your scope. |
| [`use_modal`](use_modal.md) | Registers a modal and returns the handle that opens it. |
| [`use_modal_close`](use_modal_close.md) | Closes the modal it is rendered in. |
| [`use_drawer`](use_drawer.md) | Registers a drawer and returns the handle that opens it. |
| [`use_popover`](use_popover.md) | Places a floating box next to an anchor. |
| [`use_menu`](use_menu.md) | Keeps a Menu's open state in your scope. |
| [`use_spotlight`](use_spotlight.md) | Registers a command palette and its hotkey. |
| [`use_lightbox`](use_lightbox.md) | Opens a picture viewer over the page. |
| [`use_floating_window`](use_floating_window.md) | Opens a movable, non-modal window. |
| [`use_notifications`](use_notifications.md) | Shows notifications drawn as an Alert. |
| [`use_notifications_with`](use_notifications_with.md) | Notifications of your own data type and template. |
