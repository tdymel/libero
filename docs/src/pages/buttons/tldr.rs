use crate::components::{Control, Demo, DemoValues, DocPage, SITE, TLDR_PROMPT, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Input, Text, Tldr},
    hooks::use_localization,
};

#[component]
pub fn TldrPage() -> Element {
    rsx! {
        DocPage {
            title: "Tldr",
            source: "libero/src/components/buttons/tldr.rs",
            markdown: "/md/tldr.md",
            properties: vec![props("Tldr", vec![
                prop("url", "String")
                    .doc("Required. The page's absolute address, e.g. its markdown mirror. The assistant is asked to read it."),
                prop("providers", "Vec<SummaryProvider>")
                    .default("SummaryProvider::defaults()")
                    .doc("The menu's links, in order: ChatGPT, Google AI, Claude and Perplexity. Filter the vec to drop one, push a `SummaryProvider` to add one."),
                prop("prompt", "String")
                    .doc("What the assistant is asked. `{url}` is filled with `url`; in an `rsx!` string literal write `{{url}}`. Unset, `TldrLabels::prompt` from the localization."),
                prop("label", "String")
                    .doc("The trigger's text. Unset, `TldrLabels::label` (\"TLDR\")."),
                prop("icon_only", "bool")
                    .default("false")
                    .doc("Draws only the sparkles, named by `aria_label`."),
                prop("aria_label", "String")
                    .doc("Names the icon-only trigger. Unset, `TldrLabels::icon_only` (\"Summarize with AI\")."),
                prop("variant", "Variant")
                    .default("outlined")
                    .doc("The trigger's visual style."),
                prop("size", "Size")
                    .default("sm")
                    .doc("The trigger's size step, the icon-only one too."),
                prop("radius", "Size")
                    .doc("Corner radius, independent of `size`. Unset, the trigger's own: `xl` on the labelled chip, `sm` icon-only."),
                prop("color", "ThemeAwareValue")
                    .default("neutral")
                    .doc("The trigger's accent color. A theme color name or any CSS color."),
            ])],
            accessibility: a11y()
                .key(["Enter", "Space", "ArrowDown"], "On the trigger: opens the menu on its first link.")
                .key(["ArrowUp"], "On the trigger: opens the menu on its last link.")
                .key(["ArrowDown", "ArrowUp"], "Moves between the links.")
                .key(["Enter", "Space"], "On a link: follows it in a new tab and closes the menu.")
                .key(["Escape"], "Closes the menu and returns focus to the trigger.")
                .handles([
                    "It is `Menu`'s menu button: the trigger has `aria-haspopup` and `aria-expanded`, the links sit in a group named \"Summarize with\".",
                    "Each link is an `<a role=\"menuitem\">` with a real `href`, so middle-click and the context menu work. It opens in a new tab, `rel=\"noopener noreferrer\"`.",
                    "The provider marks are hidden from assistive technology; the provider's name is the link's name.",
                    "The icon-only trigger is named \"Summarize with AI\". Every word comes from `TldrLabels` in the localization.",
                ])
                .must([
                    "Give a custom provider a name that says which service it opens.",
                ]),
            lead: rsx! {
                Text {
                    "A menu of links that hand a page to an AI assistant to summarize. Each link opens "
                    "the assistant's new chat with a prompt naming "
                    Code { source: "url" }
                    ". No script and no API key: the prompt rides in the link's query."
                }
                Text {
                    "This site's pages use it beside \"View as markdown\", with a prompt of their own."
                }
            },
            Demo {
                component: "Tldr",
                children_text: "",
                controls: vec![
                    Control::toggle("url", ["tldr", "menu", "button"])
                        .labels(["Tldr", "Menu", "Button"])
                        .default("tldr")
                        .code(|_, values| vec![format!("url: \"{}\"", page_url(&values.str("url")))]),
                    // The docs' own prompt is long; the default prints nothing.
                    Control::toggle("prompt", ["default", "docs"])
                        .labels(["Default", "Docs"])
                        .default("default")
                        .code(|_, values| match values.str("prompt").as_str() {
                            "docs" => vec![format!(
                                "prompt: {:?}",
                                TLDR_PROMPT.replace("{url}", "{{url}}")
                            )],
                            _ => vec![],
                        }),
                    Control::switch("icon_only"),
                    Control::toggle("variant", ["outlined", "filled", "tonal", "standard"])
                        .labels(["Outlined", "Filled", "Tonal", "Standard"])
                        .default("outlined"),
                    Control::color("color").default("neutral"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    // Unset is the trigger's own: `xl` on the chip, `sm` icon-only.
                    Control::slider("radius", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("auto")
                        .code(|_, values| match values.str("radius").as_str() {
                            "auto" => vec![],
                            radius => vec![format!("radius: {radius:?}")],
                        }),
                ],
                render: move |values: DemoValues| {
                    let url = page_url(&values.str("url"));
                    let docs = values.str("prompt") == "docs";
                    rsx! {
                        Flex {
                            direction: "column",
                            gap: "md",
                            Tldr {
                                url: url.clone(),
                                prompt: docs.then(|| TLDR_PROMPT.to_string()),
                                icon_only: values.str("icon_only") == "true",
                                variant: values.str("variant"),
                                color: match values.str("color").as_str() {
                                    "neutral" => Input::None,
                                    color => Input::from(color),
                                },
                                size: values.str("size"),
                                radius: match values.str("radius").as_str() {
                                    "auto" => Input::None,
                                    radius => Input::from(radius),
                                },
                            }
                            PromptPreview { url, docs }
                        }
                    }
                },
            }
        }
    }
}

fn page_url(page: &str) -> String {
    format!("{SITE}/md/{page}.md")
}

/// What the menu's links ask the assistant, `{url}` filled.
#[component]
fn PromptPreview(url: String, docs: bool) -> Element {
    let prompt = if docs {
        TLDR_PROMPT
    } else {
        use_localization().tldr.prompt
    };
    let asked = prompt.replace("{url}", &url);
    rsx! {
        Text { size: "sm", "Prompt: {asked}" }
    }
}
