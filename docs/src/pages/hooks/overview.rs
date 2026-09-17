use crate::Route;
use crate::components::{DocPage, DocSection};
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
            "use_clipboard",
            "Copies text and reports whether the write worked.",
            Route::UseClipboardPage {},
        ),
        row(
            "use_theme",
            "The active theme, for values CSS cannot carry.",
            Route::UseThemePage {},
        ),
        row(
            "use_theme_set",
            "Reads and swaps the active theme set.",
            Route::UseThemeSetPage {},
        ),
        row(
            "use_color_scheme",
            "Reads and sets light or dark.",
            Route::UseColorSchemePage {},
        ),
        row(
            "use_localization",
            "The words libero's components say, in the active language.",
            Route::UseLocalizationPage {},
        ),
        row(
            "use_localization_handle",
            "Switches the language at runtime.",
            Route::UseLocalizationHandlePage {},
        ),
        row(
            "use_formats",
            "The active date, time and number formats.",
            Route::UseFormatsPage {},
        ),
        row(
            "use_formats_handle",
            "Switches the formats at runtime.",
            Route::UseFormatsHandlePage {},
        ),
        row(
            "use_stylesheet",
            "Registers a stylesheet of your own, above every libero layer.",
            Route::UseStylesheetPage {},
        ),
        row(
            "use_scroll_area",
            "Scrolls a ScrollArea from code.",
            Route::UseScrollAreaPage {},
        ),
        row(
            "use_scroller",
            "Steps a Scroller from controls of your own.",
            Route::UseScrollerPage {},
        ),
        row(
            "use_form",
            "Controls a Form: validity, check, submit and reset.",
            Route::UseFormPage {},
        ),
        row(
            "use_form_context",
            "The handle of the Form it is called inside.",
            Route::UseFormContextPage {},
        ),
        row(
            "use_combobox",
            "Keeps a Combobox's open state in your scope.",
            Route::UseComboboxPage {},
        ),
        row(
            "use_modal",
            "Registers a modal and returns the handle that opens it.",
            Route::UseModalPage {},
        ),
        row(
            "use_modal_close",
            "Closes the modal it is rendered in.",
            Route::UseModalClosePage {},
        ),
        row(
            "use_drawer",
            "Registers a drawer and returns the handle that opens it.",
            Route::UseDrawerPage {},
        ),
        row(
            "use_popover",
            "Places a floating box next to an anchor.",
            Route::UsePopoverPage {},
        ),
        row(
            "use_menu",
            "Keeps a Menu's open state in your scope.",
            Route::UseMenuPage {},
        ),
        row(
            "use_spotlight",
            "Registers a command palette and its hotkey.",
            Route::UseSpotlightPage {},
        ),
        row(
            "use_lightbox",
            "Opens a picture viewer over the page.",
            Route::UseLightboxPage {},
        ),
        row(
            "use_floating_window",
            "Opens a movable, non-modal window.",
            Route::UseFloatingWindowPage {},
        ),
        row(
            "use_notifications",
            "Shows notifications drawn as an Alert.",
            Route::UseNotificationsPage {},
        ),
        row(
            "use_notifications_with",
            "Notifications of your own data type and template.",
            Route::UseNotificationsWithPage {},
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
                    "unconditionally, in the same order every render. Each has a page with a "
                    "small demo. A hook that belongs to a component links to that component's "
                    "page for the rest."
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
                                Anchor { to: row.route.clone(),
                                    Code { source: row.hook }
                                }
                            }),
                        column("What it is for").value(|row: &HookRow| row.purpose),
                    ],
                }
            }
        }
    }
}
