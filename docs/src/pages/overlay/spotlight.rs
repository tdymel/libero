use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{
    Button, Code, CodeBlock, Flex, Kbd, SpotlightAction, SpotlightOptions, Text, spotlight_filter,
    use_spotlight,
};

const USAGE: &str = r#"fn actions(mut last: Signal<String>) -> Vec<SpotlightAction> {
    let run = move |name: &'static str| move |_| last.set(name.to_string());
    vec![
        SpotlightAction::new("Home").group("Pages").description("The start page").onclick(run("Home")),
        SpotlightAction::new("Components").group("Pages").onclick(run("Components")),
        SpotlightAction::new("Changelog").group("Pages").keywords(["releases", "news"]).onclick(run("Changelog")),
        SpotlightAction::new("New file").group("Commands").shortcut("Ctrl N").onclick(run("New file")),
        SpotlightAction::new("Toggle sidebar").group("Commands").shortcut("Ctrl B").onclick(run("Toggle sidebar")),
        SpotlightAction::new("Sign out").group("Account").onclick(run("Sign out")),
    ]
}

let mut last = use_signal(|| String::from("nothing yet"));
let all = use_hook(|| actions(last));
let spotlight = use_spotlight(SpotlightOptions {
    actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &all))),
    // The docs shell already owns Ctrl+K; a page-level palette binds none.
    shortcut: None,
    ..Default::default()
});

rsx! {
    Button { variant: "outlined", onclick: move |_| spotlight.open(), "Open the palette" }
    Text { size: "sm", "Last run: {last()}" }
}"#;

fn actions(mut last: Signal<String>) -> Vec<SpotlightAction> {
    let run = move |name: &'static str| move |_| last.set(name.to_string());
    vec![
        SpotlightAction::new("Home")
            .group("Pages")
            .description("The start page")
            .onclick(run("Home")),
        SpotlightAction::new("Components")
            .group("Pages")
            .onclick(run("Components")),
        SpotlightAction::new("Changelog")
            .group("Pages")
            .keywords(["releases", "news"])
            .onclick(run("Changelog")),
        SpotlightAction::new("New file")
            .group("Commands")
            .shortcut("Ctrl N")
            .onclick(run("New file")),
        SpotlightAction::new("Toggle sidebar")
            .group("Commands")
            .shortcut("Ctrl B")
            .onclick(run("Toggle sidebar")),
        SpotlightAction::new("Sign out")
            .group("Account")
            .onclick(run("Sign out")),
    ]
}

#[component]
fn SpotlightDemo() -> Element {
    let last = use_signal(|| String::from("nothing yet"));
    let all = use_hook(|| actions(last));
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        shortcut: None,
        ..Default::default()
    });

    rsx! {
        Flex {
            direction: "row",
            align: "center",
            gap: "md",
            Button { variant: "outlined", onclick: move |_| spotlight.open(), "Open the palette" }
            Text { size: "sm", "Last run: {last()}" }
        }
    }
}

#[component]
pub fn SpotlightPage() -> Element {
    rsx! {
        DocPage {
            title: "Spotlight",
            source: "libero/src/components/overlay/spotlight",
            markdown: "/md/spotlight.md",
            lead: rsx! {
                Text {
                    "A command palette: a modal search box over a list of actions. "
                    Code { source: "use_spotlight" }
                    " is a hook, like "
                    Code { source: "use_modal" }
                    " - it returns a "
                    Code { source: "Copy" }
                    " handle with "
                    Code { source: "open" }
                    ", "
                    Code { source: "close" }
                    " and "
                    Code { source: "toggle" }
                    ". What it lists is yours: "
                    Code { source: "actions" }
                    " is called with the live query and returns the rows, so a static list, "
                    "a filtered one and search results are the same prop. "
                    Code { source: "spotlight_filter" }
                    " is the common case: label hits first, then description and keyword hits."
                }
                Text {
                    "Rows with a "
                    Code { source: "group" }
                    " are drawn under its header, groups in the order they first appear, and "
                    Code { source: "limit" }
                    " counts rows through the groups. A "
                    Code { source: "shortcut" }
                    " on an action is a hint, drawn as a "
                    Code { source: "Kbd" }
                    ", and never bound."
                }
            },
            DocSection {
                title: "Usage",
                SpotlightDemo {}
                CodeBlock { source: USAGE, language: "rust" }
            }
            DocSection {
                title: "Opening it",
                Text {
                    Kbd { "Ctrl" } " + " Kbd { "K" } " (" Kbd { "Cmd" } " on a Mac) toggles a palette "
                    "from anywhere on the page - try it here, it opens this site's own search. "
                    "Change the key with "
                    Code { source: "shortcut: Some('p')" }
                    ", or turn it off with "
                    Code { source: "None" }
                    ". The chord is ignored while you type in another text field, and while "
                    "a dialog or a popover is already open. It needs a document-level key "
                    "listener, which only the web has today: elsewhere, open the palette "
                    "from a button."
                }
                Text {
                    "Results that arrive from a search index are the same prop: return a "
                    "signal's contents from "
                    Code { source: "actions" }
                    " instead of calling "
                    Code { source: "spotlight_filter" }
                    ", and the palette redraws when the signal fills."
                }
            }
            DocSection {
                title: "Keyboard and accessibility",
                Text {
                    "Focus stays in the search box the whole time. The search box is a "
                    Code { source: "role=\"combobox\"" }
                    " over a "
                    Code { source: "role=\"listbox\"" }
                    ", and " Kbd { "↓" } " " Kbd { "↑" }
                    " move a highlight it names with "
                    Code { source: "aria-activedescendant" }
                    ", wrapping at both ends. "
                    Kbd { "Enter" } " runs the highlighted action; typing clears the highlight, "
                    "so Enter never runs a row you did not look at. " Kbd { "Esc" }
                    " or a click outside closes, and focus goes back to what opened it. "
                    "Groups are "
                    Code { source: "role=\"group\"" }
                    " named by their header, nothing is ever "
                    Code { source: "aria-selected" }
                    " - a palette runs things, it does not select them - and \"Nothing found\" "
                    "is announced through a status region."
                }
            }
        }
    }
}
