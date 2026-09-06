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

// snippet: in Box { states: States::new().active("active"), .. }
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
    .margin_top("-sm")            // the same scale, negated
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

const MEDIA: &str = r#"sx()
    .transition("transform 200ms ease")
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))"#;

const MEDIA_NESTING: &str = r#"// Wrong - the transition still plays under reduced motion.
sx().when("open", sx().transition("transform 200ms ease"))
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))

// Right - both rules are 0-2-0, and the media one comes later.
sx().when(
    "open",
    sx().transition("transform 200ms ease")
        .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
)"#;

const CONTAINER: &str = r#"// The ancestor whose width the answer depends on:
sx().container("demo-card")

// A descendant, asking about it:
sx().width("100%")
    .container_query("demo-card", "(min-width: 640px)", sx().width("388px"))

// Or at a Size's breakpoint:
sx().container_breakpoint("demo-card", Size::Md, sx().width("388px"))"#;

const LAYER_ORDER: &str = "@layer lsx-base, lsx-framework, lsx-user-static, lsx-user-custom;";

/// The order an app declares for itself, when it wants its own layers ranked
/// against Libero's rather than below them.
const APP_LAYER_ORDER: &str =
    "@layer lsx-base, app-base, lsx-framework, lsx-user-static, lsx-user-custom, app-overrides;";

/// The order a Tailwind v4 app declares: its reset under Libero's components,
/// its utilities over them.
const TAILWIND_LAYER_ORDER: &str = "@layer theme, base, lsx-base, lsx-framework, lsx-user-static, lsx-user-custom, components, utilities;";

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

