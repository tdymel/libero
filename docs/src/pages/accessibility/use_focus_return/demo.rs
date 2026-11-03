use dioxus::prelude::*;
use libero::{
    components::{Button, Checkbox, Flex},
    hooks::{use_focus_return, use_id},
    sx::sx,
};

#[component]
pub fn Filters() -> Element {
    let mut open = use_signal(|| false);
    let mut in_stock = use_signal(|| false);
    let trigger = use_focus_return();
    let panel = use_id();

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Button {
                variant: "outlined",
                aria_expanded: open(),
                // Only while the panel exists: a closed one leaves no id to point at.
                aria_controls: open().then_some(panel()),
                onclick: move |_| {
                    if !open() {
                        trigger.remember_active();
                    }
                    open.toggle();
                },
                "Filters"
            }
            if open() {
                Flex {
                    id: panel(),
                    direction: "column",
                    align: "flex-start",
                    gap: "sm",
                    sx: sx().padding("md").background("muted.1").border_radius("8px"),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Named(NamedKey::Escape) {
                            open.set(false);
                            trigger.restore();
                        }
                    },
                    Checkbox {
                        label: "In stock only",
                        checked: in_stock(),
                        onchange: move |next| in_stock.set(next),
                    }
                    Button {
                        onclick: move |_| {
                            open.set(false);
                            trigger.restore();
                        },
                        "Apply"
                    }
                }
            }
        }
    }
}
