use crate::components::DocPage;
use crate::icons::{CheckmarkIcon, ChevronIcon, CodeIcon};
use dioxus::prelude::*;
use libero::{
    components::{Alert, Anchor, Badge, Divider, Flex, Icon, List, ListItem, Paper, Text, Title},
    sx::sx,
    theme::Size,
};

/// The principles in priority order, as the strip names them.
const RANKING: [&str; 4] = [
    "Developer experience",
    "Accessibility",
    "Batteries included",
    "Simple yet modern",
];

#[component]
pub fn PhilosophyPage() -> Element {
    rsx! {
        DocPage {
            title: "Philosophy",
            markdown: "/md/philosophy.md",
            lead: rsx! {
                Text {
                    "Four principles decide what goes into libero and how it is shaped. They are "
                    "ranked: when two pull in different directions, the higher one wins."
                }
                Ranking {}
            },

            Flex { direction: "row", align: "stretch", gap: "lg", wrap: "wrap",
                Principle {
                    number: 1,
                    title: "Developer experience first",
                    summary: "An API that is small, clear and hard to misuse.",
                    icon: rsx! { CodeIcon {} },
                    why: "AI writes a lot of code now, and someone still has to read it, review it \
                          and change it next month. Code that is hard to get wrong is easier for a \
                          person to check and for a model to get right.",
                    Point { "Let the compiler catch mistakes before a reviewer has to." }
                    Point { "Keep one way of doing things, the same across every component." }
                    Point { "Write docs that you and your AI assistant can copy from, with examples that build." }
                }
                Principle {
                    number: 2,
                    title: "Accessibility second",
                    summary: "Everyone should be able to use what you build.",
                    icon: rsx! { AccessibilityIcon {} },
                    why: "It helps more people than screen reader users. Keyboard support helps \
                          anyone who would rather not reach for the mouse. A state that does not rely \
                          on colour still reads on a dim screen in the sun. Motion that stops on \
                          request helps anyone it makes dizzy.",
                    limit: rsx! {
                        Alert { title: "A known limit", icon: rsx! { InfoIcon {} },
                            Text {
                                "Windows High Contrast mode is supported only in part. The "
                                Anchor { to: crate::Route::AccessibilityPage {}, "Accessibility" }
                                " page says what holds."
                            }
                        }
                    },
                    Point { "Follow WAI-ARIA, the APG patterns and WCAG. A deviation needs a reason." }
                    Point { "Aim for WCAG 2.2 AA." }
                    Point { "Never show a state by colour alone." }
                    Point { "Speak the user's language: the words components say on their own translate, with English and German built in." }
                }
                Principle {
                    number: 3,
                    title: "Batteries included",
                    summary: "What a typical app needs, in one place.",
                    icon: rsx! { BatteryIcon {} },
                    why: "Starting an app should not mean hunting for a date picker. Your time goes \
                          into your app, not into stitching libraries together.",
                    Point { "Cover the everyday needs, forms and overlays included: more than 100 components." }
                    Point { "Work the same on the web and natively." }
                    Point { "Be honest where a platform falls short: a missing feature is absent, not broken." }
                }
                Principle {
                    number: 4,
                    title: "Simple yet modern",
                    summary: "A clean look that stays out of the way.",
                    icon: rsx! { SparkleIcon {} },
                    why: "Your app should look like your app. A quiet default gets you started, and \
                          when you want your own look, nothing stands in the way.",
                    Point { "Aim for a default that fits most apps as it is." }
                    Point { "Make changing the look ordinary Rust, not a fight with the library." }
                    Point { "Offer light and dark from the start." }
                    Point { "Let your own styles win over ours." }
                }
            }
        }
    }
}

/// The priority order at a glance; the chevrons are named so it reads as a sentence.
#[component]
fn Ranking() -> Element {
    rsx! {
        Flex { direction: "row", align: "center", gap: "xs", wrap: "wrap",
            for (index, name) in RANKING.iter().enumerate() {
                Flex { key: "{name}", direction: "row", align: "center", gap: "xs",
                    if index > 0 {
                        Icon { variant: "standard", size: "sm", aria_label: "before", ChevronIcon {} }
                    }
                    Badge { variant: "tonal", size: "lg", sx: sx().text_transform("none"), "{index + 1}. {name}" }
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
                    Icon { variant: "tonal", color: "primary", size: "xl", radius: "md", {icon} }
                    Title { size: "lg", component: "h2", sx: sx().flex("1").min_width("0"), "{title}" }
                    // The cards' order is the ranking; the number repeats it for the eye.
                    Badge { circle: true, size: "lg", "aria-hidden": "true", "{number}" }
                }
                Text { size: "lg", sx: sx().font_weight("600"), "{summary}" }
                Label { "What we do" }
                List { size: "sm", {children} }
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
        Text { size: "sm", sx: sx().font_weight("600"), {children} }
    }
}

#[component]
fn Point(children: Element) -> Element {
    rsx! {
        ListItem {
            Flex { direction: "row", align: "flex-start", gap: "xs", wrap: "nowrap",
                Icon { variant: "standard", color: "primary", size: "sm", sx: sx().margin_top("4px"), CheckmarkIcon {} }
                Text { component: "span", sx: sx().flex("1").min_width("0"), {children} }
            }
        }
    }
}

#[component]
fn AccessibilityIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "16", cy: "4", r: "1" }
            path { d: "m18 19 1-7-6 1" }
            path { d: "m5 8 3-3 5.5 3-2.36 3.5" }
            path { d: "M4.24 14.5a5 5 0 0 0 6.88 6" }
            path { d: "M13.76 17.5a5 5 0 0 0-6.88-6" }
        }
    }
}

#[component]
fn BatteryIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "2", y: "6", width: "16", height: "12", rx: "2" }
            path { d: "M22 14v-4" }
            path { d: "M6 10v4" }
            path { d: "M10 10v4" }
            path { d: "M14 10v4" }
        }
    }
}

#[component]
fn SparkleIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 3l1.9 5.8L20 11l-6.1 2.2L12 19l-1.9-5.8L4 11l6.1-2.2z" }
            path { d: "M19 3v4" }
            path { d: "M21 5h-4" }
        }
    }
}

#[component]
fn InfoIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M12 16v-4" }
            path { d: "M12 8h.01" }
        }
    }
}
