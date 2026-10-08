use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Input, RepoHost, Repository, RepositoryPart, Text};

#[component]
pub fn RepositoryPage() -> Element {
    rsx! {
        DocPage {
            title: "Repository",
            source: "libero/src/components/buttons/repository.rs",
            markdown: "/md/repository.md",
            properties: vec![props("Repository", vec![
                prop("repo", "String").default("required")
                    .doc("`owner/repo`, as in the repository's URL. GitLab takes nested groups too."),
                prop("host", "RepoHost")
                    .default("GitHub")
                    .doc("Where the repository lives: `RepoHost::GitHub` or `RepoHost::GitLab`."),
                prop("variant", "Variant")
                    .default("outlined")
                    .doc("Visual style, as on `ActionIcon`."),
                prop("color", "ThemeAwareValue")
                    .default("muted")
                    .doc("Accent color. A theme color name or any CSS color. Unset, a `gradient` takes the theme's gradient, as no label reads on a muted one."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size. The icon takes half of it."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of `size`."),
                prop("aria_label", "String")
                    .doc("Replaces the name's subject, the host and repository; the star count and new-tab cue still follow. A raw `\"aria-label\"` attribute replaces the whole name."),
                prop("parts", "Parts<RepositoryPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("RepositoryPart", vec![
                (RepositoryPart::Icon, "The host's logo."),
                (RepositoryPart::Count, "The star count, once it has loaded."),
            ])],
            accessibility: a11y()
                .handles([
                    "The link's name is the host and repository (\"GitHub tdymel/libero\"), the star count once it arrives, and the new-tab cue, so two buttons on one page read apart. The words come from `RepositoryLabels::stars`, `RepositoryLabels::compact` and `AnchorLabels::new_tab` in the localization.",
                    "The drawn count is not read twice: the name replaces the link's content.",
                    "A count drawn in the accent color, which could miss 4.5:1, takes the `ink` color instead.",
                    "Set, `aria_label` replaces the host and repository; the star count and new-tab cue still follow it.",
                ])
                .must([
                    "With your own `aria_label`, name the repository. With a raw `\"aria-label\"` attribute, which replaces the whole name, also say the star count and that it opens in a new tab.",
                ])
                .example("A header link, `Repository { repo: \"tdymel/libero\" }`: a screen reader reads \"GitHub tdymel/libero\", then the star count once it arrives, then that it opens in a new tab."),
            lead: rsx! {
                Text {
                    "A link to a repository with its star count beside the host's icon. "
                    "It opens in a new tab."
                }
            },
            Demo {
                component: "Repository",
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
                    Control::toggle("variant", ["outlined", "filled", "gradient", "tonal", "standard"])
                        .labels(["Outlined", "Filled", "Gradient", "Tonal", "Standard"])
                        .default("outlined"),
                    // `muted` is what an unset `color` resolves to, so that
                    // swatch prints nothing.
                    Control::color("color").default("muted"),
                    Control::sizes("size").default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                ],
                render: move |values: DemoValues| {
                    let (repo, host) = match values.str("host").as_str() {
                        "gitlab" => ("gitlab-org/gitlab", RepoHost::GitLab),
                        _ => ("tdymel/libero", RepoHost::GitHub),
                    };
                    rsx! {
                        Repository {
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

            DocSection {
                title: "Star count",
                Text {
                    "Give it the repository's name; it asks the host's public API at mount "
                    "and keeps the count for the session. A count older than 10 minutes shows "
                    "while it is fetched again. Until the count arrives, "
                    "when it is 0, or when the host does not answer, the icon stands alone. "
                    "This site's header uses "
                    Code { source: "Repository {{ repo: \"tdymel/libero\" }}" }
                    "."
                }
            }

            DocSection {
                title: "Native builds",
                Text {
                    "Native builds fetch through dioxus-native's network provider, which needs its "
                    Code { source: "net" }
                    " feature. "
                    Code { source: "net" }
                    " is on by default, so only an app that turns dioxus-native's default "
                    "features off has to add it back; without it the icon stands alone."
                }
            }
        }
    }
}
