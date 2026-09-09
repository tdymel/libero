use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        common::{ChevronDownIcon, inset_focus_ring_sx},
        layout::{Collapse, use_box},
    },
    hooks::{id_selector, use_element},
    platform::ElementApi,
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::{
        ACCORDION_BORDER_COLOR, ACCORDION_CHEVRON_DURATION, ACCORDION_CHEVRON_SIZE,
        ACCORDION_HOVER, ACCORDION_PAD_X, ACCORDION_PAD_Y, AccordionDefaults, Size,
    },
};

static ACCORDION_SX: StaticSx = StaticSx::new(|| {
    AccordionDefaults::theme_vars()
        .display("block")
        // A list fills the column it sits in; in a flex parent it would
        // otherwise shrink to its longest label.
        .width("100%")
        // One line between two sections - no outer frame, no radius. The list
        // is flat; a surface around it is the caller's to add.
        .selector(
            "& > [data-accordion-item] + [data-accordion-item]",
            sx().border_top(format!("1px solid {}", ACCORDION_BORDER_COLOR.value())),
        )
        // The heading is there for the document outline, not for its looks:
        // the trigger inside it carries the type.
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
                .color("inherit")
                .text_align("start")
                .cursor("pointer"),
        )
        // `appearance: none` and `border: 0` take the UA ring with them. Inset,
        // because the trigger spans the full width and an outset ring would be
        // clipped by whatever holds the accordion.
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button:focus-visible",
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button:hover:not([aria-disabled=\"true\"])",
            sx().background(ACCORDION_HOVER.value()),
        )
        .selector(
            "& > [data-accordion-item] > [data-accordion-heading] > button[aria-disabled=\"true\"]",
            sx().opacity("0.5").cursor("not-allowed"),
        )
        // The transition is declared here, so its reduced-motion guard is
        // nested here too - at the same specificity, or it loses.
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
        // Inside `Collapse`'s clipped box, so the padding animates with the
        // height rather than standing outside it.
        .selector(
            "& > [data-accordion-item] [data-accordion-body]",
            sx().padding(format!("0 {} {}", ACCORDION_PAD_X.value(), ACCORDION_PAD_Y.value())),
        )
});

/// One section, with `T` already gone: `content` is the rendered label and
/// `name` the accessible one.
pub(crate) struct SectionSpec {
    pub name: String,
    pub content: Element,
    pub disabled: bool,
    pub open: bool,
    pub panel: Element,
}

pub(crate) struct AccordionView {
    pub sections: Vec<SectionSpec>,
    pub ontoggle: Callback<usize>,
    /// `h1`..`h6`, already checked.
    pub heading: HtmlTag,
    pub size: Size,
    pub class: Input<ClassList>,
    pub sx: Input<crate::sx::Sx>,
    pub states: Input<States>,
    pub attributes: Vec<Attribute>,
}

/// Where each arrow key sends focus from one trigger: the next and previous
/// enabled trigger, wrapping, and the first and last. `None` with nothing
/// enabled. A disabled trigger is still a tab stop, so it has neighbours too.
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

