use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Table, Text, column};

#[derive(Clone, PartialEq)]
struct HookRow {
    hook: &'static str,
    purpose: &'static str,
    route: Route,
    page: &'static str,
}

fn row(hook: &'static str, purpose: &'static str, route: Route, page: &'static str) -> HookRow {
    HookRow {
        hook,
        purpose,
        route,
        page,
    }
}

/// Every public `use_*` of libero: the primitives first, then by the page that owns them.
fn hooks() -> Vec<HookRow> {
    vec![
        row(
            "use_id",
            "A unique id for the aria wiring between one instance's elements.",
            Route::UseIdPage {},
            "use_id",
        ),
        row(
            "use_element",
            "A handle to one of your component's elements, to focus, scroll or measure it.",
            Route::UseElementPage {},
            "use_element",
        ),
        row(
            "use_focus_return",
            "Puts focus back on the trigger when a panel closes.",
            Route::UseFocusReturnPage {},
            "use_focus_return",
        ),
        row(
            "use_drag",
            "Pointer capture and deltas for a drag.",
            Route::UseDragPage {},
            "use_drag",
        ),
        row(
            "use_clipboard",
            "Copies text and reports whether the write worked.",
            Route::UseClipboardPage {},
            "use_clipboard",
        ),
        row(
            "use_modal",
            "Registers a modal and returns a handle that opens it.",
            Route::ModalPage {},
            "Modal",
        ),
        row(
            "use_modal_close",
            "Closes the modal it is called inside.",
            Route::ModalPage {},
            "Modal",
        ),
        row(
            "use_drawer",
            "Registers a drawer and returns a handle that opens it.",
            Route::DrawerPage {},
            "Drawer",
        ),
        row(
            "use_popover",
            "Places a floating box next to an anchor.",
            Route::PopoverPage {},
            "Popover",
        ),
        row(
            "use_menu",
            "Keeps a Menu's open state in your scope.",
            Route::MenuPage {},
            "Menu",
        ),
        row(
            "use_spotlight",
            "Registers a command palette and its hotkey.",
            Route::SpotlightPage {},
            "Spotlight",
        ),
        row(
            "use_lightbox",
            "Opens a picture viewer over the page.",
            Route::LightboxPage {},
            "Lightbox",
        ),
        row(
            "use_floating_window",
            "Opens a movable, resizable window.",
            Route::FloatingWindowPage {},
            "FloatingWindow",
        ),
        row(
            "use_notifications",
            "Shows and dismisses notifications.",
            Route::NotificationsPage {},
            "Notifications",
        ),
        row(
            "use_notifications_with",
            "Notifications drawn from your own data type.",
            Route::NotificationsPage {},
            "Notifications",
        ),
        row(
            "use_combobox",
            "Keeps a Combobox's open state in your scope.",
            Route::ComboboxPage {},
            "Combobox",
        ),
        row(
            "use_scroll_area",
            "Scrolls a ScrollArea from code.",
            Route::ScrollAreaPage {},
            "ScrollArea",
        ),
        row(
            "use_scroller",
            "Steps a Scroller from code.",
            Route::ScrollerPage {},
            "Scroller",
        ),
        row(
            "use_form",
            "A form's handle: values, validation and submit.",
            Route::FormPage {},
            "Form",
        ),
        row(
            "use_form_context",
            "The enclosing Form's handle.",
            Route::FormPage {},
            "Form",
        ),
        row(
            "use_theme",
            "The active theme.",
            Route::ThemingPage {},
            "Theming",
        ),
        row(
            "use_theme_set",
            "Switches between the themes of a set.",
            Route::ThemingPage {},
            "Theming",
        ),
        row(
            "use_color_scheme",
            "Reads and sets light or dark.",
            Route::ThemingPage {},
            "Theming",
        ),
        row(
            "use_localization",
            "The labels libero's components read, in the active language.",
            Route::LocalizationPage {},
            "Localization",
        ),
        row(
            "use_localization_handle",
            "Switches the locale at runtime.",
            Route::LocalizationPage {},
            "Localization",
        ),
        row(
            "use_formats",
            "The active date, time and number formats.",
            Route::LocalizationPage {},
            "Localization",
        ),
        row(
            "use_formats_handle",
            "Switches the formats at runtime.",
            Route::LocalizationPage {},
            "Localization",
        ),
        row(
            "use_stylesheet",
            "Registers a stylesheet of your own, above every libero layer.",
            Route::StylingPage {},
            "Styling",
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
                    "unconditionally, in the same order every render. The five primitives "
                    "have a page each. A hook that belongs to a component or a guide is "
                    "documented there."
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
                        column("Documented on")
                            .value(|row: &HookRow| row.page)
                            .render(|row: &HookRow| rsx! {
                                Anchor { to: row.route.clone(), "{row.page}" }
                            }),
                    ],
                }
            }
        }
    }
}
