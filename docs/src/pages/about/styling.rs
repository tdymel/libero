use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, CodeBlock, DataList, DataListItem, Flex, States, Text},
    sx::{Sx, sx},
    theme::Size,
};

/// The chain the first demo builds, as the reader would type it. Kept beside
/// `card_sx`, which has to render the same thing.
fn card_sx_code(values: &DemoValues) -> Vec<String> {
    let color = values.str("background");
    let mut chain = vec![
        format!(".background({color:?})"),
        format!(".color({:?})", format!("{color}-contrast")),
        format!(".padding({:?})", values.str("padding")),
        format!(".border_radius({:?})", values.str("border_radius")),
    ];
    if values.str("hover") == "true" {
        chain.push(format!(
            ".hover(sx().background({:?}))",
            format!("{color}.8")
        ));
    }

    let chain = chain
        .iter()
        .map(|method| format!("\n        {method}"))
        .collect::<String>();
    vec![format!("sx: sx(){chain}")]
}

fn card_sx(values: &DemoValues) -> Sx {
    let color = values.str("background");
    sx().background(color.clone())
        .color(format!("{color}-contrast"))
        .padding(values.str("padding"))
        .border_radius(values.str("border_radius"))
        .apply_if((values.str("hover") == "true").then_some(()), |base, ()| {
            base.hover(sx().background(format!("{color}.8")))
        })
}

const STATES_SX: &str = r#"sx: sx()
        .padding("sm")
        .border_radius("xl")
        .background("grey.2")
        .when("active", sx().background("primary").color("primary-contrast"))
        .when("danger", sx().background("error").color("error-contrast"))"#;

fn states_sx() -> Sx {
    sx().padding("sm")
        .border_radius("xl")
        .background("grey.2")
        .when(
            "active",
            sx().background("primary").color("primary-contrast"),
        )
        .when("danger", sx().background("error").color("error-contrast"))
}

const THEME_VALUES: &str = r#"sx()
    .background("primary")        // shade 6, the base hex
    .border_color("primary.2")    // shades 1-9, generated from it
    .color("primary-contrast")    // black or white, whichever reads on it
    .padding("md")                // the theme's spacing scale
    .border_radius("lg")          // the theme's radius scale
    .width("240px")               // anything else is CSS text, untouched"#;

const SELECTORS: &str = r#"sx()
    .hover(sx().background("primary.7"))                  // == selector(":hover", ..)
    .selector("& svg", sx().width("18px"))                // a descendant
    .selector("&::before", sx().content("\"*\""))           // a pseudo-element
    .selector("&:not(:last-child)", sx().margin_bottom("sm"))
    .selector(".dark &", sx().background("grey.8"))       // this element, in a context
    .selector("&::before, &::after", sx().display("block"))"#;

const RESPONSIVE: &str = r#"sx()
    .flex_direction("column")
    .gap("sm")
    .breakpoint(Size::Sm, sx().flex_direction("row").gap("lg"))"#;

const RESPONSIVE_VALUE: &str = r#"sx().width(bp().sm("480px").lg("720px"))"#;

const LAYER_ORDER: &str = "@layer lsx-framework, lsx-user-static, lsx-user-custom;";

const STATIC_SX: &str = r#"static CARD_SX: StaticSx = StaticSx::new(|| {
    sx().padding("md")
        .border_radius("md")
        .background("grey.1")
        .when("selected", sx().background("primary.1"))
});

rsx! {
    Box {
        sx: &CARD_SX,
        states: States::new().active("selected"),
        "One class, built once"
    }
}"#;

const ESCAPE_HATCHES: &str = r#"Box {
    class: "prose",
    id: "intro",
    onclick: move |_| open.set(true),
    "Every component takes these"
}"#;

