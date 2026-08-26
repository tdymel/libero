use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{Variables, variables},
        layout::use_box,
    },
    hooks::use_element,
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, Size, SizeCss, TABS_BORDER_COLOR, TABS_GAP, TABS_HOVER, TABS_LINE, TABS_PAD_X,
        TABS_PAD_Y, TabsDefaults,
    },
};

const TABS_COLOR: CssVar = CssVar::new("--lsx-tabs-color");

static TABS_SX: StaticSx = StaticSx::new(|| {
    let line = format!("1px solid {}", TABS_BORDER_COLOR.value());

    TabsDefaults::theme_vars()
        .display("block")
        .selector(
            "& > [role=\"tablist\"]",
            sx().display("flex")
                .align_items("stretch")
                .border_bottom(line),
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
                // The strip's own 1px line sits under the indicator, so the
                // selected tab's border has to overlap it rather than stack.
                .margin_bottom("-1px")
                .padding(format!("{} {}", TABS_PAD_Y.value(), TABS_PAD_X.value()))
                .font("inherit")
                .color("inherit")
                .white_space("nowrap")
                .cursor("pointer"),
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
        .selector(
            "& [role=\"tab\"][aria-disabled=\"true\"]",
            sx().opacity("0.5").cursor("not-allowed"),
        )
        .selector(
            "& > [role=\"tabpanel\"]",
            sx().padding_top(SizeCss::SPACING.value(Size::Md)),
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
        size,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let enabled: Vec<usize> = (0..tabs.len()).filter(|i| !tabs[*i].disabled).collect();
    let root_element = use_element();
    let keydown_root = root.clone();
    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        if enabled.is_empty() {
            return;
        }
        let at = selected
            .and_then(|selected| enabled.iter().position(|tab| *tab == selected))
            .unwrap_or(0);
        let last = enabled.len() - 1;
        let next = match event.key() {
            Key::ArrowRight => enabled[if at == last { 0 } else { at + 1 }],
            Key::ArrowLeft => enabled[if at == 0 { last } else { at - 1 }],
            Key::Home => enabled[0],
            Key::End => enabled[last],
            _ => return,
        };
        event.prevent_default();
        onselect.call(next);
        // Auto activation: focus follows the selection, so the strip behaves
        // like one control rather than a row of buttons.
        if let Ok(tab) = root_element.query_selector(&format!("#{keydown_root}-tab-{next}")) {
            let _ = tab.focus();
        }
    });

    let states: Input<States> = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("full-width", full_width)
        .into();
    let variables: Input<Variables> = variables().with(TABS_COLOR, color.resolve(None)).into();

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
                    onkeydown: move |event| onkeydown.call(event),
                    for (index, tab) in tabs.iter().enumerate() {
                        button {
                            key: "{index}",
                            id: "{root}-tab-{index}",
                            r#type: "button",
                            role: "tab",
                            // Strings, not bools: SSR writes a bare `true`, and the
                            // selected-tab CSS matches on `[aria-selected="true"]`.
                            "aria-selected": if selected == Some(index) { "true" } else { "false" },
                            "aria-controls": "{root}-panel-{index}",
                            // `aria-disabled`, not `disabled`: a disabled tab
                            // stays reachable, it just cannot be picked.
                            "aria-disabled": if tab.disabled { "true" } else { "false" },
                            "aria-label": tab.name.clone(),
                            tabindex: if selected == Some(index) { "0" } else { "-1" },
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
