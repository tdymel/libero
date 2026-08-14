use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Backdrop, Box, Button, Container, Divider, Flex, FocusTrap, FocusTrapInitialFocus,
        HelloWorld, Image, List, ListItem, Option, Select, Text, Title, TitleVariant,
        VisuallyHidden, states,
    },
    sx::{StaticSx, sx},
    theme::Size,
};

struct Article {
    title: &'static str,
    date: &'static str,
    description: &'static str,
    image: &'static str,
}

const ARTICLES: [Article; 3] = [
    Article {
        title: "Designing With Purpose",
        date: "March 3, 2026",
        description: "A look at how intentional design choices shape the way people experience software, from first impressions to daily use.",
        image: "https://picsum.photos/id/1011/200/200",
    },
    Article {
        title: "The Quiet Craft of Refactoring",
        date: "February 18, 2026",
        description: "Why the unglamorous work of cleaning up code is often what separates maintainable systems from fragile ones.",
        image: "https://picsum.photos/id/1015/200/200",
    },
    Article {
        title: "Notes From a Long Hike",
        date: "January 27, 2026",
        description: "Reflections on slowing down, paying attention, and what mountain trails can teach us about patience.",
        image: "https://picsum.photos/id/1018/200/200",
    },
];

static BOX_SX: StaticSx = StaticSx::new(|| {
    sx().padding_top(Size::Xl)
        .when("hidden", sx().background("secondary.1"))
});

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_hello_world = use_signal(|| true);
    let mut large_gap = use_signal(|| true);
    let mut wrap_group = use_signal(|| true);
    let mut click_count = use_signal(|| 0);
    let mut fruit = use_signal(|| "apple".to_string());
    let mut backdrop_open = use_signal(|| false);
    let mut focus_trap_active = use_signal(|| false);
    let mut focus_trap_initial_active = use_signal(|| false);
    let gap = if large_gap() { "xl" } else { "sm" }.to_string();

    rsx! {
        LiberoProvider {
            button {
                onclick: move |_| {
                    show_hello_world.toggle();
                },
                if show_hello_world() { "Hide Hello World" } else { "Show Hello World" }
            }

            button {
                onclick: move |_| {
                    large_gap.toggle();
                },
                if large_gap() { "Use small gap" } else { "Use large gap" }
            }

            button {
                onclick: move |_| {
                    wrap_group.toggle();
                },
                if wrap_group() { "Disable group wrap" } else { "Enable group wrap" }
            }

            Flex {
                sx: &BOX_SX,
                align: "start",
                gap: gap.clone(),
                states: states().with("hidden", !show_hello_world()),
                if show_hello_world() {
                    HelloWorld {}
                }
                HelloWorld {}
            }

            Flex {
                sx: sx().background("blue"),
                direction: "row",
                gap: gap,
                justify: "flex-end",
                wrap: wrap_group(),
                for index in 1..=8 {
                    button { "Group item {index}" }
                }
            }

            Container {
                class: "playground-container".to_string(),
                size: Size::Md,
                gutters: Size::Sm,
                sx: sx().with("border", "1px solid var(--lsx-grey-4)"),
                p { "Container example (size: md, gutters: sm)" }
                p { "Uses theme-aware size + gutters overrides." }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: TitleVariant::H1,
                    "H1 Heading"
                }
                Title {
                    variant: "h2",
                    "H2 Heading"
                }
                Title {
                    variant: "h3",
                    "H3 Heading"
                }
                Title {
                    variant: "h4",
                    "H4 Heading"
                }
                Title {
                    variant: "h5",
                    "H5 Heading"
                }
                Title {
                    variant: "h6",
                    "H6 Heading"
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    size: "h1",
                    "H2 element with H1 styling"
                }
                Title {
                    variant: "h3",
                    size: "h2",
                    "H3 element with H2 styling"
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h3",
                    "Text Sizes"
                }
                Text {
                    size: "xs",
                    "Extra small text"
                }
                Text {
                    size: "sm",
                    "Small text"
                }
                Text {
                    size: "md",
                    "Medium text (default)"
                }
                Text {
                    size: "lg",
                    "Large text"
                }
                Text {
                    size: "xl",
                    "Extra large text"
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h3",
                    "Text with span"
                }
                p {
                    "This is a paragraph with "
                    Text {
                        size: "lg",
                        span: true,
                        "highlighted text"
                    }
                    " inside it."
                }
            }
            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    "Divider Examples"
                }
                p {
                    "Simple horizontal divider:"
                }
                Divider {}
                p {
                    "Divider with label (center):"
                }
                Divider {
                    "OR"
                }
                p {
                    "Divider with label (left):"
                }
                Divider {
                    label_position: "start",
                    "Start Label"
                }
                p {
                    "Divider with label (end):"
                }
                Divider {
                    label_position: "end",
                    "End Label"
                }
                p {
                    "Vertical dividers:"
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Flex {
                        direction: "row",
                        sx: sx().height("100px"),
                        "Left"
                        Divider {
                            vertical: true,
                        }
                        "Right"
                    }
                    Flex {
                        direction: "row",
                        sx: sx().height("100px"),
                        "Left"
                        Divider {
                            vertical: true,
                            "OR"
                        }
                        "Right"
                    }
                    Flex {
                        direction: "row",
                        sx: sx().height("100px"),
                        "Left"
                        Divider {
                            vertical: true,
                            label_position: "start",
                            "Top"
                        }
                        "Right"
                    }
                    Flex {
                        direction: "row",
                        sx: sx().height("100px"),
                        "Left"
                        Divider {
                            vertical: true,
                            label_position: "end",
                            "Bottom"
                        }
                        "Right"
                    }
                }
                p {
                    "Spacing defaults to none (plain and labeled dividers now match):"
                }
                "Above"
                Divider {}
                Divider {
                    "OR"
                }
                "Below"
                p {
                    "Horizontal spacing (\"md\"), plain and labeled alike:"
                }
                "Above"
                Divider {
                    spacing: "md",
                }
                Divider {
                    spacing: "md",
                    "OR"
                }
                "Below"
                p {
                    "Horizontal spacing (\"xl\"):"
                }
                "Above"
                Divider {
                    spacing: "xl",
                    "OR"
                }
                "Below"
                p {
                    "Vertical spacing (\"md\"), applied left/right instead:"
                }
                Flex {
                    direction: "row",
                    sx: sx().height("100px"),
                    "Left"
                    Divider {
                        vertical: true,
                        spacing: "md",
                    }
                    "Middle"
                    Divider {
                        vertical: true,
                        spacing: "md",
                        "OR"
                    }
                    "Right"
                }
                p {
                    "Color: bare name defaults to shade 3, explicit shade is respected:"
                }
                "Above"
                Divider {
                    color: "primary",
                }
                Divider {
                    color: "primary",
                    "primary (shade 3)"
                }
                "Below"
                "Above"
                Divider {
                    color: "success.7",
                }
                Divider {
                    color: "success.7",
                    "success.7"
                }
                "Below"
                Title {
                    variant: "h2",
                    "VisuallyHidden Examples"
                }
                p {
                    "Icon-only button with a visually hidden label (inspect the DOM, or use a screen reader, to see it):"
                }
                button {
                    "★"
                    VisuallyHidden {
                        "Add to favorites"
                    }
                }
                p {
                    "Extra context appended to visible text:"
                }
                Text {
                    "4.5 "
                    VisuallyHidden {
                        "out of 5 stars"
                    }
                }
                Title {
                    variant: "h2",
                    "Image Examples"
                }
                p {
                    "Default (fills parent, object-fit: cover):"
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Box {
                        sx: sx().width("160px").height("120px"),
                        Image {
                            src: "https://picsum.photos/id/1015/400/300",
                            alt: "A river winding through a mountain valley",
                        }
                    }
                    Box {
                        sx: sx().width("160px").height("120px"),
                        Image {
                            src: "https://picsum.photos/id/1025/400/300",
                            fit: "contain",
                            alt: "A dog",
                        }
                    }
                    Box {
                        sx: sx().width("160px").height("120px"),
                        Image {
                            src: "https://picsum.photos/id/1035/400/300",
                            radius: "12px",
                            alt: "A forest lake",
                        }
                    }
                }
                p {
                    "Broken src falls back to fallback_src:"
                }
                Box {
                    sx: sx().width("160px").height("120px"),
                    Image {
                        src: "https://example.com/this-image-does-not-exist.jpg",
                        fallback_src: "https://picsum.photos/id/1043/400/300".to_string(),
                        alt: "Fallback placeholder image",
                    }
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    "Button Examples"
                }
                p {
                    "Variants (default color, default variant is outlined):"
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", "Filled" }
                    Button { "Outlined" }
                    Button { variant: "text", "Text" }
                }
                p {
                    "Colors (bare color defaults to shade 6, explicit shade honored exactly):"
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", color: "primary", "Primary" }
                    Button { variant: "filled", color: "secondary", "Secondary" }
                    Button { variant: "filled", color: "success.8", "Success.8" }
                    Button { variant: "outlined", color: "error", "Error" }
                }
                p {
                    "Sizes (xs .. xl, default md):"
                }
                Flex {
                    direction: "row",
                    align: "center",
                    gap: "md",
                    Button { variant: "filled", size: "xs", "Extra small" }
                    Button { variant: "filled", size: "sm", "Small" }
                    Button { variant: "filled", size: "md", "Medium" }
                    Button { variant: "filled", size: "lg", "Large" }
                    Button { variant: "filled", size: "xl", "Extra large" }
                }
                p {
                    "Radius (xs .. xl, default md, plus a fully round pill):"
                }
                Flex {
                    direction: "row",
                    align: "center",
                    gap: "md",
                    Button { variant: "filled", radius: "xs", "Radius xs" }
                    Button { variant: "filled", radius: "xl", "Radius xl" }
                    Button { variant: "filled", sx: sx().border_radius("999px"), "Pill" }
                }
                p {
                    "Disabled:"
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", disabled: true, "Filled disabled" }
                    Button { variant: "outlined", disabled: true, "Outlined disabled" }
                }
                p {
                    "Full width:"
                }
                Button { variant: "filled", full_width: true, "Full width button" }
                p {
                    "Onclick (clicked {click_count} times):"
                }
                Button {
                    variant: "filled",
                    onclick: move |_| click_count += 1,
                    "Click me"
                }
                p {
                    "As a link (href + target, same look as a regular button):"
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button {
                        variant: "filled",
                        href: "https://dioxuslabs.com",
                        target: "_blank",
                        "Open Dioxus docs"
                    }
                    Button {
                        variant: "outlined",
                        href: "https://dioxuslabs.com",
                        disabled: true,
                        "Disabled link"
                    }
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    "List Examples"
                }
                p {
                    "Simple list:"
                }
                List {
                    ListItem { "Item one" }
                    ListItem { "Item two" }
                    ListItem { "Item three" }
                }
                p {
                    "Nested list:"
                }
                List {
                    ListItem { "Fruits" }
                    ListItem {
                        List {
                            ListItem { "Apple" }
                            ListItem { "Banana" }
                        }
                    }
                    ListItem { "Vegetables" }
                    ListItem {
                        List {
                            ListItem { "Carrot" }
                            ListItem { "Potato" }
                        }
                    }
                }
                p {
                    "List item with arbitrary content:"
                }
                List {
                    ListItem {
                        Flex {
                            direction: "row",
                            align: "center",
                            gap: "sm",
                            Text { "Custom row content" }
                            Button { variant: "text", size: "xs", "Action" }
                        }
                    }
                }
                p {
                    "Article list (image + title/description/date, separated by dividers):"
                }
                List {
                    for (index , article) in ARTICLES.iter().enumerate() {
                        ListItem {
                            Flex {
                                // Mobile-first: stacked, top-image layout by default
                                // (smartphone width). From the "xs" breakpoint up
                                // (tablet and wider), switch to image-left/text-right.
                                sx: sx()
                                    .width("100%")
                                    .flex_direction("column")
                                    .align_items("stretch")
                                    .gap("md")
                                    .breakpoint(
                                        Size::Xs,
                                        sx().flex_direction("row").align_items("flex-start"),
                                    ),
                                Box {
                                    sx: sx()
                                        .width("100%")
                                        .aspect_ratio("2 / 1")
                                        .flex_shrink("0")
                                        .breakpoint(Size::Xs, sx().width("140px").height("100px")),
                                    Image {
                                        src: article.image,
                                        radius: "sm",
                                        alt: "Cover image for {article.title}",
                                    }
                                }
                                Flex {
                                    direction: "column",
                                    gap: "xs",
                                    // Lets the text column shrink below its content's natural
                                    // width instead of overflowing/forcing the row to wrap.
                                    sx: sx().flex("1").min_width("0"),
                                    Title { variant: "h4", "{article.title}" }
                                    Text {
                                        size: "sm",
                                        sx: sx().color("grey.6"),
                                        "{article.date}"
                                    }
                                    Text { "{article.description}" }
                                }
                            }
                        }
                        if index + 1 < ARTICLES.len() {
                                Divider {}
                        }
                    }
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    "Select Examples"
                }
                p {
                    "Default (with label, selected: {fruit}):"
                }
                Select {
                    label: "Favorite fruit",
                    value: fruit(),
                    onchange: move |value| fruit.set(value),
                    Option { value: "apple", "Apple" }
                    Option { value: "banana", "Banana" }
                    Option { value: "cherry", "Cherry" }
                }
                p {
                    "Sizes (xs .. xl, default md):"
                }
                Flex {
                    direction: "row",
                    align: "flex-end",
                    wrap: true,
                    gap: "md",
                    Select {
                        size: "xs",
                        value: "a",
                        Option { value: "a", "Extra small" }
                    }
                    Select {
                        size: "sm",
                        value: "a",
                        Option { value: "a", "Small" }
                    }
                    Select {
                        size: "md",
                        value: "a",
                        Option { value: "a", "Medium" }
                    }
                    Select {
                        size: "lg",
                        value: "a",
                        Option { value: "a", "Large" }
                    }
                    Select {
                        size: "xl",
                        value: "a",
                        Option { value: "a", "Extra large" }
                    }
                }
                p {
                    "Radius override:"
                }
                Select {
                    radius: "xl",
                    value: "a",
                    Option { value: "a", "Rounder corners" }
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    "Backdrop Examples"
                }
                p {
                    "Click to open a backdrop (fades in/out, click it to close):"
                }
                Button {
                    variant: "filled",
                    onclick: move |_| backdrop_open.set(true),
                    "Open backdrop"
                }
                Backdrop {
                    open: backdrop_open(),
                    onclick: move |_| backdrop_open.set(false),
                    Text {
                        sx: sx().color("white"),
                        "Click anywhere to close"
                    }
                }
            }

            Flex {
                sx: sx().padding_top(Size::Xl),
                Title {
                    variant: "h2",
                    "FocusTrap Examples"
                }
                p {
                    "Tab/Shift+Tab cycles only through the three inputs below while active. Second input starts focused (data-autofocus):"
                }
                Button {
                    variant: "filled",
                    onclick: move |_| focus_trap_active.toggle(),
                    if focus_trap_active() { "Deactivate focus trap" } else { "Activate focus trap" }
                }
                FocusTrap {
                    active: focus_trap_active(),
                    Flex {
                        direction: "column",
                        gap: "sm",
                        sx: sx().padding_top(Size::Sm).max_width("300px"),
                        input { placeholder: "First input" }
                        input { placeholder: "Second input (autofocus)", "data-autofocus": true }
                        input { placeholder: "Third input" }
                    }
                }
                p {
                    "With FocusTrap.InitialFocus (nothing inside gets auto-focused; tabbing away from it drops it from the tab order):"
                }
                Button {
                    variant: "filled",
                    onclick: move |_| focus_trap_initial_active.toggle(),
                    if focus_trap_initial_active() { "Deactivate focus trap" } else { "Activate focus trap" }
                }
                FocusTrap {
                    active: focus_trap_initial_active(),
                    Flex {
                        direction: "column",
                        gap: "sm",
                        sx: sx().padding_top(Size::Sm).max_width("300px"),
                        FocusTrapInitialFocus {}
                        input { placeholder: "First input" }
                        input { placeholder: "Second input" }
                    }
                }
            }
        }
    }
}
