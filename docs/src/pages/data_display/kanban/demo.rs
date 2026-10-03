use dioxus::prelude::*;
use libero::components::{
    Avatar, Badge, Flex, Icon, Kanban, KanbanCard, KanbanColumn, KanbanMove, SvgData, Text,
    VisuallyHidden,
};
use libero::sx::sx;
use pictogram_icons_lucide::{
    arrow_down, arrow_up, bookmark, bug, chevrons_up, equal, square_check,
};

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
            Kind::Story => (bookmark::outlined, "success", "Story"),
            Kind::Bug => (bug::outlined, "error", "Bug"),
            Kind::Task => (square_check::outlined, "info", "Task"),
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
            Priority::Highest => (chevrons_up::outlined, "error", "Highest priority"),
            Priority::High => (arrow_up::outlined, "warning", "High priority"),
            Priority::Medium => (equal::outlined, "info", "Medium priority"),
            Priority::Low => (arrow_down::outlined, "success", "Low priority"),
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

// demo-code: start
const COLUMNS: [&str; 3] = ["To do", "In progress", "Done"];

/// The cards are your content: here `IssueCard`, built from Text, Badge, Icon and Avatar.
#[component]
pub fn Board(#[props(default = true)] move_buttons: bool) -> Element {
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
// demo-code: end
