//! `RepoButton`'s first render: a link to the host's page, the icon alone,
//! and a name from the localization. The fetched count is e2e's.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{RepoButton, RepoHost},
    localization::Localization,
};

#[test]
fn before_a_count_it_is_the_host_icon_linking_to_the_repo() {
    fn app() -> Element {
        rsx! { LiberoProvider { RepoButton { repo: "tdymel/libero" } } }
    }
    let link = attributes_of(&body(&render(app)), "a");
    assert_eq!(link["href"], "https://github.com/tdymel/libero");
    assert_eq!(link["target"], "_blank");
    assert_eq!(link["aria-label"], "GitHub (opens in a new tab)");
}

#[test]
fn gitlab_links_its_own_page_in_the_localized_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &Localization::GERMAN,
                RepoButton { repo: "gitlab-org/gitlab", host: RepoHost::GitLab }
            }
        }
    }
    let link = attributes_of(&body(&render(app)), "a");
    assert_eq!(link["href"], "https://gitlab.com/gitlab-org/gitlab");
    assert_eq!(link["aria-label"], "GitLab (öffnet in einem neuen Tab)");
}