#[component]
pub fn StylingPage() -> Element {
    rsx! {
        DocPage {
            title: "Styling",
            lead: rsx! {
                Text {
                    "Every component takes the same four styling props: "
                    Code { source: "sx" }
                    " for CSS, "
                    Code { source: "states" }
                    " for your own variants, "
                    Code { source: "class" }
                    " for a stylesheet you already have, and everything a "
                    Code { source: "GlobalAttributes" }
                    " element accepts - "
                    Code { source: "id" }
                    ", "
                    Code { source: "onclick" }
                    ", the rest - straight through to the rendered tag."
                }
                Text {
                    Code { source: "sx()" }
                    " builds a list of declarations, not an inline style: identical "
                    "declarations hash to one "
                    Code { source: "lsx-*" }
                    " class shared by every element that asked for it, emitted once into a "
                    Code { source: "<style>" }
                    " tag. So a thousand rows styled alike cost one rule, and pseudo-classes "
                    "and media queries work - neither of which an inline style can express."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Box",
                    children_text: "Styled with sx",
                    controls: vec![
                        Control::color(
                            "background",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        )
                        .code(|_, values| card_sx_code(values)),
                        // The whole chain prints from the control above, so
                        // the rest add nothing of their own.
                        Control::slider("padding", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md")
                            .code(|_, _| vec![]),
                        Control::slider("border_radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md")
                            .code(|_, _| vec![]),
                        Control::switch("hover").code(|_, _| vec![]),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Box { sx: card_sx(&values), "Styled with sx" }
                    },
                }
            }

            DocSection {
                title: "Theme values",
                Text {
                    "A value is a plain string, and the ones the theme knows resolve against "
                    "it. A bare color name is shade 6; "
                    Code { source: "1" }
                    " through "
                    Code { source: "9" }
                    " are generated from that one hex value, and "
                    Code { source: "-contrast" }
                    " is whichever of black or white reads on it. A size word ("
                    Code { source: "xs" }
                    " to "
                    Code { source: "xxl" }
                    ") resolves through whichever scale the property belongs to - spacing for "
                    Code { source: "padding" }
                    "/"
                    Code { source: "margin" }
                    "/"
                    Code { source: "gap" }
                    ", radius for the "
                    Code { source: "border_radius" }
                    " family. Everything else is CSS text, untouched."
                }
                CodeBlock { source: THEME_VALUES, language: "rust" }
            }

            DocSection {
                title: "States",
                Text {
                    "Your own variants are not a second class per variant: fold every one "
                    "into the same "
                    Code { source: "sx" }
                    " with "
                    Code { source: "when" }
                    ", and switch between them with the "
                    Code { source: "states" }
                    " prop, which renders a "
                    Code { source: "data-state" }
                    " attribute. One shared class, per-instance variation - the same "
                    "mechanism every Libero component uses for its own size and variant "
                    "props. A condition can combine tokens with "
                    Code { source: "&&" }
                    " and "
                    Code { source: "||" }
                    "."
                }
                Demo {
                    component: "Box",
                    children_text: "Badge",
                    fixed: vec![STATES_SX.to_string()],
                    controls: vec![
                        Control::toggle("state", ["default", "active", "danger"]).code(
                            |_, values| match values.str("state").as_str() {
                                "default" => vec![],
                                state => {
                                    vec![format!("states: States::new().active({state:?})")]
                                }
                            },
                        ),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Box {
                            sx: states_sx(),
                            states: match values.str("state").as_str() {
                                "active" => States::new().active("active"),
                                "danger" => States::new().active("danger"),
                                _ => States::new(),
                            },
                            "Badge"
                        }
                    },
                }
            }

            DocSection {
                title: "Selectors",
                Text {
                    Code { source: "hover" }
                    ", "
                    Code { source: "focus" }
                    " and "
                    Code { source: "focus_visible" }
                    " nest an "
                    Code { source: "Sx" }
                    " under that pseudo-class. They are conveniences over "
                    Code { source: "selector" }
                    ", which takes any selector text and substitutes "
                    Code { source: "&" }
                    " with the class this "
                    Code { source: "Sx" }
                    " generates - so "
                    Code { source: "&" }
                    " can sit anywhere in the pattern, including after an ancestor. A "
                    "pattern without one is appended, making "
                    Code { source: "\":hover\"" }
                    " and "
                    Code { source: "\"&:hover\"" }
                    " the same thing, and a comma list expands to one rule per part."
                }
                CodeBlock { source: SELECTORS, language: "rust" }
            }

            DocSection {
                title: "Responsive",
                Text {
                    Code { source: "breakpoint" }
                    " nests a whole "
                    Code { source: "Sx" }
                    " under a "
                    Code { source: "min-width" }
                    " media query, so the unnested declarations are the small-screen ones "
                    "and each breakpoint overrides upwards."
                }
                CodeBlock { source: RESPONSIVE, language: "rust" }
                Text {
                    "When only one property changes, "
                    Code { source: "bp()" }
                    " puts the breakpoints inside the value instead of wrapping a block "
                    "around it:"
                }
                CodeBlock { source: RESPONSIVE_VALUE, language: "rust" }
                Text {
                    "It carries no base value, and a second call to the same property "
                    "replaces the first rather than adding to it - so a property that also "
                    "needs a value below the smallest breakpoint wants the nested form for "
                    "the base."
                }
                Text {
                    "The six breakpoints are fixed literals rather than theme values - a "
                    Code { source: "@media" }
                    " query cannot read a CSS custom property:"
                }
                Flex {
                    direction: "row",
                    wrap: "wrap",
                    gap: "sm",
                    for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl, Size::Xxl] {
                        CodeBlock {
                            key: "{size.as_str()}",
                            source: "{size.as_str()} {size.breakpoint_value()}",
                        }
                    }
                }
            }

            DocSection {
                title: "class and attributes",
                Text {
                    Code { source: "sx" }
                    " is additive, not exclusive: "
                    Code { source: "class" }
                    " puts your own class names on the same element for CSS you already "
                    "have, and any attribute or event a "
                    Code { source: "GlobalAttributes" }
                    " element accepts is forwarded to the rendered tag."
                }
                CodeBlock { source: ESCAPE_HATCHES, language: "rust" }
            }

            DocSection {
                title: "Cascade layers",
                Text {
                    "Nothing here is settled by specificity or source order. Libero emits "
                    "one "
                    Code { source: "@layer" }
                    " statement up front, and every rule it writes goes into one of those "
                    "layers - a later layer beats an earlier one however weak its selector "
                    "is, and specificity only decides ties "
                    Code { source: "within" }
                    " a layer."
                }
                CodeBlock { source: LAYER_ORDER, language: "css" }
                DataList {
                    orientation: "horizontal",
                    DataListItem {
                        label: rsx! {
                            Code { source: "lsx-framework" }
                        },
                        "Each component's own styling, and its focus ring."
                    }
                    DataListItem {
                        label: rsx! {
                            Code { source: "lsx-user-static" }
                        },
                        "Your sx prop. Beats the component's own styling, always."
                    }
                    DataListItem {
                        label: rsx! {
                            Code { source: "lsx-user-custom" }
                        },
                        "A stylesheet you registered yourself with use_stylesheet()."
                    }
                    DataListItem {
                        label: rsx! { "unlayered" },
                        "Your own CSS files, reached through the class prop. Unlayered CSS "
                        "outranks every layer, so these win outright."
                    }
                }
                Text {
                    "That last row is the one to remember: a plain stylesheet of your own is "
                    "not in the cascade layers at all, and CSS puts unlayered rules above "
                    "every layered one. So "
                    Code { source: "class" }
                    " is the heaviest hammer on the page - reach for "
                    Code { source: "sx" }
                    " first, and keep "
                    Code { source: "class" }
                    " for stylesheets you already own."
                }
                Text {
                    "It also means a utility framework like Tailwind composes with Libero "
                    "rather than fighting it: its utilities are unlayered, so a "
                    Code { source: "class: \"mt-4\"" }
                    " wins over both the component's own margin and anything an "
                    Code { source: "sx" }
                    " set. That works in theory and we do not test it - using Tailwind "
                    "alongside Libero is unsupported, not forbidden."
                }
            }

            DocSection {
                title: "Static sx",
                Text {
                    "An "
                    Code { source: "Sx" }
                    " written inline is rebuilt, re-hashed and (the first time that exact "
                    "content appears) rendered to CSS on every render. When the styling "
                    "does not depend on props, hoist it into a "
                    Code { source: "StaticSx" }
                    " and pass a reference: the builder runs once per process, the CSS text "
                    "is rendered once and cached by address, and the per-render check "
                    "becomes a pointer comparison instead of a content hash."
                }
                CodeBlock { source: STATIC_SX, language: "rust" }
                Text {
                    "Measured at roughly 210 ns per call site per render - too small to "
                    "matter in a page, worth having in a component that a hundred rows "
                    "mount. Every constant "
                    Code { source: "sx" }
                    " inside Libero itself is a "
                    Code { source: "StaticSx" }
                    " for that reason. If you are building components on top of Libero, "
                    "that is the pattern to copy: a static per component, with the parts "
                    "that genuinely vary carried by "
                    Code { source: "states" }
                    " rather than by a fresh "
                    Code { source: "Sx" }
                    " per render."
                }
            }
        }
    }
}