// snippet: let mut open = use_signal(|| false);
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
            markdown: "/md/styling.md",
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
                        Control::color("background")
                        // The chain derives `-contrast` and `.8` from the
                        // name, which a custom hex has neither of.
                        .without_custom()
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
                    " family. Prefix it with a "
                    Code { source: "-" }
                    " to read the same scale in the negative direction. Everything else is "
                    "CSS text, untouched."
                }
                CodeBlock { source: THEME_VALUES, language: "rust" }
                Text {
                    "A theme color given to "
                    Code { source: "background" }
                    " or "
                    Code { source: "background_color" }
                    " also tells the focus rings inside the element which color reads on "
                    "it. A "
                    Code { source: "var()" }
                    " or any other CSS text tells them nothing, so they keep the "
                    "surrounding one."
                }
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
                    "and each breakpoint overrides upwards. Several blocks apply in the order "
                    "they are written, so write them smallest first; a debug build warns when a "
                    "wider one comes before a narrower one."
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
                    "the base. The steps apply from the smallest up, in whatever order they "
                    "are written."
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
                title: "Media queries",
                Text {
                    Code { source: "media" }
                    " nests an "
                    Code { source: "Sx" }
                    " under any "
                    Code { source: "@media" }
                    " query, passed through verbatim. It is the general form "
                    Code { source: "breakpoint" }
                    " is the shorthand of, and the reason it exists is "
                    Code { source: "prefers-reduced-motion" }
                    ":"
                }
                CodeBlock { source: MEDIA, language: "rust" }
                Text {
                    "Nothing validates the query, so a typo silently matches nothing - the "
                    "same tradeoff "
                    Code { source: "when" }
                    " makes. Nested "
                    Code { source: "media" }
                    " modifiers fold into a single "
                    Code { source: "and" }
                    " query, exactly like nested "
                    Code { source: "breakpoint" }
                    "s."
                }
                Text {
                    "Nest "
                    Code { source: "media" }
                    " inside the condition, never the condition inside "
                    Code { source: "media" }
                    ". A media query adds no specificity: the rule inside it carries the "
                    "same selector as the one outside. A condition appends "
                    Code { source: "[data-state~=\"open\"]" }
                    ", so it is one class and one attribute (0-2-0) against a bare class "
                    "(0-1-0), and the flat form loses no matter where it sits in the file:"
                }
                CodeBlock { source: MEDIA_NESTING, language: "rust" }
                Text {
                    "The same applies to any modifier that changes the selector - "
                    Code { source: "hover" }
                    ", "
                    Code { source: "selector" }
                    " and "
                    Code { source: "when" }
                    " all do. "
                    Code { source: "breakpoint" }
                    " and "
                    Code { source: "container_query" }
                    " do not, so those compose in either order."
                }
            }

            DocSection {
                title: "Container queries",
                Text {
                    Code { source: "breakpoint" }
                    " asks about the viewport, but "
                    "\"does this component have room\" is almost always a question about "
                    "the box it sits in. The two disagree whenever a component lives in a "
                    "column narrower than the window - a sidebar, a grid cell, a card."
                }
                Text {
                    Code { source: "container" }
                    " marks an ancestor as a query container and "
                    Code { source: "container_query" }
                    " asks about it by name. The name is required: an anonymous container "
                    "binds the query to the nearest ancestor container, which picks the "
                    "wrong one as soon as containers nest."
                }
                CodeBlock { source: CONTAINER, language: "rust" }
                Text {
                    Code { source: "container" }
                    " emits "
                    Code { source: "container-type: inline-size" }
                    ", which is "
                    Code { source: "contain: layout style inline-size" }
                    ". The element stops being sized by its own contents in the inline "
                    "axis, becomes a stacking context, and becomes the containing block "
                    "for absolutely positioned descendants - so never mark a shrink-to-fit "
                    "box as a container. Like "
                    Code { source: "@media" }
                    ", the condition cannot read a CSS custom property."
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
                    "Nothing that styles an element is settled by specificity or source "
                    "order. Libero emits one "
                    Code { source: "@layer" }
                    " statement up front, and every rule it writes that styles an element "
                    "goes into one of those layers - a later layer beats an earlier one "
                    "however weak its selector is, and specificity only decides ties "
                    Code { source: "within" }
                    " a layer. The theme's custom properties on "
                    Code { source: ":root" }
                    " and its "
                    Code { source: "@keyframes" }
                    " stay outside it: neither is a cascaded rule, so a layer would only "
                    "make them harder to override."
                }
                CodeBlock { source: LAYER_ORDER, language: "css" }
                DataList {
                    orientation: "horizontal",
                    DataListItem {
                        label: rsx! {
                            Code { source: "lsx-base" }
                        },
                        "The theme's reset and its body rules. First, so everything "
                        "else outranks them."
                    }
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
                    "Layers rank by first mention, and Libero declares its own when "
                    Code { source: "LiberoProvider" }
                    " mounts - after anything already in "
                    Code { source: "<head>" }
                    ". So a stylesheet of yours that declares layers has them ranked "
                    "below every lsx layer, and a layered "
                    Code { source: "body {{ margin: 2rem }}" }
                    " of yours still loses to "
                    Code { source: "lsx-base" }
                    ". Leave the rule unlayered and it wins - that is what moving the "
                    "reset into a layer bought you - or declare the whole order yourself, "
                    "first in a stylesheet the browser sees before Libero's, and put your own layers "
                    "wherever you want them. Both measured in Chromium."
                }
                CodeBlock { source: APP_LAYER_ORDER, language: "css" }
                Text {
                    "A utility framework like Tailwind composes with Libero once you declare "
                    "the order. Tailwind v3's utilities are unlayered, so a "
                    Code { source: "class: \"mt-4\"" }
                    " outranks every Libero layer. Tailwind v4 layers everything - its reset "
                    "in "
                    Code { source: "base" }
                    ", its utilities in "
                    Code { source: "utilities" }
                    " - and left alone those rank below Libero's: the reset leaves Libero's "
                    "components alone, but a utility class on one loses. Put this first in "
                    "your CSS, before "
                    Code { source: "@import \"tailwindcss\"" }
                    ", so the reset stays under Libero's components and the utilities win "
                    "over them. Measured in Chromium: every Button kept its fill and "
                    Code { source: "bg-red-500" }
                    " won on all of them. Using Tailwind alongside Libero is unsupported, "
                    "not forbidden."
                }
                CodeBlock { source: TAILWIND_LAYER_ORDER, language: "css" }
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