/// A plain `fn`, not a component: `Vec<Element>` props defeat memoization, so
/// a scope here would cost a scope and buy nothing.
pub(crate) fn render_accordion(view: AccordionView, root: String) -> Element {
    let AccordionView {
        sections,
        ontoggle,
        heading: level,
        size,
        class,
        sx: user_sx,
        states,
        attributes,
    } = view;

    let root_element = use_element();

    // Focus return. A panel can close while focus is inside it - a "Continue"
    // button that opens the next step closes this one in `One` mode - and then
    // `visibility: hidden` or the unmount drops focus to `<body>`. `Collapse`
    // never sees the trigger, so the repair is here: the closed panel's own
    // trigger takes focus back.
    //
    // Checked during render, against the DOM the previous render left: by the
    // time an effect runs, a zero-duration panel is already gone and nothing
    // can say where focus was. The move itself waits for the effect.
    let open: Vec<usize> = (0..sections.len()).filter(|i| sections[*i].open).collect();
    let previous = use_hook(|| Rc::new(RefCell::new(open.clone())));
    let owed = use_hook(|| Rc::new(RefCell::new(None::<usize>)));
    {
        let mut previous = previous.borrow_mut();
        if *previous != open {
            let closed = previous.iter().find(|i| {
                !open.contains(i)
                    && root_element
                        .query_selector(&format!(
                            "{} :focus",
                            id_selector(&format!("{root}-region-{i}"))
                        ))
                        .is_ok()
            });
            if let Some(closed) = closed {
                *owed.borrow_mut() = Some(*closed);
            }
            *previous = open.clone();
        }
    }
    let focus_root = root.clone();
    let focus_owed = owed.clone();
    use_effect(use_reactive!(|(open,)| {
        let _ = open;
        if let Some(index) = focus_owed.borrow_mut().take() {
            let _ = root_element
                .query_selector(&id_selector(&format!("{focus_root}-trigger-{index}")))
                .and_then(|trigger| trigger.focus());
        }
    }));

    let states: Input<States> = states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();

    let disabled: Vec<bool> = sections.iter().map(|section| section.disabled).collect();

    let items = sections.into_iter().enumerate().map(|(index, section)| {
        let trigger_id = format!("{root}-trigger-{index}");
        let region_id = format!("{root}-region-{index}");
        let targets = arrow_targets(&disabled, index);
        let key_root = root.clone();
        // A plain closure, not `use_callback`: it moves focus, and a focus
        // handler is re-entrant.
        let onkeydown = move |event: Event<KeyboardData>| {
            let Some([next, previous, first, last]) = targets else {
                return;
            };
            // Arrows move focus and never toggle - the opposite of `Tabs`.
            let to = match event.key() {
                Key::ArrowDown => next,
                Key::ArrowUp => previous,
                Key::Home => first,
                Key::End => last,
                _ => return,
            };
            event.prevent_default();
            let _ = root_element
                .query_selector(&id_selector(&format!("{key_root}-trigger-{to}")))
                .and_then(|trigger| trigger.focus());
        };
        let disabled = section.disabled;
        let trigger = rsx! {
            button {
                id: "{trigger_id}",
                r#type: "button",
                // Strings, not bools: SSR writes a bare `true`, and the
                // chevron's CSS matches on `[aria-expanded="true"]`.
                "aria-expanded": if section.open { "true" } else { "false" },
                "aria-controls": "{region_id}",
                // `aria-disabled`, not `disabled`: the section stays a tab stop
                // and still reads, it just cannot be toggled.
                "aria-disabled": if disabled { "true" } else { "false" },
                "aria-label": section.name,
                onclick: move |_| {
                    if !disabled {
                        ontoggle.call(index);
                    }
                },
                onkeydown,
                span { {section.content} }
                span { "data-accordion-chevron": "", ChevronDownIcon {} }
            }
        };
        rsx! {
            div { key: "{index}", "data-accordion-item": "",
                {heading(level, trigger)}
                Collapse {
                    open: section.open,
                    keep_mounted: false,
                    id: "{region_id}",
                    // A landmark only while open: the root stays mounted closed.
                    role: section.open.then_some("region"),
                    aria_labelledby: section.open.then(|| trigger_id.clone()),
                    div { "data-accordion-body": "", {section.panel} }
                }
            }
        }
    });
    let items: Vec<Element> = items.collect();

    use_box()
        .framework_sx(&ACCORDION_SX)
        .class(&class)
        .sx(&user_sx)
        .states(&states)
        .prepare()
        .element(&root_element)
        .render(HtmlTag::Div, attributes, items)
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

    /// Focus can sit on a disabled trigger - it is a tab stop - and the walk
    /// still leaves it for its enabled neighbours.
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
