use dioxus::prelude::*;

use super::tabs::TabsPart;
use crate::{
    components::{
        common::{
            COARSE_POINTER, ClassList, HtmlTag, Input, Part, States, Variables, coarse_hit_area_sx,
            focus_ring_sx, focused_disabled_sx, has_shortcut_modifier, inset_focus_ring_sx,
            names_itself, neighbour, reveal_block, reveal_inline, text_color, use_name_warning,
            variables,
        },
        layout::use_box,
    },
    hooks::{ElementHandle, id_selector, use_element, use_focus_within},
    platform::{ElementApi, focus_selector, logical_key, when_laid_out},
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, Size, SizeCss, TABS_BORDER_COLOR, TABS_GAP, TABS_HOVER, TABS_LINE, TABS_PAD_X,
        TABS_PAD_Y, TabsDefaults, TabsPlacement,
    },
};

const TABS_COLOR: CssVar = CssVar::new("--lsx-tabs-color");

static TABS_SX: StaticSx = StaticSx::new(|| {
    let line = format!("inset 0 -1px 0 {}", TABS_BORDER_COLOR.value());

    TabsDefaults::theme_vars()
        .display("block")
        .selector(
            "& > [role=\"tablist\"]",
            // Scrolls inside itself (1.4.10). An inset shadow, as a border would
            // clip the selected tab's underline.
            sx().display("flex")
                .align_items("stretch")
                .overflow_x("auto")
                .box_shadow(line),
        )
        .selector(
            "& [role=\"tab\"]",
            // Centres a rich label's icon on the text.
            sx().display("inline-flex")
                .align_items("center")
                .justify_content("center")
                .gap(TABS_GAP.value())
                .appearance("none")
                .background("transparent")
                .border("0")
                .border_bottom(format!("{} solid transparent", TABS_LINE.value()))
                .padding(format!("{} {}", TABS_PAD_Y.value(), TABS_PAD_X.value()))
                .font("inherit")
                .letter_spacing("inherit")
                .color("inherit")
                // The strip scrolls; a label wider than it wraps (1.4.10).
                .flex_shrink("0")
                .max_width("100%")
                .with("overflow-wrap", "break-word")
                .cursor("pointer")
                // The scrolling strip clips it vertically, so it widens a narrow icon tab only;
                // a finger gets the height from the tab itself (todo 3596).
                .position("relative")
                .and(coarse_hit_area_sx("::before"))
                .media(COARSE_POINTER, sx().min_height("44px")),
        )
        // Replaces the UA ring `appearance: none` drops; inset, or the strip clips it.
        .selector(
            "& [role=\"tab\"]:focus-visible",
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            "& [role=\"tab\"]:hover:not([aria-selected=\"true\"]):not([aria-disabled=\"true\"])",
            sx().background(TABS_HOVER.value()),
        )
        .selector(
            "& [role=\"tab\"][aria-selected=\"true\"]",
            sx().color(TABS_COLOR.value())
                .border_bottom_color(TABS_COLOR.value()),
        )
        // Forced colours paint every transparent underline; keep only the selected one.
        .media(
            FORCED_COLORS,
            sx().selector("& [role=\"tab\"]", sx().border_bottom_color("Canvas"))
                .selector(
                    "& [role=\"tab\"][aria-selected=\"true\"]",
                    sx().border_bottom_color("Highlight"),
                ),
        )
        .selector(
            "& [role=\"tab\"][aria-disabled=\"true\"]",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .and(focused_disabled_sx()),
        )
        .selector(
            "& > [role=\"tabpanel\"]",
            sx().padding_top(SizeCss::SPACING.value(Size::Md)),
        )
        // The panel is a tab stop too. Doubled to outrank a component's own ring
        // whatever the order, as `Carousel`.
        .selector(
            "& > [role=\"tabpanel\"]:focus-visible:focus-visible",
            focus_ring_sx(),
        )
        .when(
            "full-width",
            // Else a flex parent shrinks the root to the tabs. Never below the label,
            // so a crowded strip scrolls; direct path, so nested tabs keep their own.
            sx().width("100%").selector(
                "& > [role=\"tablist\"] > [role=\"tab\"]",
                sx().flex("1 0 auto"),
            ),
        )
        .when("vertical", vertical_sx())
        // The strip's line and the selected tab's indicator face the panel.
        .when(
            TabsPlacement::Start.state_name(),
            vertical_side_sx("inline-end", "-1px", "1px"),
        )
        .when(
            TabsPlacement::End.state_name(),
            vertical_side_sx("inline-start", "1px", "-1px").flex_direction("row-reverse"),
        )
});

/// The strip as a column beside the panel; direct paths, so nested tabs keep their own.
fn vertical_sx() -> crate::sx::Sx {
    sx().display("flex")
        .align_items("flex-start")
        .selector(
            "& > [role=\"tablist\"]",
            sx().flex_direction("column")
                .flex_shrink("0")
                .overflow_x("visible")
                .overflow_y("auto")
                .max_width("50%"),
        )
        .selector(
            "& > [role=\"tablist\"] > [role=\"tab\"]",
            sx().border_bottom("0")
                .justify_content("flex-start")
                .text_align("start"),
        )
        .selector(
            "& > [role=\"tabpanel\"]",
            sx().flex("1 1 0").min_width("0").padding_top("0"),
        )
}

