//! Two selects, one under an `IconProvider` whose `ChevronDown` a button swaps
//! between two glyphs (1094): one slot replaced, the other select untouched.
//! A third sits under a whole set, `IconSet::tabler_outlined()` (1133).

use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot,
    components::{Button, Options, Select, SvgData},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/icon-provider", || rsx! { IconProviderPage {} })];

const BAR: SvgData = SvgData::new(
    r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path data-glyph="bar" d="M4 12h16"/></svg>"#,
);
const ARROW: SvgData = SvgData::new(
    r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path data-glyph="arrow" d="M12 4v16M6 14l6 6 6-6"/></svg>"#,
);

#[derive(Clone, PartialEq, Options)]
enum Fruit {
    Apple,
    Pear,
}

#[component]
fn IconProviderPage() -> Element {
    let mut arrows = use_signal(|| false);
    let chevron = if arrows() { ARROW } else { BAR };

    rsx! {
        Button { id: "swap", onclick: move |_| arrows.toggle(), "Swap glyph" }
        div { id: "default",
            Select { label: "Default", value: Fruit::Apple, onchange: move |_| {} }
        }
        IconProvider { icons: IconSet::new().with(IconSlot::ChevronDown, chevron),
            div { id: "provided",
                Select { label: "Provided", value: Fruit::Apple, onchange: move |_| {} }
            }
        }
        IconProvider { icons: IconSet::tabler_outlined(),
            div { id: "tabler",
                Select { label: "Tabler", value: Fruit::Apple, onchange: move |_| {} }
            }
        }
    }
}
