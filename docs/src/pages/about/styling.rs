use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, CodeBlock, States, Table, Text, column},
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
        .background("muted.2")
        .when("active", sx().background("primary").color("primary-contrast"))
        .when("danger", sx().background("error").color("error-contrast"))"#;

fn states_sx() -> Sx {
    sx().padding("sm")
        .border_radius("xl")
        .background("muted.2")
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
    .selector(".dark &", sx().background("muted.8"))       // this element, in a context
    .selector("&::before, &::after", sx().display("block"))"#;

const RESPONSIVE: &str = r#"sx()
    .flex_direction("column")
    .gap("sm")
    .breakpoint(Size::Sm, sx().flex_direction("row").gap("lg"))"#;

const RESPONSIVE_VALUE: &str = r#"sx().width(bp().sm("480px").lg("720px"))"#;

const MEDIA: &str = r#"sx()
    .transition("transform 200ms ease")
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))"#;

const MEDIA_NESTING: &str = r#"// Wrong: the transition still plays under reduced motion.
sx().when("open", sx().transition("transform 200ms ease"))
    .media("(prefers-reduced-motion: reduce)", sx().transition("none"))

// Right: both rules are 0-2-0, and the media one comes later.
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
        .background("muted.1")
        .when("selected", sx().background("primary.1"))
});

rsx! {
    Box {
        sx: &CARD_SX,
        states: States::new().active("selected"),
        "One class, built once"
    }
}"#;

const PARTS: &str = r#"static QUIET: StaticParts<AlertPart> =
    StaticParts::new(|| Parts::new().part(AlertPart::Message, sx().color("gray.7")));

rsx! {
    Alert {
        title: "Saved",
        parts: Parts::new()
            .part(AlertPart::Title, sx().font_weight("700"))
            .part(AlertPart::Close, sx().color("error.6")),
        "Your changes are live."
    }
    Alert { title: "Synced", parts: &QUIET, "Nothing to do." }
}"#;

// snippet: let mut open = use_signal(|| false);
const ESCAPE_HATCHES: &str = r#"Box {
    class: "prose",
    id: "intro",
    onclick: move |_| open.set(true),
    "Every component takes these"
}"#;

/// The styling props every component takes.
const PROPS: [(&str, &str); 4] = [
    ("sx", "CSS, compiled to one shared class."),
    ("states", "Your own variants, matched by when."),
    ("class", "A stylesheet you already have."),
    (
        "id, onclick, ...",
        "Any global attribute or event, passed to the rendered tag.",
    ),
];

/// A value, and what the theme resolves it to.
const VALUES: [(&str, &str); 6] = [
    ("\"primary\"", "The role's base color, shade 6."),
    (
        "\"primary.1\" to \"primary.9\"",
        "Shades generated from it.",
    ),
    (
        "\"primary-contrast\"",
        "Black or white, whichever reads on it.",
    ),
    (
        "\"xs\" to \"xxl\"",
        "A step of the property's scale: spacing for padding, margin and gap, radius for corners.",
    ),
    ("\"-sm\"", "The same step, negated."),
    ("anything else", "CSS text, untouched."),
];

/// Where each kind of rule lands, weakest first.
const LAYERS: [(&str, &str); 5] = [
    ("lsx-base", "The theme's reset and its body rules."),
    ("lsx-framework", "Each component's own styling."),
    (
        "lsx-user-static",
        "Your sx prop. Beats the component's own styling.",
    ),
    (
        "lsx-user-custom",
        "Stylesheets registered with use_stylesheet().",
    ),
    (
        "unlayered",
        "Your own CSS files, through class. Beats every layer.",
    ),
];

type Pair = (&'static str, &'static str);

