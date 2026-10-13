use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Input, Share, Text};
use libero::use_theme;

/// The `label` switch's description, read after the name.
const LABEL: &str = "Link to libero's docs";

const URL: &str = "https://libero-ui.dev";

/// The visible text is a child, not a prop, so a control has to decide it here.
fn text_child(values: &DemoValues) -> String {
    match values.str("children").as_str() {
        "true" => "Share".to_string(),
        _ => String::new(),
    }
}

fn controls() -> Vec<Control> {
    vec![
        Control::toggle(
            "variant",
            ["filled", "tonal", "elevated", "outlined", "standard"],
        )
        .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]),
        // A bare `primary` is what an unset `color` resolves to, so that swatch prints nothing.
        Control::color("color"),
        Control::sizes("size").default("md"),
        Control::sizes("radius").default("sm"),
        Control::switch("label").code(|_, values| match values.str("label").as_str() {
            "true" => vec![format!("label: {LABEL:?}")],
            _ => vec![],
        }),
        Control::switch("children").code(|_, _| vec![]),
        Control::switch("disabled"),
    ]
}

fn color(values: &DemoValues) -> Input<libero::sx::ThemeAwareValue> {
    match values.str("color").as_str() {
        "primary" => Input::None,
        color => Input::from(color),
    }
}

#[component]
pub fn SharePage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Share",
            source: "libero/src/components/buttons/share.rs",
            markdown: "/md/share.md",
            properties: vec![props("Share", vec![
                prop("title", "String")
                    .doc("A title, used as the subject where the target has one."),
                prop("text", "String").doc("A message."),
                prop("url", "String")
                    .doc("The link. Where no share sheet exists, a press copies it, else `text`, else `title`."),
                prop("files", "Vec<ShareFile>")
                    .doc("Files (name, MIME type, bytes), shared where the target takes them. Elsewhere the title, text and link still go."),
                prop("variant", "Variant")
                    .doc("Visual style, as on `ActionIcon` or `Button`. Icon-only with `color` also unset, the button draws no chrome of its own."),
                prop("color", "ThemeAwareValue")
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "ThemeAwareValue")
                    .default(theme.action_icon.size.as_str())
                    .doc("Button size. With `children`, only a size word applies."),
                prop("radius", "ThemeAwareValue")
                    .default(theme.action_icon.radius.as_str())
                    .doc("Corner radius, independent of `size`."),
                prop("aria_label", "String")
                    .default("\"Share\"")
                    .doc("The button's name, such as \"Share this article\". Unset, the name is the localization's `share`, or the `children` text."),
                prop("label", "String")
                    .doc("Describes what a press shares, read after the name, such as \"Link to this page\"."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
                prop("onshare", "EventHandler<ShareOutcome>")
                    .doc("Called with the outcome once the share settled: `Shared`, `Copied`, `Cancelled`, `Unsupported` or `Failed(reason)`."),
                prop("children", "Element")
                    .doc("A visible label: renders a `Button` with the share glyph instead of an icon-only button."),
            ])],
            accessibility: a11y()
                .handles([
                    "The button keeps its focus through a share. The sheet is the system's own dialog and announces itself, so the status stays silent.",
                    "Where no share sheet exists, an always-mounted `role=\"status\"` region says \"Copied\" once the link is on the clipboard, or that sharing failed, and the icon turns into a check or an alert. A cancelled sheet says nothing.",
                    "The icon and the status reset when the pointer or the focus leaves.",
                    "Unset, the icon-only button's `aria_label` is the localization's `share`. The words are `ShareLabels`.",
                    "`label` becomes the button's description through a hidden element, so the name stays short: \"Share, Link to this page\".",
                ])
                .must([
                    "Name an icon-only button after what it shares with `aria_label`, such as \"Share article\", or keep the short name and say what it shares with `label`.",
                    "For a share control of your own on `use_share()`, call `share` in the press handler and say the outcome in a status region that is already mounted.",
                ])
                .example("An article's share button, `Share { url, label: \"Link to this article\" }`: the button reads \"Share, Link to this article\". On a phone the share sheet opens; on a desktop without one, the status region says \"Copied\"."),
            lead: rsx! {
                Text {
                    "A button that opens the system share sheet: the Web Share API on the "
                    "web and in desktop and iOS WebViews. Where there is none (Blitz, most "
                    "Linux browsers), a press copies the link, the icon turns into a check "
                    "and a screen reader hears \"Copied\". Icon-only by default; "
                    Code { source: "children" }
                    " make it a labelled "
                    Code { source: "Button" }
                    "."
                }
                Text {
                    "For a share control of your own, build on "
                    Code { source: "use_share()" }
                    ": "
                    Code { source: "share(ShareData)" }
                    " opens the sheet (call it in the press handler: the browser allows it "
                    "only inside a user action), "
                    Code { source: "outcome()" }
                    ", "
                    Code { source: "shared()" }
                    ", "
                    Code { source: "copied()" }
                    " and "
                    Code { source: "failed()" }
                    " follow once it settled, "
                    Code { source: "pending()" }
                    " says one is still open (a press meanwhile is ignored), and "
                    Code { source: "reset()" }
                    " clears them. "
                    Code { source: "libero::platform::share" }
                    " is the same without the copy fallback."
                }
            },
            Demo {
                component: "Share",
                children_text: "",
                fixed: vec![
                    format!("url: {URL:?}"),
                    "title: \"Libero\"".to_string(),
                ],
                controls: controls(),
                render: move |values: DemoValues| rsx! {
                    Share {
                        url: URL,
                        title: "Libero",
                        variant: values.str("variant"),
                        color: color(&values),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| LABEL.to_string()),
                        disabled: values.str("disabled") == "true",
                        children: (values.str("children") == "true").then(|| rsx! { "Share" }),
                    }
                },
                child: Child(text_child),
            }
        }
    }
}
