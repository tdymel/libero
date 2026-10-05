use crate::Route;
use crate::components::{DocPage, DocSection};
use crate::nav::page_label;
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Table, Text, column};

#[derive(Clone, PartialEq)]
struct HookRow {
    hook: &'static str,
    purpose: &'static str,
    route: Route,
}

fn row(hook: &'static str, purpose: &'static str, route: Route) -> HookRow {
    HookRow {
        hook,
        purpose,
        route,
    }
}

/// Every public `use_*` of libero: the primitives first, then in the docs' group order.
fn hooks() -> Vec<HookRow> {
    vec![
        row(
            "use_id",
            "A unique id for the aria wiring between one instance's elements.",
            Route::UseIdPage {},
        ),
        row(
            "use_element",
            "A handle to one of your component's elements, to focus, scroll or measure it.",
            Route::UseElementPage {},
        ),
        row(
            "use_focus_return",
            "Puts focus back on the trigger when a panel closes.",
            Route::UseFocusReturnPage {},
        ),
        row(
            "use_drag",
            "Pointer capture and deltas for a drag.",
            Route::UseDragPage {},
        ),
        row(
            "use_sortable",
            "A list whose items reorder by dragging their handle.",
            Route::SortablePage {},
        ),
        row(
            "use_sortable_item",
            "One item of a use_sortable list: its handle and offset.",
            Route::SortablePage {},
        ),
        row(
            "use_intersection",
            "How much of an element is visible inside a root.",
            Route::UseIntersectionPage {},
        ),
        row(
            "use_in_viewport",
            "Whether an element is in the viewport.",
            Route::UseIntersectionPage {},
        ),
        row(
            "use_long_press",
            "Handlers that call back once a pointer stays down.",
            Route::UseLongPressPage {},
        ),
        row(
            "use_swipe",
            "Handlers that call back once a touch swipes far enough one way.",
            Route::UseSwipePage {},
        ),
        row(
            "use_edge_swipe",
            "Opens a drawer by a swipe in from the screen's start edge.",
            Route::UseSwipePage {},
        ),
        row(
            "use_timeout",
            "Runs a callback once, a while after you start it.",
            Route::UseTimersPage {},
        ),
        row(
            "use_interval",
            "Runs a callback repeatedly, with start, stop and toggle.",
            Route::UseTimersPage {},
        ),
        row(
            "use_debounced_value",
            "A signal that follows another once it stops changing.",
            Route::UseDebouncePage {},
        ),
        row(
            "use_debounced_callback",
            "A callback that runs after its last call, with the last argument.",
            Route::UseDebouncePage {},
        ),
        row(
            "use_throttled_value",
            "A signal that follows another at most once per period.",
            Route::UseDebouncePage {},
        ),
        row(
            "use_throttled_callback",
            "A callback that runs at once, then at most once per period.",
            Route::UseDebouncePage {},
        ),
        row(
            "use_history",
            "Undo and redo over snapshots of a value, rapid changes grouped.",
            Route::UseHistoryPage {},
        ),
        row(
            "use_hotkeys",
            "Runs a handler on a keyboard shortcut, from anywhere in the page.",
            Route::UseHotkeysPage {},
        ),
        row(
            "use_media_query",
            "Whether a CSS media query matches, live.",
            Route::UseMediaQueryPage {},
        ),
        row(
            "use_is_mobile",
            "Whether the viewport is narrower than 768px, live.",
            Route::UseMediaQueryPage {},
        ),
        row(
            "use_media",
            "Plays and reads an `<audio>` or `<video>` you render yourself.",
            Route::UseMediaPage {},
        ),
        row(
            "use_fullscreen",
            "Puts one element in fullscreen, drawn where the platform refuses it.",
            Route::UseFullscreenPage {},
        ),
        row(
            "use_back",
            "Runs a handler on Android's Back button instead of leaving the app.",
            Route::UseBackPage {},
        ),
        row(
            "use_geolocation",
            "The device's position, once or followed, and the location permission.",
            Route::UseGeolocationPage {},
        ),
        row(
            "use_tour",
            "A guided tour that spotlights one element per step.",
            Route::TourPage {},
        ),
        row(
            "use_local_storage",
            "A value kept under a key across reloads and app runs.",
            Route::UseLocalStoragePage {},
        ),
        row(
            "use_session_storage",
            "A value kept under a key for the tab or the window's run.",
            Route::UseLocalStoragePage {},
        ),
        row(
            "use_user_media",
            "The camera and microphone: a preview, a photo and a recording.",
            Route::UseUserMediaPage {},
        ),
        row(
            "use_user_media_devices",
            "The page's cameras and microphones, live.",
            Route::UseUserMediaPage {},
        ),
        row(
            "use_system_notification",
            "Notifications the operating system draws, and their permission.",
            Route::UseSystemNotificationPage {},
        ),
        row(
            "use_push_subscription",
            "A web push subscription for the app's server to push to.",
            Route::UseSystemNotificationPage {},
        ),
        row(
            "use_theme",
            "The active theme, for values CSS cannot carry.",
            Route::ThemingPage {},
        ),
        row(
            "use_theme_set",
            "Reads and swaps the active theme set.",
            Route::UseThemeSetPage {},
        ),
        row(
            "use_color_scheme",
            "Reads and sets the light or dark colour scheme.",
            Route::ThemeSwitcherPage {},
        ),
        row(
            "use_direction",
            "Reads and sets the text direction, left to right or right to left.",
            Route::DirectionTogglePage {},
        ),
        row(
            "use_localization",
            "The words libero's components say, in the active language.",
            Route::LocalizationPage {},
        ),
        row(
            "use_localization_handle",
            "Switches the language at runtime.",
            Route::LocalizationPage {},
        ),
        row(
            "use_formats",
            "The active date, time and number formats.",
            Route::LocalizationPage {},
        ),
        row(
            "use_formats_handle",
            "Switches the formats at runtime.",
            Route::LocalizationPage {},
        ),
        row(
            "use_stylesheet",
            "Registers a stylesheet of your own, above every libero layer.",
            Route::UseStylesheetPage {},
        ),
        row(
            "use_accessibility",
            "The reader's motion, contrast and transparency settings; forces reduced motion.",
            Route::UseAccessibilityPage {},
        ),
        row(
            "use_clipboard",
            "Copies text and tells whether the last copy landed.",
            Route::CopyPage {},
        ),
        row(
            "use_icon",
            "The glyph an IconProvider sets for a slot, else your default.",
            Route::IconProviderPage {},
        ),
        row(
            "use_scroll_area",
            "Scrolls a ScrollArea from code.",
            Route::ScrollAreaPage {},
        ),
        row(
            "use_scroller",
            "Steps a Scroller from controls of your own.",
            Route::ScrollerPage {},
        ),
        row(
            "use_form",
            "Controls a Form: validity, check, submit and reset.",
            Route::FormPage {},
        ),
        row(
            "use_form_context",
            "The handle of the Form it is called inside.",
            Route::FormPage {},
        ),
        row(
            "use_combobox",
            "Keeps a Combobox's open state in your scope.",
            Route::ComboboxPage {},
        ),
        row(
            "use_rich_text_editor",
            "Drives a RichTextEditor from toolbar controls of your own.",
            Route::RichTextEditorPage {},
        ),
        row(
            "use_modal",
            "Registers a modal and returns the handle that opens it.",
            Route::ModalPage {},
        ),
        row(
            "use_modal_close",
            "Closes the modal it is rendered in.",
            Route::ModalPage {},
        ),
        row(
            "use_drawer",
            "Registers a drawer and returns the handle that opens it.",
            Route::DrawerPage {},
        ),
        row(
            "use_popover",
            "Places a floating box next to an anchor.",
            Route::PopoverPage {},
        ),
        row(
            "use_menu",
            "Keeps a Menu's open state in your scope.",
            Route::MenuPage {},
        ),
        row(
            "use_spotlight",
            "Registers a command palette and its hotkey.",
            Route::SpotlightPage {},
        ),
        row(
            "use_lightbox",
            "Opens a picture viewer over the page.",
            Route::LightboxPage {},
        ),
        row(
            "use_floating_window",
            "Opens a movable, non-modal window.",
            Route::FloatingWindowPage {},
        ),
        row(
            "use_notifications",
            "Shows notifications drawn as an Alert.",
            Route::NotificationsPage {},
        ),
        row(
            "use_notifications_with",
            "Notifications of your own data type and template.",
            Route::NotificationsPage {},
        ),
    ]
}