/// A two-column table whose first column is code.
#[component]
fn CodeTable(label: &'static str, head: [&'static str; 2], rows: Vec<Pair>) -> Element {
    rsx! {
        Table {
            aria_label: label,
            data: rows,
            columns: vec![
                column(head[0])
                    .value(|row: &Pair| row.0)
                    .render(|row: &Pair| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                column(head[1]).value(|row: &Pair| row.1),
            ],
        }
    }
}

#[component]
pub fn StylingPage() -> Element {
    rsx! {
        DocPage {
            title: "Styling",
            markdown: "/md/styling.md",
            lead: rsx! {
                Text {
                    "Every component takes the same styling props. "
                    Code { source: "sx()" }
                    " is not an inline style. Identical declarations share one class, emitted "
                    "once, so a thousand rows styled alike cost one rule, and hover and media "
                    "queries work."
                }
                CodeTable { label: "Styling props", head: ["Prop", "For"], rows: PROPS.to_vec() }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Box",
                    title: "Usage",
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
                Text { "A value is a plain string. The ones the theme knows resolve against it." }
                CodeTable { label: "Theme values", head: ["Value", "Resolves to"], rows: VALUES.to_vec() }
                CodeBlock { source: THEME_VALUES, language: "rust" }
            }

            DocSection {
                title: "States",
                Text {
                    "Fold every variant into one "
                    Code { source: "sx" }
                    " with "
                    Code { source: "when" }
                    ", and pick one with the "
                    Code { source: "states" }
                    " prop. Every libero component handles its own size and variant this way."
                }
                Demo {
                    component: "Box",
                    title: "States",
                    children_text: "Badge",
                    fixed: vec![STATES_SX.to_string()],
                    controls: vec![
                        Control::toggle("state", ["default", "active", "danger"]).labels(["Default", "Active", "Danger"]).code(
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
                    Code { source: "selector" }
                    " takes any selector text, with "
                    Code { source: "&" }
                    " standing for the element. "
                    Code { source: "hover" }
                    ", "
                    Code { source: "focus" }
                    " and "
                    Code { source: "focus_visible" }
                    " are shorthands for it."
                }
                CodeBlock { source: SELECTORS, language: "rust" }
            }

            DocSection {
                title: "Responsive",
                Text {
                    "Write the small screen first, then override upwards with "
                    Code { source: "breakpoint" }
                    ", smallest first. For a single property, "
                    Code { source: "bp()" }
                    " puts the steps inside the value."
                }
                CodeBlock { source: RESPONSIVE, language: "rust" }
                CodeBlock { source: RESPONSIVE_VALUE, language: "rust" }
                Table {
                    aria_label: "Breakpoints",
                    data: vec![Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl, Size::Xxl],
                    columns: vec![
                        column("Size")
                            .value(|size: &Size| size.as_str())
                            .render(|size: &Size| rsx! { Code { source: size.as_str() } }),
                        column("min-width").value(|size: &Size| size.breakpoint_value()),
                    ],
                }
            }

            DocSection {
                title: "Media and container queries",
                Text {
                    Code { source: "media" }
                    " nests styles under any "
                    Code { source: "@media" }
                    " query, mostly for reduced motion. Nest it inside a "
                    Code { source: "when" }
                    ", never the other way round, or the state's rule wins."
                }
                CodeBlock { source: MEDIA, language: "rust" }
                CodeBlock { source: MEDIA_NESTING, language: "rust" }
                Text {
                    Code { source: "container_query" }
                    " asks about a named ancestor instead of the window, for a component in "
                    "a sidebar, a grid cell or a card. A container no longer takes its width "
                    "from its content, so never make a shrink-to-fit box one."
                }
                CodeBlock { source: CONTAINER, language: "rust" }
            }

            DocSection {
                title: "class and attributes",
                Text {
                    Code { source: "class" }
                    " adds your own class names beside "
                    Code { source: "sx" }
                    ", and every other attribute or event goes to the rendered tag."
                }
                CodeBlock { source: ESCAPE_HATCHES, language: "rust" }
            }

            DocSection {
                title: "Cascade layers",
                Text {
                    "Libero writes its rules into CSS layers. A later layer wins whatever its "
                    "selectors, so your "
                    Code { source: "sx" }
                    " always beats a component's own styling."
                }
                CodeBlock { source: LAYER_ORDER, language: "css" }
                CodeTable { label: "Cascade layers", head: ["Layer", "What it holds"], rows: LAYERS.to_vec() }
                Text {
                    "A stylesheet of your own is in no layer, so "
                    Code { source: "class" }
                    " beats everything. Reach for "
                    Code { source: "sx" }
                    " first. To rank your own layers among libero's, declare the order "
                    "yourself before libero mounts."
                }
                CodeBlock { source: APP_LAYER_ORDER, language: "css" }
                Text {
                    "Declaring the order also makes Tailwind v4 work: its reset under "
                    "libero's components, its utilities over them. Put it before "
                    Code { source: "@import \"tailwindcss\"" }
                    "."
                }
                CodeBlock { source: TAILWIND_LAYER_ORDER, language: "css" }
            }

            DocSection {
                title: "Static sx",
                Text {
                    "Styling that doesn't depend on props can live in a "
                    Code { source: "StaticSx" }
                    ", built once per process instead of on every render. Every constant "
                    Code { source: "sx" }
                    " inside libero is one, and your own components should copy that."
                }
                CodeBlock { source: STATIC_SX, language: "rust" }
            }

            DocSection {
                id: "style-api",
                title: "Style API",
                Text {
                    Code { source: "sx" }
                    " styles a component's root. To reach an element inside it, a multi-part "
                    "component names its parts in an enum, such as "
                    Code { source: "AlertPart" }
                    ", and takes a "
                    Code { source: "parts" }
                    " prop keyed by it. The component's page lists its parts in a Style API tab."
                }
                Text {
                    "Each part carries a "
                    Code { source: "data-slot" }
                    " attribute. The names are a stable contract, so your own CSS can match "
                    Code { source: "[data-slot='title']" }
                    " too. A part is matched as a child of the root, through each part above it, "
                    "never as any descendant: a component of the same kind nested inside keeps "
                    "its own styles."
                }
                Text {
                    Code { source: "parts" }
                    " compiles into the root's "
                    Code { source: "sx" }
                    " class, so it beats the component's own styling. Where the instance "
                    Code { source: "sx" }
                    " styles the same part, "
                    Code { source: "sx" }
                    " wins. "
                    Code { source: "StaticParts" }
                    " builds the parts once, as "
                    Code { source: "StaticSx" }
                    " does."
                }
                CodeBlock { source: PARTS, language: "rust" }
                Text {
                    "Parts a component renders in a portal (a "
                    Code { source: "Dialog" }
                    ", a "
                    Code { source: "Menu" }
                    ", a "
                    Code { source: "Tooltip" }
                    ") sit outside its root, so "
                    Code { source: "parts" }
                    " cannot reach them. A field's portaled dropdown, such as a "
                    Code { source: "Select" }
                    "'s list, takes its own "
                    Code { source: "dropdown_parts" }
                    " prop instead."
                }
            }
        }
    }
}
