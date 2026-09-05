use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use crate::icons::{FileIcon, FolderIcon};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Code, Flex, Kbd, SpotlightAction, SpotlightOptions, Text, spotlight_filter,
        use_spotlight,
    },
    platform::{TimerSubscription, timer},
};
use std::time::Duration;

/// Each example's actions, printed verbatim above the hook - keep every one in
/// step with the fn below it.
const COMMANDS_CODE: &str = r#"fn commands(mut last: Signal<String>) -> Vec<SpotlightAction> {
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
"#;

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

// snippet: item #[component] fn FileIcon() -> Element { rsx! {} }
// snippet: item #[component] fn FolderIcon() -> Element { rsx! {} }
const FILES_CODE: &str = r#"fn files(mut last: Signal<String>) -> Vec<SpotlightAction> {
    ["src", "src/main.rs", "src/lib.rs", "tests", "Cargo.toml", "README.md"]
        .into_iter()
        .map(|path| {
            let icon = if path.contains('.') { rsx! { FileIcon {} } } else { rsx! { FolderIcon {} } };
            SpotlightAction::new(path).icon(icon).onclick(move |_| last.set(path.to_string()))
        })
        .collect()
}
"#;

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
            rsx! { FileIcon {} }
        } else {
            rsx! { FolderIcon {} }
        };
        SpotlightAction::new(path)
            .icon(icon)
            .onclick(move |_| last.set(path.to_string()))
    })
    .collect()
}

const ISSUES_CODE: &str = r#"fn issues(mut last: Signal<String>) -> Vec<SpotlightAction> {
    (1..=200)
        .map(|n| {
            SpotlightAction::new(format!("Issue #{n}"))
                .description(if n % 2 == 0 { "Open" } else { "Closed" })
                .onclick(move |_| last.set(format!("Issue #{n}")))
        })
        .collect()
}
"#;

fn issues(mut last: Signal<String>) -> Vec<SpotlightAction> {
    (1..=200)
        .map(|n| {
            SpotlightAction::new(format!("Issue #{n}"))
                .description(if n % 2 == 0 { "Open" } else { "Closed" })
                .onclick(move |_| last.set(format!("Issue #{n}")))
        })
        .collect()
}

/// The search example's extra lines: a fake fetch per keystroke. `onquery`
/// runs from the input event, so `loading` is set before the next frame and
/// "nothing found" never flashes between the keystroke and the answer.
const SEARCH_CODE: &str = r#"let mut results = use_signal(Vec::<SpotlightAction>::new);
let mut loading = use_signal(|| false);
"#;

/// How long the fake search takes - long enough to see, short enough to type
/// through.
const LATENCY: Duration = Duration::from_millis(700);