#[component]
pub fn HooksPage() -> Element {
    rsx! {
        DocPage {
            title: "Hooks",
            markdown: "/md/hooks.md",
            lead: rsx! {
                Text {
                    "Libero's components are built from these hooks, and they are public for "
                    "yours. They are positional like every dioxus hook, so call them "
                    "unconditionally, in the same order every render."
                }
            },

            DocSection {
                title: "Every public hook",
                Text {
                    "The primitives and the theme-set and stylesheet hooks have a page of their "
                    "own; the unique ID, focus return and accessibility settings hooks have "
                    "theirs under Accessibility. The rest are shown on the component or guide "
                    "page they belong to."
                }
                Table {
                    caption: "Libero's public hooks",
                    data: hooks(),
                    columns: vec![
                        column("Hook")
                            .value(|row: &HookRow| row.hook)
                            .render(|row: &HookRow| rsx! {
                                Code { source: row.hook }
                            }),
                        column("What it is for").value(|row: &HookRow| row.purpose),
                        column("Page")
                            .value(|row: &HookRow| page_label(&row.route).unwrap_or_default())
                            .render(|row: &HookRow| rsx! {
                                Anchor { to: row.route.clone(),
                                    {page_label(&row.route).unwrap_or_default()}
                                }
                            }),
                    ],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, path::Path};

    /// Every `use_*` a `pub use` in libero's source names; `pub(crate) use` stays out.
    fn exported(dir: &Path, hooks: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                exported(&path, hooks);
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let source = fs::read_to_string(&path).unwrap();
            // Indented too: `rich_text` re-exports its hook from an inline `pub mod`.
            for (at, _) in source.match_indices("pub use ") {
                let line_start = source[..at].rfind('\n').map_or(0, |at| at + 1);
                if !source[line_start..at].trim().is_empty() {
                    continue;
                }
                let rest = &source[at..];
                let statement = &rest[..rest.find(';').unwrap()];
                let ident = |c: char| c.is_alphanumeric() || c == '_';
                for (start, _) in statement.match_indices("use_") {
                    if statement[..start].ends_with(ident) {
                        continue;
                    }
                    let word: String = statement[start..]
                        .chars()
                        .take_while(|c| ident(*c))
                        .collect();
                    // A `use_modal::{..}` path segment is a module, not the hook.
                    if !statement[start + word.len()..].starts_with("::") {
                        hooks.insert(word);
                    }
                }
            }
        }
    }

    #[test]
    fn every_public_hook_has_a_row() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../libero/src");
        let mut public = BTreeSet::new();
        exported(&root, &mut public);
        let listed: BTreeSet<String> = super::hooks()
            .iter()
            .map(|row| row.hook.to_string())
            .collect();
        assert_eq!(
            public, listed,
            "libero's pub use hooks against the Hooks page's rows"
        );
    }
}
