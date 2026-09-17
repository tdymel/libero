use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, DataList, DataListItem, Flex, Text};

/// Re-render cost as a multiple of `Leaf` - one scope, one `span`, no
/// styling. The full table is `tests/render_cost.rs`; this is its shape.
const RATIOS: [(&str, &str); 7] = [
    ("Box", "1.5x"),
    ("Text", "1.6x"),
    ("Flex", "2.1x"),
    ("Button", "2.4x"),
    ("Sidebar", "4.7x"),
    ("NativeSelect", "6.1x"),
    ("Splitter", "8.2x"),
];

/// Each row is `(what, cost, why it is worth knowing)`.
const COSTS: [(&str, &str, &str); 6] = [
    (
        "A component scope",
        "~300 ns",
        "The floor for anything that is its own component, and the price of a memoization boundary.",
    ),
    (
        "A dynamic node, any {..} hole in rsx!",
        "~400-600 ns",
        "Paid whether or not it renders anything. Static markup is free, because templates are const.",
    ),
    (
        "An EventHandler props field",
        "~300 ns",
        "Even when the caller never sets it, because the default allocates. Option<EventHandler<T>> costs ~95 ns.",
    ),
    (
        "An attribute through the builder",
        "~135 ns",
        "Including a false or None that renders nothing.",
    ),
    (
        "use_signal plus one read",
        "~290 ns",
        "The subscription bookkeeping, not the hook slot.",
    ),
    (
        "A hook slot",
        "~51 ns",
        "use_context, use_hook and use_drop are one each.",
    ),
];

// snippet: ignore - pseudocode: `Row { .. }` stands for any component
const MEMOIZE: &str = r#"// Re-renders every time the list does: a bare closure has a
// fresh identity each render, so the props never compare equal.
Row { onselect: move |_| selected.set(id), .. }

// Skipped while nothing about it changed.
let onselect = use_callback(move |_| selected.set(id));
Row { onselect, .. }"#;

const MEASURE: &str = "cargo test --release -p libero --test render_cost -- --ignored --nocapture";

#[component]
pub fn PerformancePage() -> Element {
    rsx! {
        DocPage {
            title: "Performance",
            markdown: "/md/performance.md",
            lead: rsx! {
                Text {
                    "Every number on this page was measured by ablation: remove a thing and "
                    "take the delta. Timers inside a render lie in both directions. "
                    "Attributes are diffed after "
                    Code { source: "rsx!" }
                    " returns, so a probe around a render body misses them, and a dozen "
                    "nested probes inflated one component by 25%."
                }
                Text {
                    "In short, neither styling nor the size of a props struct is what costs. "
                    "The shape of the tree is: how many component scopes and dynamic nodes "
                    "there are, and how much of it re-renders when something changes."
                }
            },
            DocSection {
                title: "What a component costs",
                Text {
                    "A Libero component is one Dioxus component scope plus one styling "
                    "pass. Against a component that renders a bare "
                    Code { source: "span" }
                    " and nothing else, the cheap ones ("
                    Code { source: "Box" }
                    ", "
                    Code { source: "Text" }
                    ", "
                    Code { source: "Center" }
                    ") land around 1.5x, and that cluster is the floor. Anything above it "
                    "pays for its own markup. "
                    Code { source: "NativeSelect" }
                    " renders a label, a trigger and a list, and "
                    Code { source: "Sidebar" }
                    " a bordered panel around a scroll area. The spread across the library:"
                }
                Flex {
                    direction: "row",
                    wrap: "wrap",
                    gap: "sm",
                    for (component, ratio) in RATIOS {
                        Code { key: "{component}", source: "{component} {ratio}" }
                    }
                }
                Text {
                    "Ratios travel between machines; absolute nanoseconds do not. So these "
                    "are an ordering of which components to look at first when a screen is "
                    "slow, not a budget."
                }
            }

            DocSection {
                title: "Styling is not the expensive part",
                Text {
                    "An "
                    Code { source: "Sx" }
                    " hashes to a class by its content, so a thousand elements styled alike "
                    "share one class and one CSS rule, built once. Per render, an unchanged "
                    Code { source: "sx" }
                    " costs a hash comparison. No CSS is re-rendered and no style attribute "
                    "is diffed."
                }
                Text {
                    "Three habits keep it that way, all also advised on the styling page. "
                    "Hoist a constant style into a "
                    Code { source: "StaticSx" }
                    ", so it is built once per process and compared by pointer. Switch "
                    "variants with "
                    Code { source: "states" }
                    ", so every variant shares one class. For an unbounded value, such as a "
                    "drag offset or a percentage, use "
                    Code { source: "Variables" }
                    ", which writes a CSS custom property into the style attribute instead "
                    "of a new class per value."
                }
            }

            DocSection {
                title: "Memoization boundaries",
                Text {
                    "A component scope is also a boundary Dioxus stops at. While a "
                    "component's props compare equal, its whole subtree is skipped. So "
                    "markup that changes with one piece of state, inside something that "
                    "re-renders for other reasons, is often cheaper as its own component. "
                    "Extracting the divider out of "
                    Code { source: "Splitter" }
                    " that way cost one scope and saved 27% of the component."
                }
                Text {
                    "It only works if every prop compares by value or by a stable identity. "
                    "Plain values, "
                    Code { source: "Signal" }
                    "s and a "
                    Code { source: "use_callback" }
                    " do. A bare closure never does, and neither does "
                    Code { source: "children" }
                    ", because two "
                    Code { source: "Element" }
                    "s compare by pointer. A component that takes children never memoizes."
                }
                CodeBlock { source: MEMOIZE, language: "rust" }
            }

            DocSection {
                title: "What costs",
                Text {
                    "Per re-rendered instance, all by ablation. The second row usually "
                    "surprises. A "
                    Code { source: "{{..}}" }
                    " hole in "
                    Code { source: "rsx!" }
                    " is paid for on every render even when its branch renders nothing, "
                    "while static markup around it is free."
                }
                DataList {
                    for (what, cost, note) in COSTS {
                        DataListItem {
                            key: "{what}",
                            label: rsx! {
                                Code { source: what }
                            },
                            Text {
                                Code { source: cost }
                                ". {note}"
                            }
                        }
                    }
                }
                Text {
                    "Props field count is free: nine extra optional fields on a component "
                    "that does nothing else measured at zero. Only the "
                    Code { source: "EventHandler" }
                    " is expensive, because its default allocates."
                }
            }

            DocSection {
                title: "Measuring it yourself",
                Text {
                    "The library's own harness covers every component, one row each, and "
                    "prints nanoseconds per re-rendered instance. It is ignored by default, "
                    "because the numbers only mean anything in release:"
                }
                CodeBlock { source: MEASURE, language: "bash" }
                Text {
                    "Four rules make a run trustworthy, most important first. Price a thing "
                    "by removing it, not by wrapping it in a timer. Keep a control row in "
                    "every run, so machine drift is visible. Interleave the variants and "
                    "keep a per-variant minimum, because sequential runs drift enough to "
                    "invert a sign. Compare against a baseline you ran on the same machine, "
                    "never against a number someone wrote down."
                }
            }
        }
    }
}
