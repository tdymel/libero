use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{
            ClassList, HtmlTag, Input, LogicalTextAlign, OptionLabel, Options, States,
            disabled_look_sx, draw_svg, has_shortcut_modifier, inset_focus_ring_sx,
            use_closing_focus,
        },
        layout::{Collapse, use_box},
    },
    context::IconSlot,
    hooks::{ElementHandle, id_selector, use_element, use_icon},
    platform::ElementApi,
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::{
        ACCORDION_BORDER_COLOR, ACCORDION_CHEVRON_DURATION, ACCORDION_CHEVRON_SIZE,
        ACCORDION_HOVER, ACCORDION_PAD_X, ACCORDION_PAD_Y, AccordionDefaults,
        FOCUS_RING_HALO_SPREAD, Size,
    },
};

static ACCORDION_SX: StaticSx = StaticSx::new(|| {
    AccordionDefaults::theme_vars()
        .display("block")
        // Or a flex parent shrinks it to its longest label.
        .width("100%")
        // One line between sections; no outer frame.
        .selector(
            "& > [data-accordion-item] + [data-accordion-item]",
            sx().border_top(format!("1px solid {}", ACCORDION_BORDER_COLOR.value())),
        )
        // The heading is for the outline; the trigger carries the type.
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading]",
            sx().margin("0").font("inherit"),
        )
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button",
            sx().display("flex")
                .align_items("center")
                .justify_content("space-between")
                .gap(ACCORDION_PAD_X.value())
                .width("100%")
                .appearance("none")
                .background("transparent")
                .border("0")
                .padding(format!("{} {}", ACCORDION_PAD_Y.value(), ACCORDION_PAD_X.value()))
                .font("inherit")
                .letter_spacing("inherit")
                .color("inherit")
                .text_align_start()
                .cursor("pointer"),
        )
        // A long word wraps instead of pushing the chevron off-screen (1.4.10).
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button > span:not([data-accordion-chevron])",
            sx().min_width("0").with("overflow-wrap", "anywhere"),
        )
        // Inset: a full-width outset ring gets clipped by the container.
        // Doubled to outrank a `Button`'s ring whatever the stylesheet order.
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button:focus-visible:focus-visible",
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button:hover:not([aria-disabled=\"true\"])",
            sx().background(ACCORDION_HOVER.value()),
        )
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button[aria-disabled=\"true\"]",
            disabled_look_sx("not-allowed"),
        )
        // The reduced-motion guard nests here, at the same specificity, or it loses.
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] [data-accordion-chevron]",
            sx().display("inline-flex")
                .flex_shrink("0")
                .width(ACCORDION_CHEVRON_SIZE.value())
                .height(ACCORDION_CHEVRON_SIZE.value())
                .transition(format!(
                    "transform {} ease",
                    ACCORDION_CHEVRON_DURATION.value()
                ))
                .selector("& > svg", sx().width("100%").height("100%"))
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button[aria-expanded=\"true\"] [data-accordion-chevron]",
            sx().transform("rotate(180deg)"),
        )
        // Inside `Collapse`'s clipped box, so the padding animates with the height. The
        // top pad keeps a first child's ring clear of the clip; wide content wraps or scrolls (1.4.10).
        .selector(
            "& > [data-accordion-item] [data-accordion-body]",
            sx().padding(format!(
                "{} {} {}",
                FOCUS_RING_HALO_SPREAD.value(),
                ACCORDION_PAD_X.value(),
                ACCORDION_PAD_Y.value()
            ))
            .with("overflow-wrap", "anywhere")
            .overflow_x("auto"),
        )
});

pub(crate) struct SectionSpec<T> {
    pub value: T,
    pub label: OptionLabel,
    pub disabled: bool,
    pub open: bool,
}

