use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Box, Button, Container, Divider, Flex, HelloWorld, Image, Text, Title, TitleVariant,
        VisuallyHidden, states,
    },
    sx::{StaticSx, sx},
    theme::Size,
};

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
            }
        }
    }
}
