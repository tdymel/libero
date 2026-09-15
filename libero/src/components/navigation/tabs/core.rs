use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{
            Variables, focus_ring_sx, has_shortcut_modifier, inset_focus_ring_sx, neighbour,
            text_color, variables,
        },
        layout::use_box,
    },
    hooks::{id_selector, use_element},
    platform::ElementApi,
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
            // Scrolls inside itself, so a crowded strip does not widen the
            // page (WCAG 1.4.10). The line is an inset shadow: a border
            // would clip the selected tab's underline where they overlap.
            sx().display("flex")
                .align_items("stretch")
                .overflow_x("auto")
                .box_shadow(line),
        )
        .selector(
            "& [role=\"tab\"]",
            // A flex line, so a rich label (an icon beside the text) sits on
            // the text's centre rather than its baseline.
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
                // Sized to the whole label so a crowded strip scrolls, but never
                // wider than the strip: a longer label wraps (1.4.10).
                .flex_shrink("0")
                .max_width("100%")
                .with("overflow-wrap", "anywhere")
                .cursor("pointer"),
        )
        // `appearance: none` and `border: 0` above take the UA's own focus
        // ring with them, so the tab has to draw one or keyboard users cannot
        // see where they are. Inset, because the strip's line sits flush
        // against the tab's bottom edge and an outset ring would be clipped.
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
        // Forced colours paint every transparent underline, so each tab would
        // look selected; system colours keep only the selected one's.
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
        // The panel is a tab stop too, and without this it gets the UA's
        // outline rather than the house ring. Doubled to outrank a component's
        // own ring whatever the stylesheet order, as `Carousel`.
        .selector(
            "& > [role=\"tabpanel\"]:focus-visible:focus-visible",
            focus_ring_sx(),
        )
        .when(
            "full-width",
            // The root pins its own width, or a flex parent shrinks it to the
            // tabs and there is nothing for them to share.
            sx().width("100%")
                .selector("& [role=\"tab\"]", sx().flex("1 1 0")),
        )
});

/// One tab, with `T` already gone: `content` is the rendered label and `name`
/// the accessible one.
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
    pub size: Size,
    pub class: Input<ClassList>,
    pub sx: Input<crate::sx::Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// A plain `fn`, not a component: `Vec<Element>` props defeat memoization, so
/// a scope here would cost a scope and buy nothing.
pub(crate) fn render_tabs(view: TabsView, root: String) -> Element {
    let TabsView {
        tabs,
        selected,
        panel,
        onselect,
        color,
        full_width,
        manual,
        size,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let disabled: Vec<bool> = tabs.iter().map(|tab| tab.disabled).collect();
    // One tab stop for the strip: the selected tab, or the first enabled one
    // when `value` is not among the tabs, so the strip stays reachable.
    let tab_stop = selected.or_else(|| disabled.iter().position(|off| !off));
    let root_element = use_element();
    let keydown_root = root.clone();
    // Steps from the focused tab: a click focuses a disabled tab without
    // selecting it.
    let onkeydown = use_callback(move |(at, event): (usize, Event<KeyboardData>)| {
        if has_shortcut_modifier(&event) {
            return;
        }
        let next = match event.key() {
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
        // Auto activation: the selection follows the focus, so the strip
        // behaves like one control rather than a row of buttons.
        if !manual {
            onselect.call(next);
        }
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
    // The selected tab's own label, so the text role - the underline under
    // it takes the same colour rather than a second one.
    let variables: Input<Variables> = variables().with(TABS_COLOR, text_color(&color)).into();

    // The caller's name belongs on the tablist: `aria-label` is prohibited on
    // the role-less root (APG).
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
                            // Only the selected panel is rendered, and an id
                            // that is not in the document is an invalid ref.
                            "aria-controls": (selected == Some(index)).then(|| format!("{root}-panel-{index}")),
                            // `aria-disabled`, not `disabled`: a disabled tab
                            // stays reachable, it just cannot be picked.
                            "aria-disabled": if tab.disabled { "true" } else { "false" },
                            "aria-label": tab.name.clone(),
                            tabindex: if tab_stop == Some(index) { "0" } else { "-1" },
                            onkeydown: move |event| onkeydown.call((index, event)),
                            onclick: {
                                let disabled = tab.disabled;
                                move |_| {
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
                        tabindex: "0",
                        {panel}
                    }
                }
            },
        )
}
