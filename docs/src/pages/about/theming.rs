use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Table, Text, column};

/// A part of the theme, its fields, and what it changes.
type Part = (&'static str, &'static str, &'static str);

const PARTS: [Part; 4] = [
    (
        "Colors",
        "primary, secondary, success, error, warning, info",
        "One hex per role, with nine shades and a text color derived from it.",
    ),
    (
        "Page",
        "surface, ink",
        "The page and the text on it. A dark surface makes a dark theme.",
    ),
    (
        "Scales",
        "spacing, radius, elevation, font_size",
        "Every size word, gap, corner, shadow and font size.",
    ),
    (
        "Components",
        "button, dialog, ...",
        "Every prop a caller leaves unset.",
    ),
];

/// The fields of `Theme`: name, type, what it holds.
const FIELDS: [Part; 8] = [
    (
        "spacing",
        "Sizes<u8>",
        "Pixels per spacing step, xs to xxl. Every size word, gap and padding steps along it.",
    ),
    ("radius", "Sizes<u8>", "Pixels per radius step."),
    (
        "elevation",
        "Sizes<&'static str>",
        "One box-shadow per elevation step. Theme::DARK has a deeper scale, since a shadow on a dark page needs more alpha to show.",
    ),
    (
        "primary, secondary, success, error, warning, info, neutral, muted",
        "HexColor",
        "One hex per palette role. Nine shades, and a readable text color for each, are derived from it.",
    ),
    (
        "ink, surface",
        "HexColor",
        "The text color and the page it is set on. Every role is derived against surface.",
    ),
    (
        "font_smoothing",
        "bool",
        "Whether the reset asks for antialiased text.",
    ),
    (
        "gradient",
        "GradientDefaults",
        "The stops and angle of every gradient fill: Primary to Secondary at 45deg.",
    ),
    (
        "one field per component",
        "*Defaults",
        "For example button: ButtonDefaults. Each component's page lists its own struct.",
    ),
];

// snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] Home {} }
// snippet: item #[component] fn Home() -> Element { rsx! {} }
const CUSTOM_THEME: &str = r#"static THEME: Theme = Theme {
    primary: HexColor::new(0x7C3AED),
    spacing: Sizes::new(4, 8, 12, 16, 20, 24),
    ..Theme::DEFAULT
};

fn App() -> Element {
    rsx! {
        LiberoProvider {
            themes: &THEME,
            Router::<Route> {}
        }
    }
}"#;

const USING_COLORS: &str = r#"Text { sx: sx().color("error"), "Payment failed" }
Box { sx: sx().background("primary.1").padding("md").border_radius("md"), "Saved" }"#;

const COMPONENT_DEFAULTS: &str = r#"static THEME: Theme = Theme {
    button: ButtonDefaults {
        radius: Size::Xl,
        ..ButtonDefaults::DEFAULT
    },
    ..Theme::DEFAULT
};"#;

const SCALES: &str = r#"static THEME: Theme = Theme {
    font_size: Sizes::new("0.8rem", "0.9rem", "1rem", "1.2rem", "1.4rem", "1.6rem"),
    paper: PaperDefaults {
        glass_background: 85,
        glass_blur: "blur(20px)",
        ..PaperDefaults::DEFAULT
    },
    ..Theme::DEFAULT
};"#;

const GRADIENT: &str = r#"static THEME: Theme = Theme {
    gradient: GradientDefaults { from: Color::Info, to: Color::Success, deg: 90 },
    ..Theme::DEFAULT
};"#;

// snippet: ignore - the themes and `Route` are the app's own
const THEME_SET: &str = r#"LiberoProvider {
    themes: ThemeSet::new().light(&LIGHT).dark(&DARK),
    Router::<Route> {}
}"#;

const READING_THE_THEME: &str = r#"let theme = use_theme();
let gap = theme.spacing.get(Size::Md);"#;

