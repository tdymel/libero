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
                    "unconditionally, in the same order every render. The primitives and the "
                    "theme-set and stylesheet hooks have a page of their own. The rest are "
                    "shown on the component or guide page they belong to."
                }
            },

            DocSection {
                title: "Every public hook",
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
