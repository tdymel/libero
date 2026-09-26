use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{ComboboxState, HtmlTag, Input, Part, Parts, States},
        form::{DropdownPart, combobox::COMBOBOX_DROPDOWN_SX},
        layout::{BoxStyle, use_box},
    },
    hooks::{
        ElementHandle, PopoverHandle, PopoverOptions, PopoverWidth, use_element,
        use_field_list_layer, use_popover_on,
    },
    sx::{StaticSx, sx},
    theme::Size,
};

use super::{
    core::{CascaderLayout, narrow_query},
    keys::CascaderKeys,
    search::{CASCADER_SEARCH_SX, use_focus_search},
};

/// The dropdown. Narrow, a bottom sheet: full width at the screen's foot. `!important`
/// beats the popover's inline position, which keeps measuring but no longer places.
pub(super) static CASCADER_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    COMBOBOX_DROPDOWN_SX.clone().media(
        narrow_query(),
        sx().left("0 !important")
            .right("0 !important")
            .top("auto !important")
            .bottom("0 !important")
            .width("100% !important")
            .min_width("0 !important")
            .max_width("100% !important")
            .max_height("70vh")
            .border_bottom_left_radius("0")
            .border_bottom_right_radius("0")
            .padding_bottom("calc(4px + env(safe-area-inset-bottom, 0px))")
            // Tracks the visible viewport as a phone's URL bar slides in and out.
            .supports("(height: 1dvh)", sx().max_height("70dvh")),
    )
});

/// The open list's box, holding the search box and the rows.
pub(super) fn dropdown_box(
    style: BoxStyle,
    floating: &ElementHandle,
    keys: Rc<CascaderKeys>,
    content: Element,
) -> Element {
    style
        .element(floating)
        // A click on the padding or scrollbar must not blur the trigger, which would close the list.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default()
        })
        // Portaled, so keys inside would bubble to `PortalOutlet`, not the trigger.
        .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
        .attr("data-slot", DropdownPart::Panel.slot())
        .render(HtmlTag::Div, Vec::new(), content)
}

pub(super) struct DropdownSetup<'a> {
    pub(super) state: ComboboxState,
    pub(super) opened: bool,
    pub(super) searchable: bool,
    pub(super) search: ElementHandle,
    pub(super) size: Size,
    pub(super) radius: Size,
    pub(super) layout: CascaderLayout,
    pub(super) remeasure: u64,
    pub(super) gap: f64,
    pub(super) padding: f64,
    pub(super) parts: &'a Input<Parts<DropdownPart>>,
}

pub(super) struct Dropdown {
    pub(super) anchor: ElementHandle,
    pub(super) popover: PopoverHandle,
    pub(super) search_box: BoxStyle,
    pub(super) wrapper: BoxStyle,
    pub(super) dropdown: BoxStyle,
}

pub(super) fn use_cascader_dropdown(setup: DropdownSetup) -> Dropdown {
    let DropdownSetup {
        state,
        opened,
        searchable,
        search,
        size,
        radius,
        layout,
        remeasure,
        gap,
        padding,
        parts,
    } = setup;

    // On the Escape stack while open, so a surrounding `HoverCard` leaves the press to it.
    use_field_list_layer(opened, use_callback(move |()| state.close()));
    let anchor = use_element();
    let popover = use_popover_on(
        anchor,
        use_element(),
        opened,
        PopoverOptions::new(gap, padding)
            // Columns and joined paths run wider than the trigger.
            .width(PopoverWidth::Min)
            .remeasure(remeasure),
    );

    let search_box = use_box().framework_sx(&CASCADER_SEARCH_SX).prepare();
    let wrapper = use_box().prepare();
    let dropdown_states: Input<States> = States::new()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("bordered", true)
        .with(layout.state_name(), true)
        .into();
    let dropdown = use_box()
        // `use_popover` caps it to the viewport; wide columns scroll inside instead of off-screen.
        .framework_sx(&CASCADER_DROPDOWN_SX)
        .parts(parts)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();

    use_focus_search(opened, searchable, popover.placed(), search);

    Dropdown {
        anchor,
        popover,
        search_box,
        wrapper,
        dropdown,
    }
}
