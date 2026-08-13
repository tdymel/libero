use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Container, HelloWorld, Stack, Text, Title, TitleVariant, states},
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

            Stack {
                sx: &BOX_SX,
                align: "start",
                gap: gap.clone(),
                states: states().with("hidden", !show_hello_world()),
                if show_hello_world() {
                    HelloWorld {}
                }
                HelloWorld {}
            }

            Stack {
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

            Stack {
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

            Stack {
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

            Stack {
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

            Stack {
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
        }
    }
}
