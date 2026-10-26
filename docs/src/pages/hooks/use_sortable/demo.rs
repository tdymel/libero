use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Box, Orientation, Pictogram, VisuallyHidden},
    hooks::{
        SortableMove, SortableOptions, drag_handle_sx, use_id, use_labelled_sortable_item,
        use_sortable,
    },
    sx::sx,
};
use pictogram_icons_lucide as lucide;

#[component]
pub fn Fruit(#[props(default, into)] orientation: Orientation) -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);
    let list = use_sortable(SortableOptions {
        orientation,
        onreorder: Callback::new(move |step: SortableMove| step.apply(&mut fruit.write())),
    });
    let horizontal = orientation == Orientation::Horizontal;

    rsx! {
        Box {
            role: "list",
            onmounted: list.element.mount(),
            onpointermove: move |event| list.onpointermove.call(event),
            onpointerup: move |event| list.onpointerup.call(event),
            onpointercancel: move |event| list.onpointercancel.call(event),
            sx: sx().display("flex").gap("xs").flex_direction(if horizontal { "row" } else { "column" }),
            for (index, name) in fruit().into_iter().enumerate() {
                Item { key: "{name}", index, name, horizontal }
            }
        }
        VisuallyHidden { role: "status", {list.announcement} }
    }
}

#[component]
fn Item(index: usize, name: &'static str, horizontal: bool) -> Element {
    let item = use_labelled_sortable_item(index, Some(name.to_string()));
    let help = use_id();
    let (earlier, later) = match horizontal {
        true => (
            lucide::chevron_left::outlined,
            lucide::chevron_right::outlined,
        ),
        false => (lucide::chevron_up::outlined, lucide::chevron_down::outlined),
    };
    let mut row = sx()
        .display("flex")
        .align_items("center")
        .gap("xs")
        .padding("xs")
        .border_radius("sm")
        .background("muted.1")
        .position("relative");
    if (item.dragging)() {
        row = row.z_index("1").box_shadow("0 4px 12px rgb(0 0 0 / 25%)");
    } else if (item.sorting)() {
        row = row.transition("transform 150ms ease");
    }

    rsx! {
        Box {
            role: "listitem",
            onmounted: item.element.mount(),
            style: item.style(),
            sx: row,
            Box {
                component: "button",
                r#type: "button",
                aria_label: "Reorder {name}",
                aria_describedby: "{help}",
                onmounted: item.handle.mount(),
                onpointerdown: move |event| item.onpointerdown.call(event),
                onkeydown: move |event| item.onkeydown.call(event),
                onblur: move |event| item.onblur.call(event),
                sx: drag_handle_sx().padding("0").border("0").background("transparent")
                    .color("inherit").cursor("grab"),
                Pictogram { icon: lucide::grip_vertical::outlined, width: "1em", height: "1em" }
            }
            Box { id: "{help}", hidden: true, {item.instructions} }
            Box { component: "span", "{name}" }
            ActionIcon {
                aria_label: "Move {name} earlier",
                size: "sm",
                icon: earlier,
                disabled: (item.first)(),
                onclick: move |event| item.onearlier.call(event),
            }
            ActionIcon {
                aria_label: "Move {name} later",
                size: "sm",
                icon: later,
                disabled: (item.last)(),
                onclick: move |event| item.onlater.call(event),
            }
        }
    }
}