/// One vertical side: the strip's inset line (physical, flipped in RTL), the tabs'
/// indicator on `edge`, and the panel's gap on the far side.
fn vertical_side_sx(edge: &str, ltr: &str, rtl: &str) -> crate::sx::Sx {
    let line = |x: &str| format!("inset {x} 0 0 {}", TABS_BORDER_COLOR.value());
    let tab = "& > [role=\"tablist\"] > [role=\"tab\"]";
    let border = format!("border-{edge}");
    let panel_gap = match edge {
        "inline-end" => "padding-inline-start",
        _ => "padding-inline-end",
    };
    sx().selector("& > [role=\"tablist\"]", sx().box_shadow(line(ltr)))
        .rtl(sx().selector("& > [role=\"tablist\"]", sx().box_shadow(line(rtl))))
        .selector(
            tab,
            sx().with(&border, format!("{} solid transparent", TABS_LINE.value())),
        )
        .selector(
            format!("{tab}[aria-selected=\"true\"]"),
            sx().with(format!("{border}-color"), TABS_COLOR.value()),
        )
        .media(
            FORCED_COLORS,
            sx().selector(tab, sx().with(format!("{border}-color"), "Canvas"))
                .selector(
                    format!("{tab}[aria-selected=\"true\"]"),
                    sx().with(format!("{border}-color"), "Highlight"),
                ),
        )
        .selector(
            "& > [role=\"tabpanel\"]",
            sx().with(panel_gap, SizeCss::SPACING.value(Size::Md)),
        )
}

/// One tab, with `T` already gone: `content` is the rendered label and `name`
/// the accessible one, or empty for the content's own.
#[derive(Clone, PartialEq)]
pub(crate) struct TabSpec {
    /// Stays with the tab as the list changes, so focus and the tab stop do too.
    pub key: String,
    pub name: String,
    pub content: Element,
    pub disabled: bool,
}

