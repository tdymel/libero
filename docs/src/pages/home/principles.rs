use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Flex, Kbd, Options, Paper, Select, Text, Title},
    sx::sx,
    theme::Size,
};

use crate::{Route, nav};

const TYPED_SELECT: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Roast {
    Light,
    Medium,
    Dark,
}

let mut roast = use_signal(|| Some(Roast::Medium));

Select {
    label: "Roast",
    value: roast(),
    onchange: move |next| roast.set(next),
}"#;

const SX_BUTTON: &str = r#"Button {
    sx: sx()
        .padding_inline("xl")
        .hover(sx().background("primary.8")),
    "Save"
}"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Roast {
    Light,
    Medium,
    Dark,
}

/// The heavy components, by name, each linking to its page.
const HEAVY: [(&str, Route); 7] = [
    ("Combobox", Route::ComboboxPage {}),
    ("DatePicker", Route::DatePickerPage {}),
    ("Form", Route::FormPage {}),
    ("Spotlight", Route::SpotlightPage {}),
    ("Notifications", Route::NotificationsPage {}),
    ("CodeBlock", Route::CodeBlockPage {}),
    ("QrCode", Route::QrCodePage {}),
];

#[component]
fn Principle(title: &'static str, children: Element) -> Element {
    rsx! {
        Paper {
            sx: sx()
                .padding("lg")
                .flex("1 1 100%")
                .min_width("0")
                .breakpoint(Size::Md, sx().flex("1 1 calc(50% - 12px)")),
            Flex { direction: "column", gap: "md",
                Title { size: "md", component: "h3", "{title}" }
                {children}
            }
        }
    }
}

/// The four principles from the Philosophy page, each with something to see.
#[component]
pub fn Principles() -> Element {
    let mut roast = use_signal(|| Some(Roast::Medium));
    let (components, hooks) = nav::page_counts();

    rsx! {
        section { "aria-labelledby": "principles-title",
            Flex { direction: "column", gap: "lg",
                Title { size: "xl", component: "h2", id: "principles-title", "What we care about" }
                Text {
                    "Four principles decide what goes into libero, in this order. The "
                    Anchor { to: Route::PhilosophyPage {}, "Philosophy" }
                    " page explains each one."
                }
                Flex { direction: "row", gap: "lg", wrap: "wrap", align: "stretch",
                    Principle { title: "Developer experience first",
                        Text {
                            "Props are typed, so the compiler catches what a convention would "
                            "leave to code review. A "
                            Code { source: "Select" }
                            " over an enum lists its variants on its own:"
                        }
                        CodeBlock { source: TYPED_SELECT, language: "rust" }
                        Select {
                            label: "Roast",
                            value: roast(),
                            onchange: move |next| roast.set(next),
                        }
                        Text {
                            "Every docs page also has a markdown copy whose examples the test "
                            "suite compiles, so code you or your AI assistant copies out of it "
                            "builds."
                        }
                    }
                    Principle { title: "Accessibility second",
                        Text {
                            "Press "
                            Kbd { "Tab" }
                            ". The first stop is a skip link past the header, and a focus ring "
                            "follows you from there. "
                            Kbd { "Ctrl K" }
                            " opens the search from anywhere."
                        }
                        Text {
                            "Menus, tabs, trees and listboxes take the keyboard model the ARIA "
                            "Authoring Practices describe for them, and each component page lists "
                            "its keys. WCAG 2.2 AA is the target, and a selected control never "
                            "differs by colour alone."
                        }
                        Anchor { to: Route::AccessibilityPage {}, "What holds, and what does not" }
                    }
                    Principle { title: "Batteries included",
                        Text {
                            if hooks > 0 {
                                "{components} components and {hooks} hooks, each with its own page."
                            } else {
                                "{components} components, each with its own page."
                            }
                            " Among them:"
                        }
                        Flex { direction: "row", gap: "sm", wrap: "wrap",
                            for (name, route) in HEAVY {
                                Anchor { key: "{name}", to: route, Code { source: name } }
                            }
                        }
                        Text {
                            "CodeBlock highlights 30 languages, and you compile only the grammars "
                            "you turn on."
                        }
                    }
                    Principle { title: "Simple yet modern",
                        Text {
                            "The default theme stays out of the way. When you want more, "
                            Code { source: "sx()" }
                            " styles any component with a typed builder over current CSS, with no "
                            "CSS framework underneath."
                        }
                        CodeBlock { source: SX_BUTTON, language: "rust" }
                        Text {
                            "Your own unlayered CSS wins over every libero rule, because the "
                            "library's styles sit in cascade layers."
                        }
                    }
                }
            }
        }
    }
}
