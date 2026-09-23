//! `IconProvider`'s contract: a slot it sets replaces libero's own glyph below
//! it, nested providers merge per slot, and without one the lucide default draws.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot, LiberoProvider,
    components::{Checkbox, Options, Pictogram, Select, SvgData},
    hooks::use_icon,
};
use pictogram_icons_lucide as lucide;

/// Paths no lucide icon draws, so a match can only be the override.
const MINE: SvgData = SvgData::new(r#"<svg viewBox="0 0 24 24"><path d="M1 2h3"/></svg>"#);
const OUTER: SvgData = SvgData::new(r#"<svg viewBox="0 0 24 24"><path d="M5 6h7"/></svg>"#);

#[derive(Clone, PartialEq, Options)]
enum Pick {
    First,
    Second,
}

#[test]
fn a_checkbox_draws_the_provided_check_at_its_own_stroke() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                IconProvider { icons: IconSet::new().with(IconSlot::CheckboxCheck, MINE),
                    Checkbox { label: "A", checked: true, onchange: move |_| {} }
                }
            }
        }
    }
    let html = body(&render(app));
    assert!(html.contains(MINE.body), "{html}");
    assert!(!html.contains(lucide::check::outlined.body), "{html}");
    assert!(html.contains(r#"stroke-width="3""#), "{html}");
}

#[test]
fn a_select_draws_the_provided_chevron() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                IconProvider { icons: IconSet::new().with(IconSlot::ChevronDown, MINE),
                    Select { value: Pick::First, onchange: move |_: Option<Pick>| {} }
                }
            }
        }
    }
    let html = body(&render(app));
    assert!(html.contains(MINE.body), "{html}");
    assert!(
        !html.contains(lucide::chevron_down::outlined.body),
        "{html}"
    );
}

#[test]
fn without_a_provider_the_lucide_default_draws() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Select { value: Pick::First, onchange: move |_: Option<Pick>| {} }
            }
        }
    }
    let html = body(&render(app));
    assert!(html.contains(lucide::chevron_down::outlined.body), "{html}");
}

#[test]
fn an_unset_slot_keeps_its_default() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                IconProvider { icons: IconSet::new().with(IconSlot::Close, MINE),
                    Select { value: Pick::First, onchange: move |_: Option<Pick>| {} }
                }
            }
        }
    }
    let html = body(&render(app));
    assert!(html.contains(lucide::chevron_down::outlined.body), "{html}");
}

#[component]
fn Drawn(slot: IconSlot) -> Element {
    let icon = use_icon(slot, lucide::x::outlined);
    rsx! { Pictogram { icon } }
}

#[test]
fn the_inner_provider_wins_per_slot_and_the_outer_fills_the_rest() {
    fn app() -> Element {
        rsx! {
            IconProvider {
                icons: IconSet::new().with(IconSlot::Close, OUTER).with(IconSlot::Check, OUTER),
                IconProvider { icons: IconSet::new().with(IconSlot::Close, MINE),
                    Drawn { slot: IconSlot::Close }
                    Drawn { slot: IconSlot::Check }
                    Drawn { slot: IconSlot::Plus }
                }
            }
        }
    }
    let html = body(&render(app));
    let close = html.find(MINE.body).expect("the inner Close");
    let check = html.find(OUTER.body).expect("the outer Check");
    let plus = html.find(lucide::x::outlined.body).expect("the default");
    assert!(close < check && check < plus, "{html}");
    assert_eq!(html.matches(OUTER.body).count(), 1, "{html}");
}