pub(crate) struct TabsView {
    pub tabs: Vec<TabSpec>,
    /// `None` when `value` is not among the tabs - the strip still renders.
    pub selected: Option<usize>,
    pub panel: Element,
    pub onselect: Callback<usize>,
    pub color: ThemeAwareValue,
    pub full_width: bool,
    /// `start` or `end` make a vertical strip, moved by Up and Down.
    pub placement: TabsPlacement,
    /// Arrows move the focus only; the tab's own click (Enter, Space) selects.
    pub manual: bool,
    /// `false` keeps the tabs out of the tab order, for a picker in a dropdown.
    pub focusable: bool,
    /// The panel is a tab stop (APG, for content that can't take focus).
    pub panel_stop: bool,
    pub size: Size,
    pub class: Input<ClassList>,
    pub sx: Input<crate::sx::Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// A plain `fn`: `Vec<Element>` props defeat memoization, so a scope buys nothing.
pub(crate) fn render_tabs(view: TabsView, root: String) -> Element {
    let TabsView {
        tabs,
        selected,
        panel,
        onselect,
        color,
        full_width,
        placement,
        manual,
        focusable,
        panel_stop,
        size,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let root_element = use_element();
    let vertical = placement != TabsPlacement::Top;

    let states: Input<States> = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("full-width", full_width)
        .with("vertical", vertical)
        .with(placement.state_name(), vertical)
        .into();
    // The text role, shared by the selected label and its underline.
    let variables: Input<Variables> = variables().with(TABS_COLOR, text_color(&color)).into();

    // The caller's name goes on the tablist: `aria-label` is prohibited on the role-less root.
    let (naming, attributes): (Vec<Attribute>, Vec<Attribute>) = attributes
        .into_iter()
        .partition(|attribute| matches!(attribute.name, "aria-label" | "aria-labelledby"));
    let named = names_itself(&naming);
    use_name_warning(
        named,
        "Tabs: no `aria-label` or `aria-labelledby`, so the tab list is announced without a name.",
    );
    let naming = if named { naming } else { Vec::new() };
    let panel_id = selected.map(|selected| format!("{root}-panel-{selected}"));
    let panel_labelled_by = selected.map(|selected| format!("{root}-tab-{selected}"));

    use_box()
        .framework_sx(&TABS_SX)
        .class(&class)
        .sx(&user_sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .element(&root_element)
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                TabStrip {
                    tabs,
                    selected,
                    onselect,
                    manual,
                    vertical,
                    focusable,
                    root: root.clone(),
                    root_element,
                    naming,
                }
                if let (Some(panel_id), Some(labelled_by)) = (panel_id, panel_labelled_by) {
                    div {
                        id: "{panel_id}",
                        role: "tabpanel",
                        "aria-labelledby": "{labelled_by}",
                        tabindex: (focusable && panel_stop).then_some("0"),
                        "data-slot": TabsPart::Panel.slot(),
                        {panel}
                    }
                }
            },
        )
}

/// The tablist, its own scope: a focus move redraws the strip, not the panel.
#[component]
fn TabStrip(
    tabs: Vec<TabSpec>,
    selected: Option<usize>,
    onselect: Callback<usize>,
    manual: bool,
    vertical: bool,
    focusable: bool,
    root: String,
    root_element: ElementHandle,
    naming: Vec<Attribute>,
) -> Element {
    let disabled: Vec<bool> = tabs.iter().map(|tab| tab.disabled).collect();
    // One tab stop: the focused tab, else the selected one, else the first enabled.
    let keys: Vec<String> = tabs.iter().map(|tab| tab.key.clone()).collect();
    let focused = use_signal(|| None::<String>);
    let tab_stop = focused
        .read()
        .as_ref()
        .and_then(|key| keys.iter().position(|tab| tab == key))
        .or(selected)
        .or_else(|| disabled.iter().position(|off| !off));
    let focus_on = move |key: Option<String>| {
        if *focused.peek() != key {
            let mut focused = focused;
            focused.set(key);
        }
    };
    let list_element = use_element();
    // Focus leaving the strip hands the tab stop back to the selected tab.
    // Also on Blitz, whose Tab fires no blur.
    let focus = use_focus_within(
        move || vec![list_element.mounted()],
        move |change| {
            if !change.within {
                focus_on(None);
            }
        },
    );
    let keydown_root = root.clone();
    // From the focused tab, which may be a clicked disabled one.
    let onkeydown = use_callback(move |(at, event): (usize, Event<KeyboardData>)| {
        // A lone enabled tab has nowhere to go: the keys scroll the page (todo 2465).
        if has_shortcut_modifier(&event) || neighbour(&disabled, at, 1) == Some(at) {
            return;
        }
        // APG: Up and Down on a vertical strip, Left and Right on a horizontal one.
        let (forward, back) = match vertical {
            true => (NamedKey::ArrowDown, NamedKey::ArrowUp),
            false => (NamedKey::ArrowRight, NamedKey::ArrowLeft),
        };
        let next = match logical_key(&event) {
            Key::Named(key) if key == forward => neighbour(&disabled, at, 1),
            Key::Named(key) if key == back => neighbour(&disabled, at, -1),
            Key::Named(NamedKey::Home) => disabled.iter().position(|off| !off),
            Key::Named(NamedKey::End) => disabled.iter().rposition(|off| !off),
            _ => return,
        };
        // Home on the first or End on the last tab scrolls the page (todo 3053).
        let Some(next) = next.filter(|&next| next != at) else {
            return;
        };
        event.prevent_default();
        if !manual {
            onselect.call(next);
        }
        // Blitz fires no focus event for a scripted `focus()`.
        focus_on(keys.get(next).cloned());
        let tab = id_selector(&format!("{keydown_root}-tab-{next}"));
        match root_element.query_selector(&tab) {
            Ok(tab) => {
                let _ = tab.focus();
            }
            // A WebView queries nothing: the page focuses it (2871).
            Err(_) => {
                let _ = focus_selector(&tab);
            }
        }
    });

    // A selected tab past the overflowing strip's edge scrolls into view, also a late `value`.
    let reveal_root = root.clone();
    use_effect(use_reactive!(|(selected, vertical)| {
        if let Some(index) = selected {
            let tab = id_selector(&format!("{reveal_root}-tab-{index}"));
            when_laid_out(move || match vertical {
                true => reveal_block(list_element, &tab),
                false => reveal_inline(list_element, &tab),
            });
        }
    }));

    rsx! {
        div {
            id: "{root}-tablist",
            role: "tablist",
            "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            "data-slot": TabsPart::List.slot(),
            onmounted: list_element.mount(),
            onfocusout: focus.focusout(0),
            ..naming,
            for (index, tab) in tabs.iter().enumerate() {
                button {
                    key: "{tab.key}",
                    id: "{root}-tab-{index}",
                    r#type: "button",
                    role: "tab",
                    // Strings, not bools: SSR writes a bare `true`, and the
                    // selected-tab CSS matches on `[aria-selected="true"]`.
                    "aria-selected": if selected == Some(index) { "true" } else { "false" },
                    // Only the selected panel exists to point at.
                    "aria-controls": (selected == Some(index)).then(|| format!("{root}-panel-{index}")),
                    // Not `disabled`: the tab stays reachable.
                    "aria-disabled": if tab.disabled { "true" } else { "false" },
                    // An empty name leaves the content to name the tab.
                    "aria-label": (!tab.name.is_empty()).then(|| tab.name.clone()),
                    tabindex: if focusable && tab_stop == Some(index) { "0" } else { "-1" },
                    "data-slot": TabsPart::Tab.slot(),
                    onkeydown: move |event| onkeydown.call((index, event)),
                    onfocus: {
                        let key = tab.key.clone();
                        move |_| focus_on(Some(key.clone()))
                    },
                    onclick: {
                        let disabled = tab.disabled;
                        let key = tab.key.clone();
                        move |_| {
                            focus_on(Some(key.clone()));
                            if !disabled {
                                onselect.call(index);
                            }
                        }
                    },
                    {tab.content.clone()}
                }
            }
        }
    }
}