pub(crate) struct AccordionView<T: Options> {
    pub sections: Vec<SectionSpec<T>>,
    pub panel: Option<Callback<T, Element>>,
    pub ontoggle: Callback<usize>,
    /// `h1`..`h6`, already checked.
    pub heading: HtmlTag,
    pub size: Size,
    pub class: Input<ClassList>,
    pub sx: Input<crate::sx::Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// Arrow targets from one trigger: next, previous (wrapping), first, last enabled.
/// `None` with nothing enabled.
fn arrow_targets(disabled: &[bool], from: usize) -> Option<[usize; 4]> {
    let enabled: Vec<usize> = (0..disabled.len()).filter(|i| !disabled[*i]).collect();
    let first = *enabled.first()?;
    let last = *enabled.last()?;
    let next = enabled.iter().copied().find(|i| *i > from).unwrap_or(first);
    let previous = enabled.iter().copied().rfind(|i| *i < from).unwrap_or(last);
    Some([next, previous, first, last])
}

fn heading(level: HtmlTag, trigger: Element) -> Element {
    match level {
        HtmlTag::H1 => rsx! { h1 { "data-accordion-heading": "", {trigger} } },
        HtmlTag::H2 => rsx! { h2 { "data-accordion-heading": "", {trigger} } },
        HtmlTag::H4 => rsx! { h4 { "data-accordion-heading": "", {trigger} } },
        HtmlTag::H5 => rsx! { h5 { "data-accordion-heading": "", {trigger} } },
        HtmlTag::H6 => rsx! { h6 { "data-accordion-heading": "", {trigger} } },
        _ => rsx! { h3 { "data-accordion-heading": "", {trigger} } },
    }
}

/// A plain `fn`, run in `Accordion`'s scope; each section is a scope of its own.
pub(crate) fn render_accordion<T: Options>(view: AccordionView<T>, root: String) -> Element {
    let AccordionView {
        sections,
        panel,
        ontoggle,
        heading: level,
        size,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let root_element = use_element();

    // A panel closing with focus inside returns it to its trigger. Checked in
    // render: by the effect, a zero-duration panel is gone.
    let open: Vec<usize> = (0..sections.len()).filter(|i| sections[*i].open).collect();
    let previous = use_hook(|| Rc::new(RefCell::new(open.clone())));
    let closing = use_closing_focus(root_element);
    {
        let mut previous = previous.borrow_mut();
        if *previous != open {
            let _ = previous.iter().filter(|i| !open.contains(i)).any(|i| {
                closing.closing(
                    id_selector(&format!("{root}-region-{i}")),
                    id_selector(&format!("{root}-trigger-{i}")),
                )
            });
            *previous = open.clone();
        }
    }
    let repay = closing.clone();
    use_effect(use_reactive!(|(open,)| {
        let _ = open;
        repay.repay();
    }));

    let states: Input<States> = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();

    let disabled: Vec<bool> = sections.iter().map(|section| section.disabled).collect();

    let items = rsx! {
        for (index, section) in sections.into_iter().enumerate() {
            AccordionSection {
                key: "{index}",
                value: section.value,
                label: section.label,
                index,
                open: section.open,
                disabled: section.disabled,
                targets: arrow_targets(&disabled, index),
                level,
                root: root.clone(),
                root_element,
                ontoggle,
                panel,
            }
        }
    };

    use_box()
        .framework_sx(&ACCORDION_SX)
        .class(&class)
        .sx(&user_sx)
        .states(&states)
        .prepare()
        .element(&root_element)
        .render(HtmlTag::Div, attributes, items)
}

/// One section, its own scope: a switch redraws the two sections it changes, and a
/// panel is built only when its section redraws.
#[component]
fn AccordionSection<T: Options>(
    value: T,
    label: OptionLabel,
    index: usize,
    open: bool,
    disabled: bool,
    /// From [`arrow_targets`].
    targets: Option<[usize; 4]>,
    level: HtmlTag,
    root: String,
    root_element: ElementHandle,
    ontoggle: Callback<usize>,
    panel: Option<Callback<T, Element>>,
) -> Element {
    let trigger_id = format!("{root}-trigger-{index}");
    let region_id = format!("{root}-region-{index}");
    let name = label.name;
    let content = label.content.unwrap_or_else(|| rsx! { "{name}" });
    // Drawn in this scope, not by a `Glyph` scope per section: same `<svg>`, 50 scopes fewer at 50.
    let chevron = draw_svg(
        &use_icon(IconSlot::ChevronDown, lucide::chevron_down::outlined),
        Vec::new(),
    );
    // Built while open; a closing panel animates out with its last body, and
    // `Collapse` drops it once closed.
    let kept = use_hook(|| Rc::new(RefCell::new(None::<Element>)));
    let body = match (&panel, open) {
        (Some(panel), true) => {
            let body = panel.call(value.clone());
            *kept.borrow_mut() = Some(body.clone());
            body
        }
        (None, true) => rsx! {},
        (_, false) => kept.borrow().clone().unwrap_or_else(|| rsx! {}),
    };
    // A plain closure, not `use_callback`: a focus handler is re-entrant.
    let onkeydown = move |event: Event<KeyboardData>| {
        // A lone enabled trigger has nowhere to go: the keys scroll the page.
        let Some([next, previous, first, last]) = targets.filter(|[next, ..]| *next != index)
        else {
            return;
        };
        // Ctrl/Alt/Meta chords are the browser's.
        if has_shortcut_modifier(&event) {
            return;
        }
        // Arrows move focus and never toggle - the opposite of `Tabs`.
        let to = match event.key() {
            Key::ArrowDown => next,
            Key::ArrowUp => previous,
            Key::Home => first,
            Key::End => last,
            _ => return,
        };
        // Home on the first or End on the last trigger scrolls the page.
        if to == index {
            return;
        }
        event.prevent_default();
        let _ = root_element
            .query_selector(&id_selector(&format!("{root}-trigger-{to}")))
            .and_then(|trigger| trigger.focus());
    };
    let trigger = rsx! {
        button {
            id: "{trigger_id}",
            r#type: "button",
            // Strings: SSR writes a bare `true`, and the chevron CSS matches `"true"`.
            "aria-expanded": if open { "true" } else { "false" },
            "aria-controls": "{region_id}",
            // `aria-disabled`, not `disabled`: the section stays a tab stop.
            "aria-disabled": if disabled { "true" } else { "false" },
            "aria-label": name,
            onclick: move |_| {
                if !disabled {
                    ontoggle.call(index);
                }
            },
            onkeydown,
            span { {content} }
            span { "data-accordion-chevron": "", {chevron} }
        }
    };
    rsx! {
        div { "data-accordion-item": "",
            {heading(level, trigger)}
            Collapse {
                open,
                keep_mounted: false,
                id: "{region_id}",
                role: "region",
                aria_labelledby: "{trigger_id}",
                div { "data-accordion-body": "", {body} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::arrow_targets;

    #[test]
    fn arrows_wrap_over_the_enabled_triggers() {
        let disabled = [false, false, false];
        assert_eq!(arrow_targets(&disabled, 0), Some([1, 2, 0, 2]));
        assert_eq!(arrow_targets(&disabled, 2), Some([0, 1, 0, 2]));
    }

    #[test]
    fn arrows_skip_a_disabled_trigger() {
        let disabled = [false, true, false];
        assert_eq!(arrow_targets(&disabled, 0), Some([2, 2, 0, 2]));
    }

    /// A disabled trigger is a tab stop; the walk leaves it for its enabled neighbours.
    #[test]
    fn arrows_leave_a_focused_disabled_trigger() {
        let disabled = [false, true, false];
        assert_eq!(arrow_targets(&disabled, 1), Some([2, 0, 0, 2]));
    }

    #[test]
    fn nothing_enabled_moves_nowhere() {
        assert_eq!(arrow_targets(&[true, true], 0), None);
    }
}
