use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Input, RepoButton, RepoHost, Text};

#[component]
pub fn RepoButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "RepoButton",
            source: "libero/src/components/buttons/repo_button.rs",
            markdown: "/md/repo_button.md",
            properties: vec![props("RepoButton", vec![
                prop("repo", "String")
                    .doc("Required. `owner/repo`, as in the repository's URL. GitLab takes nested groups too."),
                prop("host", "RepoHost")
                    .default("GitHub")
                    .doc("Where the repository lives: `RepoHost::GitHub` or `RepoHost::GitLab`."),
                prop("variant", "Variant")
                    .default("outlined")
                    .doc("Visual style, as on `ActionIcon`."),
                prop("color", "ThemeAwareValue")
                    .default("muted")
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size. The icon takes half of it."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of `size`."),
                prop("aria_label", "String")
                    .doc("Replaces the whole built name: host, repository, star count and new-tab cue. Unset, the built name."),
            ])],
            accessibility: a11y()
                .handles([
                    "The link's name is the host and repository (\"GitHub tdymel/libero\"), the star count once it arrives, and the new-tab cue, so two buttons on one page read apart. The words come from `RepoButtonLabels::stars` and `AnchorLabels::new_tab` in the localization.",
                    "The drawn count is not read twice: the name replaces the link's content.",
                    "A count drawn in the accent color, which could miss 4.5:1, takes the `ink` color instead.",
                    "Set, `aria_label` replaces that whole name, count and new-tab cue included.",
                ])
                .must([
                    "With your own `aria_label`, name the repository and say that it opens in a new tab.",
                ]),
            lead: rsx! {
                Text {
                    "A link to a repository with its star count beside the host's icon. "
                    "Give it the repository's name; it asks the host's public API once per "
                    "mount and keeps the count for the session. Until the count arrives, "
                    "when it is 0, or when the host does not answer, the icon stands alone."
                }
                Text {
                    "It opens in a new tab. This site's header uses "
                    Code { source: "RepoButton {{ repo: \"tdymel/libero\" }}" }
                    "."
                }
            },
            Demo {
                component: "RepoButton",
                children_text: "",
                controls: vec![
                    Control::toggle("host", ["github", "gitlab"])
                        .labels(["GitHub", "GitLab"])
                        .code(|_, values| match values.str("host").as_str() {
                            "gitlab" => vec![
                                "repo: \"gitlab-org/gitlab\"".to_string(),
                                "host: RepoHost::GitLab".to_string(),
                            ],
                            _ => vec!["repo: \"tdymel/libero\"".to_string()],
                        }),
                    Control::toggle("variant", ["outlined", "filled", "tonal", "standard"])
                        .labels(["Outlined", "Filled", "Tonal", "Standard"])
                        .default("outlined"),
                    // `muted` is what an unset `color` resolves to, so that
                    // swatch prints nothing.
                    Control::color("color").default("muted"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                ],
                render: move |values: DemoValues| {
                    let (repo, host) = match values.str("host").as_str() {
                        "gitlab" => ("gitlab-org/gitlab", RepoHost::GitLab),
                        _ => ("tdymel/libero", RepoHost::GitHub),
                    };
                    rsx! {
                        RepoButton {
                            repo,
                            host,
                            variant: values.str("variant"),
                            color: match values.str("color").as_str() {
                                "muted" => Input::None,
                                color => Input::from(color),
                            },
                            size: values.str("size"),
                            radius: values.str("radius"),
                        }
                    }
                },
            }
        }
    }
}
