use dioxus::prelude::*;
use libero::{
    components::{Input, Orientation, Sortable, SortableItem},
    hooks::SortableMove,
    theme::Responsive,
};

#[component]
pub fn Fruit(#[props(default, into)] orientation: Input<Responsive<Orientation>>) -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);
    rsx! {
        Sortable {
            orientation,
            onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
            for (index, name) in fruit().into_iter().enumerate() {
                SortableItem { key: "{name}", index, label: name, "{name}" }
            }
        }
    }
}
