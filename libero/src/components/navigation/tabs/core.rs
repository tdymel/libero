use dioxus::prelude::*;

use super::tabs::TabsPart;
use crate::{
    components::{
        common::{
            ClassList, HtmlTag, Input, Part, States, Variables, focus_ring_sx,
            has_shortcut_modifier, inset_focus_ring_sx, neighbour, text_color, variables,
        },
        layout::use_box,
    },
    hooks::{id_selector, use_element, use_focus_within},
    platform::{ElementApi, logical_key},
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, Size, SizeCss, TABS_BORDER_COLOR, TABS_GAP, TABS_HOVER, TABS_LINE, TABS_PAD_X,
        TABS_PAD_Y, TabsDefaults,
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
                .color("inherit")
                // The strip scrolls; a label wider than it wraps (1.4.10).
                .flex_shrink("0")
                .max_width("100%")
                .with("overflow-wrap", "anywhere")
                .cursor("pointer"),
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
            sx().opacity("0.5").cursor("not-allowed"),
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
            // Else a flex parent shrinks the root to the tabs.
            sx().width("100%")
                .selector("& [role=\"tab\"]", sx().flex("1 1 0")),
        )
});

/// One tab, with `T` already gone: `content` is the rendered label and `name`
/// the accessible one, or empty for the content's own.
pub(crate) struct TabSpec {
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
        manual,
        focusable,
        panel_stop,
        size,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let disabled: Vec<bool> = tabs.iter().map(|tab| tab.disabled).collect();
    // One tab stop: the focused tab, else the selected one, else the first enabled.
    let mut focused = use_signal(|| None::<usize>);
    let tab_stop = focused()
        .filter(|&index| index < tabs.len())
        .or(selected)
        .or_else(|| disabled.iter().position(|off| !off));
    let root_element = use_element();
    let list_element = use_element();
    // Focus leaving the strip hands the tab stop back to the selected tab.
    // Also on Blitz, whose Tab fires no blur.
    let focus = use_focus_within(
        move || vec![list_element.mounted()],
        move |change| {
            if !change.within {
                let mut focused = focused;
                focused.set(None);
            }
        },
    );
    let keydown_root = root.clone();
    // From the focused tab, which may be a clicked disabled one.
    let onkeydown = use_callback(move |(at, event): (usize, Event<KeyboardData>)| {
        if has_shortcut_modifier(&event) {
            return;
        }
        let next = match logical_key(&event) {
            Key::ArrowRight => neighbour(&disabled, at, 1),
            Key::ArrowLeft => neighbour(&disabled, at, -1),
            Key::Home => disabled.iter().position(|off| !off),
            Key::End => disabled.iter().rposition(|off| !off),
            _ => return,
        };
        let Some(next) = next else {
            return;
        };
        event.prevent_default();
        if !manual {
            onselect.call(next);
        }
        // Blitz fires no focus event for a scripted `focus()`.
        focused.set(Some(next));
        if let Ok(tab) =
            root_element.query_selector(&id_selector(&format!("{keydown_root}-tab-{next}")))
        {
            let _ = tab.focus();
        }
    });

    let states: Input<States> = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("full-width", full_width)
        .into();
    // The text role, shared by the selected label and its underline.
    let variables: Input<Variables> = variables().with(TABS_COLOR, text_color(&color)).into();

    // The caller's name goes on the tablist: `aria-label` is prohibited on the role-less root.
    let (naming, attributes): (Vec<Attribute>, Vec<Attribute>) = attributes
        .into_iter()
        .partition(|attribute| matches!(attribute.name, "aria-label" | "aria-labelledby"));
    let list_id = format!("{root}-tablist");
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
                div {
                    id: "{list_id}",
                    role: "tablist",
                    "aria-orientation": "horizontal",
                    "data-slot": TabsPart::List.slot(),
                    onmounted: list_element.mount(),
                    onfocusout: focus.focusout(0),
                    ..naming,
                    for (index, tab) in tabs.iter().enumerate() {
                        button {
                            key: "{index}",
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
                            onfocus: move |_| focused.set(Some(index)),
                            onclick: {
                                let disabled = tab.disabled;
                                move |_| {
                                    focused.set(Some(index));
                                    if !disabled {
                                        onselect.call(index);
                                    }
                                }
                            },
                            {tab.content.clone()}
                        }
                    }
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
