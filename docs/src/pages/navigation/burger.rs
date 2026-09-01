use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Burger, Code, CodeBlock, Flex, Input, Paper, Text};
use libero::sx::sx;

/// The panel the preview's `aria-controls` points at. A `Burger` whose
/// `opened` is `Some` and whose `aria-controls` names nothing is exactly what
/// the component warns about, so the demo has to own a real one - and
/// `opened` is a relation to a neighbour, which the preview needs anyway.
const PANEL_ID: &str = "burger-demo-panel";

// Rendered on both states, hidden rather than absent: `aria-controls`
// pointing at an element that is not in the document is a dangling
// reference, and a disclosure's panel is the thing that exists and is
// hidden.
const PANEL: &str = r#"Paper {
    id: "burger-demo-panel",
    sx: sx().display(if opened() { "block" } else { "none" }),
    "Navigation"
}"#;

const SIGNAL: &str = r#"let mut opened = use_signal(|| false);

rsx! {
    Burger {
        opened: opened(),
        "aria-controls": "site-nav",
        onclick: move |_| opened.toggle(),
        sx: sx().breakpoint(Size::Sm, sx().display("none")),
    }
    Sidebar { id: "site-nav", side: "left", role: "navigation",
        // ..
    }
}"#;

const MODAL: &str = r#"// No `opened`: nothing is expanded, so nothing announces a state.
Burger { onclick: move |_| modal.open() }"#;

const LOCALE: &str = r#"Theme {
    burger: BurgerDefaults {
        labels: BurgerLabels { open: "Menü öffnen", close: "Menü schließen" },
        ..Theme::DEFAULT.burger
    },
    ..Theme::DEFAULT
}

// Or, when the locale is only known at runtime:
Burger {
    opened: opened(),
    label: move |opened| t(if opened { "nav.close" } else { "nav.open" }),
}"#;

/// The preview is a pair - the button and the thing it expands - so the block
/// has to be that pair too, signal and all.
fn wrap(_values: &DemoValues, source: &str) -> String {
    let mut code = String::from("let mut opened = use_signal(|| false);\n\nrsx! {\n");
    code.push_str(&indent(source));
    code.push_str(&indent(PANEL));
    code.push('}');
    code
}

#[component]
pub fn BurgerPage() -> Element {
    rsx! {
        DocPage {
            title: "Burger",
            source: "libero/src/components/navigation/burger.rs",
            markdown: "/md/burger.md",
            properties: vec![props("Burger", vec![
                prop("opened", "bool")
                    .doc("`true` draws the X, and either value emits `aria-expanded`, which makes the button a disclosure. Omit it entirely for a burger that opens something that is not one - a modal."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("`Burger` never owns the open state; the panel does, and the caller already holds that signal to drive the panel itself."),
                prop("label", "Callback<bool, String>")
                    .doc("Replaces the theme's two labels, keyed by `opened`. It runs during render, so it can read a live locale."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("The glyph's width and height. The bars are a twelfth of it thick, and the button around it is one `spacing.xs` larger."),
                prop("color", "ThemeAwareValue")
                    .default("currentColor")
                    .doc("The bars. Unset they inherit, so styling the button's `color` reaches them."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("The button's corners, not the bars'."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Passed through to the underlying button."),
            ])],
            lead: rsx! {
                Text {
                    "Three bars that morph into an X. It renders an "
                    Code { source: "ActionIcon" }
                    " - a real "
                    Code { source: "<button type=\"button\">" }
                    " with the ripple and the disabled handling already right - and adds the "
                    "glyph plus the three ARIA facts a burger is usually missing: "
                    Code { source: "aria-expanded" }
                    ", an accessible name that changes with the state, and the "
                    Code { source: "aria-controls" }
                    " you spread to name the panel. The morph animates "
                    Code { source: "background-color" }
                    " and "
                    Code { source: "transform" }
                    " only, and is dropped entirely under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " - the two states differ in shape, not just in position, so snapping "
                    "between them stays legible."
                }
            },
            Demo {
                component: "Burger",
                children_text: "",
                // The attribute is a `GlobalAttributes` pass-through, not a
                // prop, and it is the half of the contract no control can
                // show - so it is printed on every state.
                fixed: vec![format!("{:?}: {PANEL_ID:?}", "aria-controls")],
                controls: vec![
                    // `Some(false)` is still a disclosure
                    // (`aria-expanded="false"`); unset is not.
                    Control::switch("opened").code(|_, values| match values.str("opened").as_str() {
                        "true" => vec![
                            "opened: opened()".to_string(),
                            "onclick: move |_| opened.toggle()".to_string(),
                        ],
                        // Off is unset, not `false`: no `opened` means no
                        // `aria-expanded` at all, which is what a burger that
                        // opens a modal wants.
                        _ => vec!["onclick: move |_| opened.toggle()".to_string()],
                    }),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::color(
                        "color",
                        [UNSET, "primary", "secondary", "success", "error", "warning"],
                    ),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::switch("disabled"),
                ],
                wrap: Wrap(wrap),
                render: move |values: DemoValues| {
                    let opened = values.str("opened") == "true";

                    rsx! {
                        Flex {
                            direction: "row",
                            align: "center",
                            gap: "md",
                            Burger {
                                opened: opened.then_some(true),
                                "aria-controls": PANEL_ID,
                                onclick: move |_| {},
                                size: values.str("size"),
                                color: match values.str("color").as_str() {
                                    UNSET => Input::None,
                                    color => Input::from(color.to_string()),
                                },
                                radius: values.str("radius"),
                                disabled: values.str("disabled") == "true",
                            }
                            Paper {
                                id: PANEL_ID,
                                sx: sx()
                                    .padding("sm")
                                    .display(if opened { "block" } else { "none" }),
                                "Navigation"
                            }
                        }
                    }
                },
            }
            DocSection {
                title: "Driving it from a signal",
                Text {
                    "The panel owns the state. "
                    Code { source: "Burger" }
                    " is told what it is, and tells the accessibility tree - which is why "
                    Code { source: "aria-controls" }
                    " has to name a real element: without it the state is announced but the "
                    "thing in that state is not."
                }
                CodeBlock { source: SIGNAL, language: "rust" }
            }
            DocSection {
                title: "Opening something that is not a disclosure",
                Text {
                    "Leave "
                    Code { source: "opened" }
                    " unset and no "
                    Code { source: "aria-expanded" }
                    " is emitted at all. A modal is not expanded by its trigger - it replaces "
                    "the page - so claiming otherwise is worse than saying nothing. The glyph "
                    "then stays three bars, which is honest: nothing has opened in place."
                }
                CodeBlock { source: MODAL, language: "rust" }
            }
            DocSection {
                title: "The two words it says",
                Text {
                    "The accessible name is the theme's, not the caller's, so every burger in "
                    "a project announces itself the same way. "
                    Code { source: "BurgerLabels" }
                    " holds them apart from the geometry, so a translation replaces two "
                    "strings without restating a pixel scale."
                }
                CodeBlock { source: LOCALE, language: "rust" }
            }
        }
    }
}
