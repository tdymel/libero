use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, Table, Text, column},
    sx::sx,
};

/// Name, what it is used for, licence, and its source.
type Row = (&'static str, &'static str, &'static str, &'static str);

const CREDITS: [Row; 4] = [
    (
        "Lucide",
        "Icons on these pages and the default icons of libero's components.",
        "ISC",
        "https://lucide.dev",
    ),
    (
        "pictogram",
        "Carries the Lucide icons and the Lobe and Simple Icons marks as Rust data.",
        "MIT OR Apache-2.0",
        "https://github.com/tdymel/pictogram",
    ),
    (
        "Lobe Icons",
        "The ChatGPT, Google, Claude and Perplexity marks in Tldr.",
        "MIT",
        "https://github.com/lobehub/lobe-icons",
    ),
    (
        "Simple Icons",
        "The GitHub and GitLab marks in RepoButton, and the GitHub and Markdown marks on these pages.",
        "CC0 1.0",
        "https://simpleicons.org",
    ),
];

#[component]
pub fn CreditsPage() -> Element {
    rsx! {
        DocPage {
            title: "Credits",
            markdown: "/md/credits.md",
            lead: rsx! {
                Text {
                    "The icons and logos on these pages and inside libero's components come from "
                    "other projects. Their licences ask that the notices below stay with any copy."
                }
            },

            DocSection {
                title: "Icons and logos",
                Table {
                    aria_label: "Credits",
                    data: CREDITS.to_vec(),
                    columns: vec![
                        column("Name").value(|row: &Row| row.0),
                        column("Used for").value(|row: &Row| row.1),
                        column("Licence").value(|row: &Row| row.2),
                        column("Source")
                            .value(|row: &Row| row.3)
                            .render(|row: &Row| rsx! { Anchor { to: row.3, "{row.3}" } }),
                    ],
                }
                Text {
                    "The Claude, Google, Perplexity, ChatGPT, GitLab and GitHub marks are "
                    "trademarks of their owners, shown only to name the service a link opens."
                }
            }

            DocSection {
                title: "Notices",
                Text { sx: sx().font_weight("600"), "Lucide (ISC)" }
                Text {
                    "Copyright (c) 2026 Lucide Icons and Contributors. Icons derived from "
                    "Feather are also under the MIT License, Copyright (c) 2013-present Cole Bemis."
                }
                Text {
                    "Permission to use, copy, modify, and/or distribute this software for any "
                    "purpose with or without fee is hereby granted, provided that the above "
                    "copyright notice and this permission notice appear in all copies."
                }
                Text { sx: sx().font_weight("600"), "Lobe Icons (MIT)" }
                Text { "Copyright (c) 2023 LobeHub." }
                Text { sx: sx().font_weight("600"), "pictogram (MIT OR Apache-2.0)" }
                Text {
                    "Copyright (c) 2025 Tom Dymel. "
                    "Used as the "
                    Code { source: "pictogram-core" }
                    ", "
                    Code { source: "pictogram-icons-lucide" }
                    ", "
                    Code { source: "pictogram-icons-lobe" }
                    " and "
                    Code { source: "pictogram-icons-simple" }
                    " crates. Licence texts ship with the crates."
                }
            }
        }
    }
}
