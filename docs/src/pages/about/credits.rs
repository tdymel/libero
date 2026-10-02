use crate::components::{DocPage, DocSection};
use crate::site::PICTOGRAM_REPO;
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, Table, Text, column},
    sx::sx,
};

/// Name, what it is used for, licence, and its source.
type Row = (&'static str, &'static str, &'static str, &'static str);

const CREDITS: [Row; 16] = [
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
        PICTOGRAM_REPO,
    ),
    (
        "Lobe Icons",
        "The ChatGPT, Google, Claude and Perplexity marks in Tldr.",
        "MIT",
        "https://github.com/lobehub/lobe-icons",
    ),
    (
        "Simple Icons",
        "The GitHub and GitLab marks in Repository, and the GitHub and Markdown marks on these pages.",
        "CC0 1.0",
        "https://simpleicons.org",
    ),
    (
        "Bootstrap Icons",
        "The Bootstrap set on the IconProvider page and in the Pictogram page's icon catalogue, behind libero's icons-bootstrap feature.",
        "MIT",
        "https://github.com/twbs/icons",
    ),
    (
        "Material Design Icons",
        "The Material set on the IconProvider page and in the Pictogram page's icon catalogue, behind libero's icons-material feature.",
        "Apache-2.0",
        "https://github.com/google/material-design-icons",
    ),
    (
        "Phosphor",
        "The Phosphor set on the IconProvider page and in the Pictogram page's icon catalogue, behind libero's icons-phosphor feature.",
        "MIT",
        "https://github.com/phosphor-icons/core",
    ),
    (
        "Tabler Icons",
        "The Tabler set on the IconProvider page and in the Pictogram page's icon catalogue, behind libero's icons-tabler feature.",
        "MIT",
        "https://github.com/tabler/tabler-icons",
    ),
    (
        "Feather",
        "The Feather set in the Pictogram page's icon catalogue.",
        "MIT",
        "https://github.com/feathericons/feather",
    ),
    (
        "Font Awesome Free",
        "The Font Awesome set in the Pictogram page's icon catalogue.",
        "CC BY 4.0",
        "https://fontawesome.com",
    ),
    (
        "Heroicons",
        "The Heroicons set in the Pictogram page's icon catalogue.",
        "MIT",
        "https://github.com/tailwindlabs/heroicons",
    ),
    (
        "Iconoir",
        "The Iconoir set in the Pictogram page's icon catalogue.",
        "MIT",
        "https://github.com/iconoir-icons/iconoir",
    ),
    (
        "Ionicons",
        "The Ionicons set in the Pictogram page's icon catalogue.",
        "MIT",
        "https://github.com/ionic-team/ionicons",
    ),
    (
        "Octicons",
        "The Octicons set in the Pictogram page's icon catalogue.",
        "MIT",
        "https://github.com/primer/octicons",
    ),
    (
        "VS Code Codicons",
        "The Codicons set in the Pictogram page's icon catalogue.",
        "CC BY 4.0",
        "https://github.com/microsoft/vscode-codicons",
    ),
    (
        "Big Buck Bunny",
        "The film in the Video demo, streamed from Wikimedia Commons. (c) 2008 Blender Foundation.",
        "CC BY 3.0",
        "https://peach.blender.org",
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
                    "The icons and logos on these pages and inside libero's components, and the "
                    "Video demo's film, come from other projects. Their licences ask that the "
                    "notices below stay with any copy."
                }
            },

            DocSection {
                title: "Icons, logos and media",
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
                Text { sx: sx().font_weight("600"), "MIT icon sets" }
                Text { "Feather: Copyright (c) 2013-2023 Cole Bemis." }
                Text { "Heroicons: Copyright (c) Tailwind Labs, Inc." }
                Text { "Iconoir: Copyright (c) 2021 Luca Burgio." }
                Text { "Ionicons: Copyright (c) 2015-present Ionic (http://ionic.io/)." }
                Text { "Octicons: Copyright (c) 2026 GitHub Inc." }
                Text { "Bootstrap Icons: Copyright (c) 2019-2024 The Bootstrap Authors." }
                Text { "Phosphor: Copyright (c) 2023 Phosphor Icons." }
                Text { "Tabler Icons: Copyright (c) 2020-2026 Paweł Kuna." }
                Text {
                    "Permission is hereby granted, free of charge, to any person obtaining a copy "
                    "of this software and associated documentation files (the \"Software\"), to "
                    "deal in the Software without restriction, including without limitation the "
                    "rights to use, copy, modify, merge, publish, distribute, sublicense, and/or "
                    "sell copies of the Software, and to permit persons to whom the Software is "
                    "furnished to do so, subject to the following conditions: The above copyright "
                    "notice and this permission notice shall be included in all copies or "
                    "substantial portions of the Software."
                }
                Text {
                    "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR "
                    "IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, "
                    "FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE "
                    "AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER "
                    "LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, "
                    "OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE "
                    "SOFTWARE."
                }
                Text { sx: sx().font_weight("600"), "Material Design Icons (Apache-2.0)" }
                Text {
                    "Copyright Google LLC. Licensed under the Apache License, Version 2.0 (the "
                    "\"License\"); you may not use this file except in compliance with the License. "
                    "You may obtain a copy of the License at "
                    Anchor { to: "https://www.apache.org/licenses/LICENSE-2.0", "https://www.apache.org/licenses/LICENSE-2.0" }
                    ". Unless required by applicable law or agreed to in writing, software "
                    "distributed under the License is distributed on an \"AS IS\" BASIS, WITHOUT "
                    "WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied."
                }
                Text { sx: sx().font_weight("600"), "Font Awesome Free (CC BY 4.0)" }
                Text {
                    "Font Awesome Free by @fontawesome - "
                    Anchor { to: "https://fontawesome.com", "https://fontawesome.com" }
                    ", (c) Fonticons, Inc., licensed under "
                    Anchor { to: "https://creativecommons.org/licenses/by/4.0/", "CC BY 4.0" }
                    ". Changed: converted from SVG to Rust data by pictogram."
                }
                Text { sx: sx().font_weight("600"), "VS Code Codicons (CC BY 4.0)" }
                Text {
                    "Codicons by Microsoft Corporation, "
                    Anchor { to: "https://github.com/microsoft/vscode-codicons", "https://github.com/microsoft/vscode-codicons" }
                    ", licensed under "
                    Anchor { to: "https://creativecommons.org/licenses/by/4.0/", "CC BY 4.0" }
                    ". Changed: converted from SVG to Rust data by pictogram."
                }
            }
        }
    }
}