/// The four examples: the name of the handle and the actions fn in the
/// printed code, the actions fn's source, and the trigger's label.
const EXAMPLES: [(&str, &str, &str, &str); 4] = [
    ("commands", "commands", COMMANDS_CODE, "Commands"),
    ("files", "files", FILES_CODE, "Files"),
    ("issues", "issues", ISSUES_CODE, "200 issues"),
    ("search", "issues", ISSUES_CODE, "Slow search"),
];

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
fn wrap_example(values: &DemoValues, _: &str) -> String {
    let example = values.str("example");
    let (handle, actions, actions_code, _) = EXAMPLES
        .into_iter()
        .find(|(name, ..)| *name == example)
        .unwrap_or(EXAMPLES[0]);

    let mut setup = format!("let all = use_hook(|| {actions}(last));\n");
    let mut options = vec![];
    if handle == "search" {
        setup.push_str(SEARCH_CODE);
        options.push(
            "actions: Some(Callback::new(move |query: String| match query.trim().is_empty() {\n    \
             true => vec![],\n    \
             false => results(),\n})),"
                .into(),
        );
        options.push("loading: loading(),".into());
        options.push(
            "// The input event, not a render: the next frame is already loading.\n\
             onquery: Some(Callback::new(move |query: String| {\n    \
             loading.set(true);\n    \
             let all = all.clone();\n    \
             spawn(async move {\n        \
             results.set(search_on_server(&query, &all).await);\n        \
             loading.set(false);\n    \
             });\n})),"
                .into(),
        );
    } else {
        options.push(
            "actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &all))),"
                .into(),
        );
    }
    if handle == "files" {
        options.push(r#"placeholder: Some("Go to file...".into()),"#.into());
        options.push(r#"nothing_found: Some(rsx! { "No file by that name." }),"#.into());
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
    format!(
        "{actions_code}\nlet last = use_signal(|| String::from(\"nothing yet\"));\n\
         {setup}\
         let {handle} = use_spotlight(SpotlightOptions {{\n{}}});\n\n\
         rsx! {{\n    \
             Button {{ variant: \"outlined\", onclick: move |_| {handle}.open(), \"Show\" }}\n\
         {hint}    \
             Text {{ size: \"sm\", \"Last run: {{last()}}\" }}\n\
         }}",
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
    let last = use_signal(|| String::from("nothing yet"));
    // Only the palette the code block shows is bound to the key.
    let options = |name: &str| SpotlightOptions {
        limit,
        close_on_action,
        clear_on_close,
        highlight_first_on_query,
        shortcut: shortcut.filter(|_| example == name),
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
        placeholder: Some("Go to file...".into()),
        nothing_found: Some(rsx! { "No file by that name." }),
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
    let mut results = use_signal(Vec::<SpotlightAction>::new);
    let mut loading = use_signal(|| false);
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));
    let search_actions = use_callback(move |query: String| match query.trim().is_empty() {
        true => vec![],
        false => results(),
    });
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
            properties: vec![
                props("SpotlightOptions", vec![
                    prop("actions", "Option<Callback<String, Vec<SpotlightAction>>>").doc("Called with the live query, returns the rows. Capture a `Signal`, not a `Vec`, if the list changes. `None` warns and shows nothing."),
                    prop("placeholder", "Option<String>").default("\"Search...\"").doc("The search box's placeholder, from the theme's labels."),
                    prop("nothing_found", "Option<Element>").doc("Shown, and announced, when a non-empty query matches nothing. Unset, the theme's text."),
                    prop("limit", "Option<usize>").doc("A cap on the rows drawn, counted through the groups."),
                    prop("close_on_action", "bool").default("true").doc("Close after running an action."),
                    prop("clear_on_close", "bool").default("true").doc("Start every opening with an empty query."),
                    prop("aria_label", "Option<String>").default("\"Command palette\"").doc("Names the dialog."),
                    prop("shortcut", "Option<char>").default("Some('k')").doc("Ctrl (Cmd on a Mac) plus this key toggles the palette from anywhere on the page. `None` for no hotkey. Web only. A key the browser already uses (L, T, W, R, F, ...) warns in a debug build."),
                    prop("highlight_first_on_query", "bool").default("true").doc("Highlight the first row after every keystroke, so `Enter` runs it without an `ArrowDown` first. Off, a fresh query arms nothing."),
                    prop("loading", "bool").default("false").doc("The results are still coming. A loader replaces the rows and \"nothing found\", and the status region says \"Searching\" (the theme's label)."),
                    prop("onquery", "Option<Callback<String>>").doc("Called with the new query on every keystroke, from the input event. Set `loading` and start the search here."),
                ]),
                props("SpotlightAction", vec![
                    prop("label", "String").doc("The row's text, and the first thing `spotlight_filter` matches."),
                    prop("description", "Option<String>").doc("A second line, matched after the label."),
                    prop("keywords", "Vec<String>").doc("Matched, never drawn."),
                    prop("group", "Option<String>").doc("A section header. Groups keep the order they first appear in."),
                    prop("icon", "Option<Element>").doc("Drawn before the label."),
                    prop("shortcut", "Option<String>").doc("A hint drawn as a `Kbd`. Never bound."),
                    prop("onclick", "Option<Callback<()>>").doc("Run on Enter or a click."),
                ]),
                props("SpotlightHandle", vec![
                    prop("open()", "()").doc("Opens with focus in the search box. Call it from the trigger's handler, so focus returns there."),
                    prop("close()", "()").doc("Closes."),
                    prop("toggle()", "()").doc("One or the other."),
                    prop("is_open()", "bool").doc("Whether it is open."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A command palette: a modal search box over a list of actions. "
                    Code { source: "use_spotlight" }
                    " returns a "
                    Code { source: "Copy" }
                    " handle, like "
                    Code { source: "use_modal" }
                    ". What it lists is yours: "
                    Code { source: "actions" }
                    " is called with the live query and returns the rows, so a fixed list and "
                    "search results are the same prop. "
                    Code { source: "spotlight_filter" }
                    " is the common case, label hits first. The "
                    Code { source: "example" }
                    " control picks which palette the button opens, and the code block follows "
                    "it. The hotkey here is J or P, because this site's own "
                    "search already owns "
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "K" }
                    "."
                }
            },
            Demo {
                component: "SpotlightOptions",
                children_text: "",
                controls: vec![
                    // Not a prop: which palette the one button opens, and the
                    // example the code block prints.
                    Control::toggle("example", ["commands", "files", "issues", "search"])
                        .labels(["Commands", "Files", "200 issues", "Slow search"]),
                    Control::toggle("shortcut", ["j", "p", "none"]),
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
                title: "Accessibility",
                Text {
                    "Focus stays in the search box. " Kbd { "↓" } " " Kbd { "↑" }
                    " move the highlight, wrapping at both ends. " Kbd { "Enter" }
                    " runs the highlighted action, which by default is the first row of the "
                    "last query. Turn "
                    Code { source: "highlight_first_on_query" }
                    " off for a palette whose actions do something, and typing then arms "
                    "nothing until you press " Kbd { "↓" } ". " Kbd { "Esc" }
                    " or a click outside closes, and focus goes back to what opened it. The "
                    "hotkey is ignored while you type in another text field, and while a dialog "
                    "or popover is open."
                }
            }
        }
    }
}
