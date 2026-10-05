use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop, props,
};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::{
    components::{
        Button, Code, Flex, Kbd, SpotlightAction, SpotlightOptions, SpotlightPart, Text,
        spotlight_filter, use_spotlight,
    },
    platform::{TimerSubscription, timer},
};
use std::time::Duration;

/// The page's own source: each example's actions fn prints above its hook.
const FILE: DemoFile = DemoFile(include_str!("spotlight.rs"));

// demo-code: commands start
fn commands(mut last: Signal<String>) -> Vec<SpotlightAction> {
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
// demo-code: commands end

// demo-code: files start
fn files(mut last: Signal<String>) -> Vec<SpotlightAction> {
    [
        "src",
        "src/main.rs",
        "src/lib.rs",
        "tests",
        "Cargo.toml",
        "README.md",
    ]
    .into_iter()
    .map(|path| {
        let icon = if path.contains('.') {
            rsx! { Pictogram { icon: lucide::file::outlined } }
        } else {
            rsx! { Pictogram { icon: lucide::folder::outlined } }
        };
        SpotlightAction::new(path)
            .icon(icon)
            .onclick(move |_| last.set(path.to_string()))
    })
    .collect()
}
// demo-code: files end

// demo-code: issues start
fn issues(mut last: Signal<String>) -> Vec<SpotlightAction> {
    (1..=200)
        .map(|n| {
            SpotlightAction::new(format!("Issue #{n}"))
                .description(if n % 2 == 0 { "Open" } else { "Closed" })
                .onclick(move |_| last.set(format!("Issue #{n}")))
        })
        .collect()
}
// demo-code: issues end

/// How long the fake search takes - long enough to see, short enough to type
/// through.
const LATENCY: Duration = Duration::from_millis(700);

/// The four examples: the name of the handle and the actions fn in the
/// printed code, the section of the actions fn's source, and the dialog's `aria_label`.
const EXAMPLES: [(&str, &str, &str, &str); 4] = [
    ("commands", "commands", "commands", "Commands"),
    ("files", "files", "files", "Files"),
    ("issues", "issues", "issues", "Issues"),
    ("search", "issues", "issues", "Issue search"),
];

fn dialog_name(example: &str) -> &'static str {
    EXAMPLES
        .into_iter()
        .find(|(name, ..)| *name == example)
        .map_or(EXAMPLES[0].3, |(.., label)| label)
}

/// What the controls add to every palette's options.
fn option_lines(values: &DemoValues) -> Vec<String> {
    let mut lines = vec![];
    match values.str("limit").as_str() {
        "none" => {}
        limit => lines.push(format!("limit: Some({limit}),")),
    }
    if values.str("close_on_action") == "false" {
        lines.push("close_on_action: false,".into());
    }
    if values.str("clear_on_close") == "false" {
        lines.push("clear_on_close: false,".into());
    }
    if values.str("highlight_first_on_query") == "false" {
        lines.push("highlight_first_on_query: false,".into());
    }
    lines.push(match values.str("shortcut").as_str() {
        "none" => "shortcut: None,".into(),
        key => format!("shortcut: Some('{key}'),"),
    });
    lines
}

