use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Text};

#[component]
pub fn PhilosophyPage() -> Element {
    rsx! {
        DocPage {
            title: "Philosophy",
            markdown: "/md/philosophy.md",
            lead: rsx! {
                Text {
                    "Libero is a component library for Dioxus. Four principles decide what goes "
                    "into it and how it is shaped, listed here in order of priority. Each section "
                    "says what the principle means in libero and where you can see it in the code."
                }
            },

            DocSection {
                title: "Developer experience first",
                Text {
                    "AI writes a lot of code now, and that makes the API matter more, not less. "
                    "Someone still reads the result, reviews it and changes it next month. A small "
                    "API that is hard to misuse is easy for a person to review and easy for a model "
                    "to get right the first time."
                }
                Text {
                    "So libero leans on the type system instead of conventions. "
                    Code { source: "Tabs" }
                    " over an enum gets its tab strip from "
                    Code { source: "#[derive(Options)]" }
                    ", and its "
                    Code { source: "panel" }
                    " is a "
                    Code { source: "match" }
                    ", so the compiler rejects a tab without a panel. A "
                    Code { source: "Table" }
                    " column is "
                    Code { source: "column(\"Age\").value(|p| p.age)" }
                    ", and the cell's type alone makes it sort numerically and align right. Props "
                    "are typed as well, but the common ones also take a string: "
                    Code { source: "size: \"sm\"" }
                    " and "
                    Code { source: "size: Size::Sm" }
                    " are the same prop."
                }
                Text {
                    "Composition works the same way. Every field has the same five slots: label, "
                    "description, control, helper text and validation message. A "
                    Code { source: "Fieldset" }
                    " groups fields into one value, a "
                    Code { source: "Form" }
                    " holds the whole value, and typed paths from "
                    Code { source: "#[derive(Fields)]" }
                    " tie each layer to your own structs. Styling is one builder, "
                    Code { source: "sx()" }
                    ", on every component. Setup is one "
                    Code { source: "LiberoProvider" }
                    " at the root."
                }
                Text {
                    "Every docs page also has a plain markdown copy with complete examples, the "
                    "\"View as markdown\" link at the top. Those examples are compiled by the test "
                    "suite, so the code you or your assistant copies out of them builds."
                }
            }

            DocSection {
                title: "Accessibility second",
                Text {
                    "Accessibility is not only for screen reader users. Keyboard support helps "
                    "anyone who would rather not reach for the mouse. A state that does not rely "
                    "on colour still reads on a dim laptop screen in the sun. Motion that stops on "
                    "request helps anyone it makes dizzy."
                }
                Text {
                    "Where WAI-ARIA, the APG patterns or WCAG have an opinion, libero follows it, "
                    "and a deviation is what needs a reason. Menus, trees, tabs and listboxes take "
                    "the keyboard model the APG describes for them, and each component page lists "
                    "its keys. WCAG 2.2 AA is the target: 4.5:1 contrast for text and 3:1 for "
                    "borders and icons are the numbers we measure against."
                }
                Text {
                    "A pressed, selected or current control never differs by colour alone. It "
                    "carries a short 2px line in its own text colour. Under "
                    Code { source: "prefers-reduced-motion" }
                    " a "
                    Code { source: "Carousel" }
                    " opens paused and an "
                    Code { source: "Indicator" }
                    " stops its ping. A component that needs an accessible name and has none warns "
                    "you in a debug build. Every string a component says on its own, like \"Go to "
                    "page 3\" on a "
                    Code { source: "Pagination" }
                    " button, comes from a "
                    Code { source: "Localization" }
                    ", with English and German built in."
                }
                Text {
                    "Forced colours (Windows High Contrast) are supported only in part. The "
                    Anchor { to: crate::Route::AccessibilityPage {}, "Accessibility" }
                    " page says what holds and what does not."
                }
            }

            DocSection {
                title: "Batteries included",
                Text {
                    "Building an app should not start with a hunt for a date picker. Libero has "
                    "more than 100 components: layout, typography, navigation, feedback, data "
                    "display, and a form stack from "
                    Code { source: "TextField" }
                    " to "
                    Code { source: "PhoneField" }
                    ", "
                    Code { source: "DatePicker" }
                    ", "
                    Code { source: "ColorPicker" }
                    " and "
                    Code { source: "FileField" }
                    ", with validation and a focused error summary in "
                    Code { source: "Form" }
                    "."
                }
                Text {
                    "The overlays are there too. A modal and a drawer open from a hook ("
                    Code { source: "use_modal" }
                    ", "
                    Code { source: "use_drawer" }
                    ") and can hand a result back. "
                    Code { source: "Popover" }
                    ", "
                    Code { source: "Tooltip" }
                    ", "
                    Code { source: "HoverCard" }
                    ", "
                    Code { source: "Menu" }
                    " and a command palette cover the rest, and "
                    Code { source: "use_notifications()" }
                    " shows a message from anywhere. Around them sit the hooks the components are "
                    "built from ("
                    Code { source: "use_drag" }
                    ", "
                    Code { source: "use_clipboard" }
                    "), "
                    Code { source: "Virtualize" }
                    " for long lists, and "
                    Code { source: "CodeBlock" }
                    " with 30 grammars that you pay for only when you turn them on."
                }
                Text {
                    "The same components run on the web and natively through Blitz. Anything that "
                    "reaches the machine goes through one trait in "
                    Code { source: "libero::platform" }
                    ", and a capability the renderer lacks is "
                    Code { source: "None" }
                    ": absent, not broken. You branch once, and the rest of your code stays the same."
                }
            }

            DocSection {
                title: "Simple yet modern",
                Text {
                    "The default theme aims to look clean and stay out of the way, so it fits most "
                    "apps without a fight. When it does not, theming is plain Rust. A theme is one "
                    "struct of colours, scales and per-component defaults, and you change what you "
                    "need with struct update syntax over "
                    Code { source: "Theme::DEFAULT" }
                    "."
                }
                Text {
                    "The provider emits the theme once as CSS custom properties, light and dark "
                    "together. Switching the scheme is one attribute on the root, correct on first "
                    "paint, and "
                    Code { source: "ColorSchemeButton" }
                    " does it for you. Styling uses current CSS without a CSS framework: container "
                    "queries in "
                    Code { source: "sx" }
                    ", and cascade layers, so your own unlayered CSS beats every libero rule."
                }
            }
        }
    }
}
