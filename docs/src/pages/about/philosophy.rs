use crate::components::DocPage;
use dioxus::prelude::*;
use libero::{
    components::{
        Alert, Anchor, Badge, Divider, Flex, Icon, List, ListItem, Paper, Pictogram, Text, Title,
    },
    sx::sx,
    theme::Size,
};
use pictogram_icons_lucide as lucide;

#[component]
pub fn PhilosophyPage() -> Element {
    let components = crate::exports::COMPONENTS.len();

    rsx! {
        DocPage {
            title: "Philosophy",
            markdown: "/md/philosophy.md",
            lead: rsx! {
                Text {
                    "Four principles decide what goes into libero and how it is shaped. They are "
                    "ranked: when two pull in different directions, the higher one wins."
                }
            },

            Flex { direction: "row", align: "stretch", gap: "lg", wrap: "wrap",
                Principle {
                    number: 1,
                    title: "Developer experience",
                    summary: "An API that is small, clear and hard to misuse.",
                    icon: rsx! { Pictogram { icon: lucide::code::outlined } },
                    why: "AI writes a lot of code now, and someone still has to read it, review it \
                          and change it next month. Code that is hard to get wrong is easier for a \
                          person to check and for a model to get right.",
                    ListItem {"Let the compiler catch mistakes before a reviewer has to." }
                    ListItem {"Keep one way of doing things, the same across every component." }
                    ListItem {"Write docs that you and your AI assistant can copy from, with examples that build." }
                }
                Principle {
                    number: 2,
                    title: "Accessibility",
                    summary: "Everyone should be able to use what you build.",
                    icon: rsx! { Pictogram { icon: lucide::accessibility::outlined } },
                    why: "It helps more people than screen reader users. Keyboard support helps \
                          anyone who would rather not reach for the mouse. A state that does not rely \
                          on colour still reads on a dim screen in the sun. Motion that stops on \
                          request helps anyone it makes dizzy.",
                    limit: rsx! {
                        Alert { title: "A known limit", icon: rsx! { InfoIcon {} },
                            Text {
                                "Windows High Contrast mode is supported only in part. The "
                                // The page's link colour falls short on the tint; the alert's text reads.
                                Anchor {
                                    to: crate::Route::AccessibilityPage {},
                                    underline: "always",
                                    sx: sx().color("inherit"),
                                    "Accessibility"
                                }
                                " page says what holds."
                            }
                        }
                    },
                    ListItem {"Follow WAI-ARIA, the APG patterns and WCAG. A deviation needs a reason." }
                    ListItem {"Aim for WCAG 2.2 AA." }
                    ListItem {"Never show a state by colour alone." }
                    ListItem {"Speak the user's language: the words components say on their own translate, with English and German built in." }
                }
                Principle {
                    number: 3,
                    title: "Batteries included",
                    summary: "What a typical app needs, in one place.",
                    icon: rsx! { BatteryIcon {} },
                    why: "Starting an app should not mean hunting for a date picker. Your time goes \
                          into your app, not into stitching libraries together.",
                    ListItem {"Cover the everyday needs, forms and overlays included: {components} components." }
                    ListItem {"Work the same on the web and natively, and say where a platform falls short." }
                }
                Principle {
                    number: 4,
                    title: "Simple yet modern",
                    summary: "A clean look that stays out of the way.",
                    icon: rsx! { SparkleIcon {} },
                    why: "Your app should look like your app. A quiet default gets you started, and \
                          when you want your own look, nothing stands in the way.",
                    ListItem {"Aim for a default that fits most apps as it is." }
                    ListItem {"Make changing the look ordinary Rust, not a fight with the library." }
                    ListItem {"Offer light and dark from the start." }
                    ListItem {"Let your own styles win over ours." }
                }
            }
        }
    }
}

/// One principle as a numbered card: the ideal, what we do about it, and why.
#[component]
fn Principle(
    number: u8,
    title: &'static str,
    summary: &'static str,
    icon: Element,
    why: &'static str,
    #[props(default)] limit: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        Paper {
            sx: sx()
                .padding("lg")
                .flex("1 1 100%")
                .min_width("0")
                .breakpoint(Size::Md, sx().flex("1 1 calc(50% - 12px)")),
            Flex { direction: "column", gap: "md",
                Flex { direction: "row", align: "center", gap: "sm", wrap: "nowrap",
                    Icon {
                        variant: "tonal",
                        color: "primary",
                        size: "xl",
                        radius: "md",
                        {icon}
                    }
                    Title { size: "lg", component: "h2", sx: sx().flex("1").min_width("0"), "{title}" }
                    // The cards' order is the ranking; the number repeats it for the eye.
                    Badge { circle: true, size: "lg", "aria-hidden": "true", "{number}" }
                }
                Text { size: "lg", sx: sx().font_weight("600"), "{summary}" }
                Label { "What we do" }
                List {
                    size: "sm",
                    icon: rsx! { Icon { variant: "standard", color: "primary", size: "sm", Pictogram { icon: lucide::check::outlined } } },
                    {children}
                }
                if let Some(limit) = limit {
                    {limit}
                }
                Divider {}
                Label { "Why it matters" }
                Text { size: "sm", "{why}" }
            }
        }
    }
}

#[component]
fn Label(children: Element) -> Element {
    rsx! {
        // A heading under the card's h2, styled as the small bold line it was.
        Title { component: "h3", size: "sm", sx: sx().font_weight("600").margin("0"), {children} }
    }
}

#[component]
fn BatteryIcon() -> Element {
    rsx! {
        Pictogram { icon: lucide::battery::outlined }
    }
}

#[component]
fn SparkleIcon() -> Element {
    rsx! {
        Pictogram { icon: lucide::sparkles::outlined }
    }
}

#[component]
fn InfoIcon() -> Element {
    rsx! {
        Pictogram { icon: lucide::info::outlined }
    }
}