/// The code of the example opened last. The preview's buttons pick it.
// snippet: mirrors SpotlightDemo except Callback already async await frame next search_on_server label none actions_code hint setup dioxus core Task search_task cancel take write
fn wrap_example(values: &DemoValues, _: &str) -> String {
    let example = values.str("example");
    let (handle, actions, actions_code, label) = EXAMPLES
        .into_iter()
        .find(|(name, ..)| *name == example)
        .unwrap_or(EXAMPLES[0]);

    let mut setup = format!("let all = use_hook(|| {actions}(last));\n");
    let mut options = vec![];
    if handle == "search" {
        setup.push_str(&(FILE.section("search") + "\n"));
        setup.push_str("let mut search_task = use_signal(|| None::<dioxus::core::Task>);\n");
        let search_actions = FILE.section("search_actions");
        options.push(format!(
            "actions: Some(Callback::new({})),",
            search_actions.trim_end_matches(',')
        ));
        options.push("loading: loading(),".into());
        options.push(
            "// The input event, not a render: the next frame is already loading.\n\
             // Cancelling the older search keeps a slow answer from landing late.\n\
             onquery: Some(Callback::new(move |query: String| {\n    \
             loading.set(true);\n    \
             if let Some(older) = search_task.write().take() {\n        \
             older.cancel();\n    \
             }\n    \
             let all = all.clone();\n    \
             search_task.set(Some(spawn(async move {\n        \
             results.set(search_on_server(&query, &all).await);\n        \
             loading.set(false);\n    \
             })));\n})),"
                .into(),
        );
    } else {
        options.push(
            "actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &all))),"
                .into(),
        );
    }
    options.push(format!("aria_label: Some({label:?}.into()),"));
    if handle == "files" {
        options.push(FILE.section("files_options"));
    }
    options.extend(option_lines(values));
    options.push("..Default::default()".into());

    let key = values.str("shortcut");
    let hint = if key == "none" {
        String::new()
    } else {
        format!(
            "    Text {{ size: \"sm\", Kbd {{ \"Ctrl\" }} \" + \" Kbd {{ {:?} }} \" toggles it\" }}\n",
            key.to_uppercase()
        )
    };
    let actions_code = FILE.section(actions_code);
    format!(
        "{actions_code}\n\n{}\n\
         {setup}\
         let {handle} = use_spotlight(SpotlightOptions {{\n{}}});\n\n\
         rsx! {{\n    \
             Button {{ variant: \"outlined\", onclick: move |_| {handle}.open(), \"Show\" }}\n\
         {hint}    \
             Text {{ size: \"sm\", role: \"status\", \"Last run: {{last()}}\" }}\n\
         }}",
        FILE.section("last"),
        indent(&options.join("\n")),
    )
}

#[derive(Props, Clone, PartialEq)]
struct SpotlightDemoProps {
    limit: Option<usize>,
    close_on_action: bool,
    clear_on_close: bool,
    highlight_first_on_query: bool,
    shortcut: Option<char>,
    values: DemoValues,
}

