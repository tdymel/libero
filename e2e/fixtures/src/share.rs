//! `Share` icon-only, labelled, with one file, with two files, and disabled; `#outcome` echoes the last `onshare`.

use dioxus::prelude::*;
use libero::components::{Flex, Share, Text};
use libero::platform::{ShareFile, ShareOutcome};

use crate::Routes;

pub const ROUTES: Routes = &[("/share", || rsx! { SharePage {} })];

#[component]
fn SharePage() -> Element {
    let mut outcome = use_signal(String::new);
    let onshare = move |shared: ShareOutcome| outcome.set(format!("{shared:?}"));
    // Android's share sheet covers the activity (3351).
    crate::save_file::use_covered_mark();
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            span { id: "bare",
                Share {
                    url: "https://libero-ui.dev",
                    title: "Libero",
                    text: "Components for Dioxus",
                    onshare,
                }
            }
            // The visible text the contrast pass needs; an icon button has none.
            Text { "libero-ui.dev" }
            span { id: "labelled",
                Share { url: "https://libero-ui.dev", label: "Link to the docs", onshare, "Share" }
            }
            span { id: "file",
                Share {
                    title: "Report",
                    files: vec![ShareFile {
                        name: "report.csv".into(),
                        mime: "text/csv".into(),
                        bytes: b"a,b\n1,2\n".to_vec(),
                    }],
                    aria_label: "Share the report",
                    onshare,
                }
            }
            span { id: "files",
                Share {
                    title: "Reports",
                    files: vec![
                        ShareFile { name: "a.csv".into(), mime: "text/csv".into(), bytes: b"a,b\n1,2\n".to_vec() },
                        ShareFile { name: "b.txt".into(), mime: "text/plain".into(), bytes: b"hello".to_vec() },
                    ],
                    aria_label: "Share both reports",
                    onshare,
                }
            }
            span { id: "off", Share { url: "x", variant: "filled", disabled: true } }
            span { id: "outcome", "{outcome}" }
        }
    }
}