#[component]
pub fn ThemingPage() -> Element {
    rsx! {
        DocPage {
            title: "Theming",
            markdown: "/md/theming.md",
            lead: rsx! {
                Text {
                    "A theme is one struct. Hand yours to "
                    Code { source: "LiberoProvider" }
                    " and every component follows it."
                }
                Table {
                    aria_label: "What a theme holds",
                    data: PARTS.to_vec(),
                    columns: vec![
                        column("Part").value(|row: &Part| row.0),
                        column("Fields")
                            .value(|row: &Part| row.1)
                            .render(|row: &Part| rsx! { Code { source: row.1 } }),
                        column("What it changes").value(|row: &Part| row.2),
                    ],
                }
            },

            DocSection {
                title: "A custom theme",
                Text {
                    "Name the fields you change. The rest come from "
                    Code { source: "Theme::DEFAULT" }
                    "."
                }
                CodeBlock { source: CUSTOM_THEME, language: "rust" }
            }

            DocSection {
                title: "Colors",
                Text {
                    "One hex per role is enough. Libero derives nine shades from it and keeps "
                    "text legible on each. In "
                    Code { source: "sx" }
                    ", a role name is its base color and "
                    Code { source: "primary.1" }
                    " to "
                    Code { source: "primary.9" }
                    " are the shades."
                }
                CodeBlock { source: USING_COLORS, language: "rust" }
            }

            DocSection {
                title: "Component defaults",
                Text {
                    "Every prop a caller leaves unset comes from the component's struct on "
                    "the theme. Pill-shaped buttons everywhere is one change here."
                }
                CodeBlock { source: COMPONENT_DEFAULTS, language: "rust" }
            }

            DocSection {
                title: "Type scale and glass",
                Text {
                    Code { source: "font_size" }
                    " holds six steps, "
                    Code { source: "xs" }
                    " to "
                    Code { source: "xxl" }
                    ". "
                    Code { source: "sx().font_size(\"sm\")" }
                    " and "
                    Code { source: "Text" }
                    " read it, so one change rescales the text. "
                    Code { source: "paper.glass_background" }
                    " is how much of the surface a "
                    Code { source: "glass" }
                    " Paper or Header keeps, in percent, and "
                    Code { source: "paper.glass_blur" }
                    " blurs what shows through. Stay at 70 or more, so text keeps its contrast."
                }
                CodeBlock { source: SCALES, language: "rust" }
            }

            DocSection {
                title: "Gradient",
                Text {
                    Code { source: "gradient" }
                    " is the fill of "
                    Code { source: "variant: \"gradient\"" }
                    ", of a gradient "
                    Code { source: "Paper" }
                    " and of gradient "
                    Code { source: "Text" }
                    ": two palette roles and an angle. Its label is picked to read on both "
                    "stops, in light and in dark. A component's "
                    Code { source: "color" }
                    " is the first stop, "
                    Code { source: "from" }
                    " only its fallback. Its "
                    Code { source: "gradient" }
                    " prop sets the second stop and the angle, as "
                    Code { source: "(\"secondary\", 45)" }
                    "."
                }
                CodeBlock { source: GRADIENT, language: "rust" }
            }

            DocSection {
                title: "Light and dark",
                Text {
                    "A "
                    Code { source: "ThemeSet" }
                    " pairs a light theme with a dark one. The default pairs "
                    Code { source: "Theme::DEFAULT" }
                    " with "
                    Code { source: "Theme::DARK" }
                    " and follows the system setting. A dark theme is one whose "
                    Code { source: "surface" }
                    " is dark and "
                    Code { source: "ink" }
                    " light. The roles adapt on their own. "
                    Code { source: "ThemeSet::CATALOGUE" }
                    " has ready-made pairs such as Catppuccin, Dracula and Nord."
                }
                CodeBlock { source: THEME_SET, language: "rust" }
                Text {
                    Code { source: "ThemeToggle" }
                    " is a ready-made switch."
                }
            }

            DocSection {
                title: "Reading the theme",
                Text {
                    Code { source: "use_theme()" }
                    " returns the active theme, for a value you need in Rust rather than in "
                    Code { source: "sx" }
                    "."
                }
                CodeBlock { source: READING_THE_THEME, language: "rust" }
            }

            DocSection {
                title: "Theme fields",
                Text {
                    Code { source: "Theme" }
                    " is about 2 KB and not "
                    Code { source: "Copy" }
                    ", so a stray by-value use cannot copy it silently. "
                    Code { source: "use_theme()" }
                    " hands out a "
                    Code { source: "&'static Theme" }
                    "."
                }
                Table {
                    aria_label: "Theme fields",
                    data: FIELDS.to_vec(),
                    columns: vec![
                        column("Field")
                            .value(|row: &Part| row.0)
                            .render(|row: &Part| rsx! { Code { source: row.0 } }),
                        column("Type")
                            .value(|row: &Part| row.1)
                            .render(|row: &Part| rsx! { Code { source: row.1 } }),
                        column("Description").value(|row: &Part| row.2),
                    ],
                }
            }
        }
    }
}
