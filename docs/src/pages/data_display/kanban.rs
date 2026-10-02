use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Avatar, Badge, Code, Flex, Icon, Kanban, KanbanCard, KanbanCardPart, KanbanColumn,
    KanbanColumnPart, KanbanMove, SvgData, Text, VisuallyHidden,
};
use libero::sx::sx;
use pictogram_icons_lucide as lucide;

const COLUMNS: [&str; 3] = ["To do", "In progress", "Done"];

/// An issue's type and priority: an icon, its colour and its name.
type Look = (SvgData, &'static str, &'static str);

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Story,
    Bug,
    Task,
}

impl Kind {
    fn look(self) -> Look {
        match self {
            Kind::Story => (lucide::bookmark::outlined, "success", "Story"),
            Kind::Bug => (lucide::bug::outlined, "error", "Bug"),
            Kind::Task => (lucide::square_check::outlined, "info", "Task"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Priority {
    Highest,
    High,
    Medium,
    Low,
}

impl Priority {
    fn look(self) -> Look {
        match self {
            Priority::Highest => (lucide::chevrons_up::outlined, "error", "Highest priority"),
            Priority::High => (lucide::arrow_up::outlined, "warning", "High priority"),
            Priority::Medium => (lucide::equal::outlined, "info", "Medium priority"),
            Priority::Low => (lucide::arrow_down::outlined, "success", "Low priority"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Issue {
    key: &'static str,
    title: &'static str,
    kind: Kind,
    priority: Priority,
    labels: &'static [&'static str],
    points: u8,
    /// The assignee's name, initials and avatar colour.
    assignee: (&'static str, &'static str, &'static str),
}

const ADA: (&str, &str, &str) = ("Ada Lovelace", "AL", "primary");
const GRACE: (&str, &str, &str) = ("Grace Hopper", "GH", "secondary");
const RADIA: (&str, &str, &str) = ("Radia Perlman", "RP", "warning");

fn issues() -> Vec<Vec<Issue>> {
    let issue = |key, title, kind, priority, labels, points, assignee| Issue {
        key,
        title,
        kind,
        priority,
        labels,
        points,
        assignee,
    };
    vec![
        vec![
            issue(
                "LIB-142",
                "Drag cards between columns",
                Kind::Story,
                Priority::High,
                &["kanban", "a11y"],
                5,
                ADA,
            ),
            issue(
                "LIB-147",
                "Focus ring clipped in a scrolled board",
                Kind::Bug,
                Priority::Highest,
                &["a11y"],
                2,
                GRACE,
            ),
            issue(
                "LIB-150",
                "Document the column states",
                Kind::Task,
                Priority::Low,
                &["docs"],
                1,
                RADIA,
            ),
        ],
        vec![
            issue(
                "LIB-139",
                "Touch drag on Android",
                Kind::Story,
                Priority::Medium,
                &["android"],
                8,
                GRACE,
            ),
            issue(
                "LIB-145",
                "Announce moves across columns",
                Kind::Task,
                Priority::High,
                &["a11y"],
                3,
                ADA,
            ),
        ],
        vec![issue(
            "LIB-131",
            "Move to column menu",
            Kind::Story,
            Priority::Medium,
            &["kanban"],
            3,
            RADIA,
        )],
    ]
}

/// One issue as a Jira card: title, labels, then type, key, priority, points and assignee.
#[component]
fn IssueCard(issue: Issue) -> Element {
    let (kind, kind_color, kind_name) = issue.kind.look();
    let (priority, priority_color, priority_name) = issue.priority.look();
    let (name, initials, color) = issue.assignee;
    rsx! {
        Flex { gap: "xs",
            Text { size: "sm", "{issue.title}" }
            Flex { direction: "row", gap: "4px",
                for label in issue.labels {
                    Badge { key: "{label}", variant: "outlined", color: "secondary", size: "xs", "{label}" }
                }
            }
            Flex { direction: "row", justify: "space-between", gap: "xs", wrap: false,
                Flex { direction: "row", gap: "4px", wrap: false,
                    Icon { svg: kind, color: kind_color, variant: "standard", size: "sm", role: "img", aria_label: kind_name }
                    Text { component: "span", size: "xs", sx: sx().color("text-dimmed").white_space("nowrap"), "{issue.key}" }
                }
                Flex { direction: "row", gap: "4px", wrap: false,
                    Icon { svg: priority, color: priority_color, variant: "standard", size: "sm", role: "img", aria_label: priority_name }
                    Badge { variant: "tonal", color: "muted", size: "sm", circle: true,
                        "{issue.points}"
                        VisuallyHidden { " story points" }
                    }
                    Avatar { name, initials, color, size: "xs" }
                }
            }
        }
    }
}

/// A column header: the name, then the card count.
#[component]
fn Header(label: &'static str, count: usize) -> Element {
    rsx! {
        Flex { direction: "row", justify: "space-between",
            Text { component: "span", size: "sm", "{label}" }
            Badge { variant: "tonal", color: "muted", size: "sm", circle: true, "{count}" }
        }
    }
}

/// The board in the code tab: the demo's wiring, with a minimal card.
fn code(values: &DemoValues, _: &str) -> String {
    let buttons = match values.str("move_buttons").as_str() {
        "false" => "\n        move_buttons: false,",
        _ => "",
    };
    format!(
        r#"let columns = ["To do", "In progress", "Done"];
let mut cards = use_signal(|| vec![
    vec!["Drag cards between columns", "Document the column states"],
    vec!["Touch drag on Android"],
    vec![],
]);

rsx! {{
    Kanban {{{buttons}
        onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
        for (column, label) in columns.into_iter().enumerate() {{
            KanbanColumn {{ key: "{{label}}", index: column, label,
                for (index, title) in cards()[column].clone().into_iter().enumerate() {{
                    KanbanCard {{ key: "{{title}}", index, label: title,
                        // Any content: the demo's issue card is Text, Badge, Icon and Avatar.
                        Text {{ size: "sm", "{{title}}" }}
                    }}
                }}
            }}
        }}
    }}
}}"#
    )
}

#[component]
fn Board(move_buttons: bool) -> Element {
    let mut issues = use_signal(issues);
    rsx! {
        Kanban {
            move_buttons,
            onmove: move |step: KanbanMove| step.apply(&mut issues.write()),
            for (column, label) in COLUMNS.into_iter().enumerate() {
                KanbanColumn { key: "{label}", index: column, label,
                    header: rsx! { Header { label, count: issues()[column].len() } },
                    for (index, issue) in issues()[column].clone().into_iter().enumerate() {
                        KanbanCard { key: "{issue.key}", index,
                            label: format!("{} {}", issue.key, issue.title),
                            IssueCard { issue }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn KanbanPage() -> Element {
    rsx! {
        DocPage {
            title: "Kanban",
            source: "libero/src/components/data_display/kanban",
            markdown: "/md/kanban.md",
            properties: vec![
                props("Kanban", vec![
                    prop("onmove", "EventHandler<KanbanMove>")
                        .doc("Fires when a card changed place: on a drop, a move button or a Move to entry. Apply it to your data with `KanbanMove::apply`; until then the board keeps its old order."),
                    prop("move_buttons", "bool")
                        .default("true")
                        .doc("Each card's two buttons that move it one slot in its column without dragging (WCAG 2.5.7)."),
                    prop("children", "Element").doc("The `KanbanColumn`s."),
                ]),
                props("KanbanColumn", vec![
                    prop("index", "usize")
                        .doc("The column's position on the board, from 0. Key the column by its data, not by this."),
                    prop("label", "String")
                        .doc("The column's name: the header's text, the card list's accessible name, its entry in every Move to menu and the announcements."),
                    prop("header", "Option<Element>")
                        .doc("The header's content instead of `label`, e.g. with a count. `label` stays the list's name."),
                    prop("parts", "Parts<KanbanColumnPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").doc("The column's `KanbanCard`s."),
                ])
                .parts("KanbanColumnPart", vec![
                    (KanbanColumnPart::Header, "The header naming the column."),
                    (KanbanColumnPart::List, "The list holding the cards."),
                ]),
                props("KanbanCard", vec![
                    prop("index", "usize")
                        .doc("The card's position in its column, from 0. Key the card by its data, not by this."),
                    prop("label", "Option<String>")
                        .default("\"Item {n}\"")
                        .doc("Names the card in its controls and the announcements. Unset, `SortableLabels::item` with its position."),
                    prop("parts", "Parts<KanbanCardPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").doc("The card's content, between the handle and the move buttons."),
                ])
                .parts("KanbanCardPart", vec![
                    (KanbanCardPart::Handle, "The drag handle."),
                    (KanbanCardPart::Content, "The wrapper round the card's content."),
                    (KanbanCardPart::MoveEarlier, "The move up button."),
                    (KanbanCardPart::MoveLater, "The move down button."),
                    (KanbanCardPart::MoveTo, "The Move to column menu's trigger."),
                ]),
            ],
            accessibility: a11y()
                .key(["Space", "Enter"], "On a handle, lifts its card. Lifted, drops it where it is.")
                .key(["Up", "Down"], "Moves a lifted card one slot in its column.")
                .key(["Home", "End"], "Moves a lifted card to the top or bottom of its column.")
                .key(["Escape"], "Puts a lifted or dragged card back where it was.")
                .handles([
                    "Each column is a list named by its `label`; each card is a list item.",
                    "Each card's handle is a button named by `SortableLabels::handle` with the card's name (\"Reorder Write\"), at least 24px square (WCAG 2.5.8), described by `SortableLabels::instructions`.",
                    "Each card has a move up and a move down button in its column, and a Move to column menu button named by `KanbanLabels::move_to`, each naming the card (\"Move Write up\", \"Move Write to column\"). The menu lists every column by `label`, the card's own disabled. Neither needs a drag (WCAG 2.5.7).",
                    "A card moved to another column by its Move to menu lands at that column's end, and its Move to button takes the focus there. One dragged there lands where it was let go, and its handle keeps the focus.",
                    "One `role=\"status\"` region for the board announces each lift, move, drop and cancel, and a move or a drop in another column with the column's name and the card's position, from the `SortableLabels` and `KanbanLabels` templates of the active `Localization`.",
                    "A drag starts after the pointer moved 4px (8px for a touch); only the handle takes a touch, so a swipe on the rest of a card scrolls. The column under the dragged card's centre takes it, marked `target`; let go off the board, the card goes back.",
                    "On a board wider than its container, a card dragged within 48px of a side edge scrolls the board that way, faster the closer, until that edge's column is in view.",
                    "In a narrow column, where the content would get less than 8rem, the move buttons wrap below it.",
                ])
                .must([
                    "Give each `KanbanCard` a `label`, or its controls and the announcements name it by position (\"Item 2\").",
                ])
                .limits([
                    "The keyboard drag moves a card within its column only. Moving to another column is the Move to menu.",
                    "A drag scrolls the board sideways only, not the page: to reach a card or column above or below the window, use the Move to menu.",
                    "Columns keep their place: the board has no column reorder.",
                ]),
            lead: rsx! {
                Text {
                    "A board of columns. A card moves by dragging its handle, in its column or to another; "
                    "by keyboard or with its move buttons in its column; and to another column with its Move to menu. "
                    Code { source: "onmove" }
                    " gets a "
                    Code { source: "KanbanMove" }
                    " with the old and new column and index, and "
                    Code { source: "apply" }
                    " does it to a "
                    Code { source: "Vec<Vec<T>>" }
                    ". The cards are your content: the board draws no card look. The demo builds Jira-style "
                    "issue cards from Text, Badge, Icon and Avatar."
                }
            },
            Demo {
                component: "Kanban",
                children_text: "",
                controls: vec![Control::switch("move_buttons").default("true")],
                render: move |values: DemoValues| rsx! {
                    Board { move_buttons: values.str("move_buttons") != "false" }
                },
                wrap: Wrap(code),
                wide_preview: true,
            }
        }
    }
}
