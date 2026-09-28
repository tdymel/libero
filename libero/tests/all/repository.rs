//! `Repository`'s first render: a link to the host's page, the icon alone,
//! and a name of host, repo and the localized new-tab cue. The fetched count is e2e's.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{RepoHost, Repository},
    localization::Localization,
};

#[test]
fn before_a_count_it_is_the_host_icon_linking_to_the_repo() {
    fn app() -> Element {
        rsx! { LiberoProvider { Repository { repo: "tdymel/libero" } } }
    }
    let link = attributes_of(&body(&render(app)), "a");
    assert_eq!(link["href"], "https://github.com/tdymel/libero");
    assert_eq!(link["target"], "_blank");
    assert_eq!(
        link["aria-label"],
        "GitHub tdymel/libero (opens in a new tab)"
    );
}

#[test]
fn aria_label_replaces_the_built_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Repository { repo: "tdymel/libero", aria_label: "Libero source (new tab)" }
            }
        }
    }
    let link = attributes_of(&body(&render(app)), "a");
    assert_eq!(link["aria-label"], "Libero source (new tab)");
}

#[test]
fn gitlab_links_its_own_page_in_the_localized_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &Localization::GERMAN,
                Repository { repo: "gitlab-org/gitlab", host: RepoHost::GitLab }
            }
        }
    }
    let link = attributes_of(&body(&render(app)), "a");
    assert_eq!(link["href"], "https://gitlab.com/gitlab-org/gitlab");
    assert_eq!(
        link["aria-label"],
        "GitLab gitlab-org/gitlab (öffnet in einem neuen Tab)"
    );
}