/// The hooks need a scope of their own: `Demo` calls `render` from its own.
#[component]
fn SpotlightDemo(props: SpotlightDemoProps) -> Element {
    let SpotlightDemoProps {
        limit,
        close_on_action,
        clear_on_close,
        highlight_first_on_query,
        shortcut,
        values,
    } = props;
    let example = values.str("example");
    // demo-code: last start
    let last = use_signal(|| String::from("nothing yet"));
    // demo-code: last end
    // Only the palette the code block shows is bound to the key.
    let options = |name: &str| SpotlightOptions {
        limit,
        close_on_action,
        clear_on_close,
        highlight_first_on_query,
        shortcut: shortcut.filter(|_| example == name),
        aria_label: Some(dialog_name(name).into()),
        ..Default::default()
    };

    let all_commands = use_hook(|| commands(last));
    let commands_actions =
        use_callback(move |query: String| spotlight_filter(&query, &all_commands));
    let commands = use_spotlight(SpotlightOptions {
        actions: Some(commands_actions),
        ..options("commands")
    });
    let all_files = use_hook(|| files(last));
    let files_actions = use_callback(move |query: String| spotlight_filter(&query, &all_files));
    let files = use_spotlight(SpotlightOptions {
        actions: Some(files_actions),
        // demo-code: files_options start
        placeholder: Some("Go to file...".into()),
        nothing_found: Some(rsx! { "No file by that name." }),
        // demo-code: files_options end
        ..options("files")
    });
    let all_issues = use_hook(|| issues(last));
    let search_all = all_issues.clone();
    let issues_actions = use_callback(move |query: String| spotlight_filter(&query, &all_issues));
    let issues = use_spotlight(SpotlightOptions {
        actions: Some(issues_actions),
        ..options("issues")
    });

    // The printed `spawn` is a timer here, so a newer keystroke cancels the
    // older search by dropping it, and a slow answer never lands late.
    // demo-code: search start
    let mut results = use_signal(Vec::<SpotlightAction>::new);
    let mut loading = use_signal(|| false);
    // demo-code: search end
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));
    let search_actions = use_callback(
        // demo-code: search_actions start
        move |query: String| match query.trim().is_empty() {
            true => vec![],
            false => results(),
        },
        // demo-code: search_actions end
    );
    let onquery = use_callback(move |query: String| {
        loading.set(true);
        let all = search_all.clone();
        let answer = timer().map(|timer| {
            timer.after(
                LATENCY,
                Box::new(move || {
                    results.set(spotlight_filter(&query, &all));
                    loading.set(false);
                }),
            )
        });
        pending.set(answer);
    });
    let search = use_spotlight(SpotlightOptions {
        actions: Some(search_actions),
        loading: loading(),
        onquery: Some(onquery),
        ..options("search")
    });

    // One trigger: the `example` control picks which palette it opens, and the
    // code block follows the same value.
    let handles = [commands, files, issues, search];
    let picked = EXAMPLES
        .iter()
        .position(|(name, ..)| *name == example)
        .unwrap_or(0);
    let handle = handles[picked];

    rsx! {
        Flex { direction: "column", align: "center", gap: "md",
            Button { variant: "outlined", onclick: move |_| handle.open(), "Show" }
            if let Some(key) = shortcut {
                Text { size: "sm",
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "{key.to_ascii_uppercase()}" }
                    " toggles it"
                }
            }
            Text { size: "sm", role: "status", "Last run: {last()}" }
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
            properties: vec![
                props("SpotlightOptions", vec![
                    prop("actions", "Callback<String, Vec<SpotlightAction>>").doc("Called with the query, returns the rows. Capture a `Signal`, not a `Vec`, if the list changes. Unset warns and shows nothing."),
                    prop("placeholder", "String").default("\"Search...\"").doc("The search box's placeholder."),
                    prop("nothing_found", "Element").doc("Shown and announced when a query matches nothing. Unset, the localization's text."),
                    prop("limit", "usize").doc("The most rows drawn, counted across groups."),
                    prop("close_on_action", "bool").default("true").doc("Closes after running an action."),
                    prop("clear_on_close", "bool").default("true").doc("Starts every opening with an empty query."),
                    prop("aria_label", "String").default("\"Command palette\"").doc("Names the dialog and its list."),
                    prop("shortcut", "Option<char>").default("Some('k')").doc("Ctrl (Cmd on a Mac) plus this key toggles the palette. `None` for no hotkey. Web only. A key the browser already uses, such as L, T or W, warns in a debug build. Elsewhere, open the palette from a button. Two palettes on one page should not share a key."),
                    prop("highlight_first_on_query", "bool").default("true").doc("Highlights the first row after every keystroke, so Enter runs it. Off, Enter does nothing until the arrows pick a row."),
                    prop("loading", "bool").default("false").doc("The results are still coming. A loader replaces the rows, and a screen reader hears \"Searching\"."),
                    prop("onquery", "Callback<String>").doc("Called with the query on every keystroke. Set `loading` and start the search here, cancelling the previous one: a newer query supersedes it."),
                    prop("sx", "Sx").doc("Styles the dialog box."),
                    prop("parts", "Parts<SpotlightPart>").doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .without_base_props()
                .parts("SpotlightPart", vec![
                    (SpotlightPart::Body, "Holds the search box, the list and the status line."),
                    (SpotlightPart::Search, "The search box."),
                    (SpotlightPart::List, "The `listbox`, a `ScrollArea`."),
                    (SpotlightPart::Group, "A named group of rows."),
                    (SpotlightPart::GroupLabel, "A group's visible name."),
                    (SpotlightPart::Option, "One action row. The highlighted one has `data-active`."),
                    (SpotlightPart::Icon, "A row's icon."),
                    (SpotlightPart::Text, "The column holding the label and the description."),
                    (SpotlightPart::Label, "A row's label."),
                    (SpotlightPart::Description, "A row's second line."),
                    (SpotlightPart::Shortcut, "A row's key hint, one `Kbd` per key."),
                    (SpotlightPart::Status, "The status line: \"nothing found\" or the loader, and the result count for a reader only."),
                ]),
                props("SpotlightAction", vec![
                    prop("label", "String").default("required").doc("The row's text, and the first thing `spotlight_filter` matches."),
                    prop("description", "String").doc("A second line, matched after the label."),
                    prop("keywords", "Vec<String>").doc("Matched, never drawn."),
                    prop("group", "String").doc("A section header. Groups keep the order they first appear in."),
                    prop("icon", "Element").doc("Drawn before the label."),
                    prop("shortcut", "String").doc("A hint, one `Kbd` per key, split on spaces and `+`. Never bound."),
                    prop("onclick", "Callback<()>").doc("Runs on Enter or a click."),
                ]).without_base_props(),
                props("SpotlightHandle", vec![
                    prop("open()", "()").doc("Opens with focus in the search box. Call it from the trigger's handler, so focus returns there."),
                    prop("close()", "()").doc("Closes."),
                    prop("toggle()", "()").doc("Opens or closes."),
                    prop("is_open()", "bool").doc("Whether it is open."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Down", "Up"], "Moves the highlight, wrapping at both ends.")
                .key(["Enter"], "Runs the highlighted action. After typing, the first row is highlighted; right after opening, none is.")
                .key(["Escape"], "Closes, as a click outside does. Focus goes back to what opened it.")
                .handles([
                    "Focus stays in the search box.",
                    "A polite status region says how many actions a query left (`SpotlightLabels::results`, \"2 results\"), \"nothing found\" when none, and \"searching\" while `loading`.",
                    "The hotkey is ignored while you type in another text field, and while a dialog or popover is open.",
                ])
                .must([
                    "Turn `highlight_first_on_query` off for a palette whose actions change things. Then nothing is highlighted until you press Down.",
                    "Call `open()` from the trigger's handler, so focus returns there.",
                ])
                .example("A command palette opened from a \"Search\" button that calls `open()`: typing \"the\" says \"2 results\", Enter runs the first one, and Escape puts focus back on Search.")
                .limits([
                    "In a desktop WebView or on Android, Tab and Shift+Tab can leave the palette for the page behind it.",
                ]),
            lead: rsx! {
                Text {
                    "A command palette, a modal search box over a list of actions. "
                    Code { source: "use_spotlight" }
                    " returns a "
                    Code { source: "Copy" }
                    " handle, like "
                    Code { source: "use_modal" }
                    ". "
                    Code { source: "actions" }
                    " is called with the query and returns the rows, so a fixed list and "
                    "search results are the same prop. "
                    Code { source: "spotlight_filter" }
                    " covers the common case. Label hits come first, exact and prefix matches "
                    "before matches inside a word, so \"table\" puts Table above Sortable."
                }
            },
            // snippet: item async fn search_on_server(_: &str, _: &[SpotlightAction]) -> Vec<SpotlightAction> { Vec::new() }
            Demo {
                component: "SpotlightOptions",
                children_text: "",
                controls: vec![
                    // Not a prop: which palette the one button opens, and the
                    // example the code block prints.
                    Control::toggle("example", ["commands", "files", "issues", "search"])
                        .labels(["Commands", "Files", "200 issues", "Slow search"]),
                    Control::toggle("shortcut", ["j", "p", "none"]).labels(["J", "P", "None"]),
                    Control::slider("limit", ["3", "5", "10", "none"]).default("none"),
                    Control::switch("close_on_action").default("true"),
                    Control::switch("clear_on_close").default("true"),
                    Control::switch("highlight_first_on_query").default("true"),
                ],
                render: move |values: DemoValues| rsx! {
                    SpotlightDemo {
                        limit: values.str("limit").parse().ok(),
                        close_on_action: values.str("close_on_action") == "true",
                        clear_on_close: values.str("clear_on_close") == "true",
                        highlight_first_on_query: values.str("highlight_first_on_query") == "true",
                        shortcut: values.str("shortcut").chars().next().filter(|_| values.str("shortcut") != "none"),
                        values: values.clone(),
                    }
                },
                wrap: Wrap(wrap_example),
            }

            DocSection {
                title: "The demo's hotkey",
                Text {
                    "The hotkey here is J or P, because this site's search owns "
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "K" }
                    "."
                }
            }
        }
    }
}
