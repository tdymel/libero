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
    platform::ElementApi,
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

/// The bottom sheet may cover the field (WCAG 2.4.11): once placed over it, the trigger's
/// `scroll-margin-bottom` clears the sheet plus the helper and error under the trigger.
fn use_sheet_clearance(
    anchor: ElementHandle,
    field: Option<ElementHandle>,
    popover: PopoverHandle,
    opened: bool,
    gap: f64,
) -> f64 {
    let mut clearance = use_signal(|| 0.0);
    let placed = popover.placed();
    let sheet = *popover.floating();
    use_effect(use_reactive!(|(opened, placed)| {
        if !(opened && placed) {
            if *clearance.peek() != 0.0 {
                clearance.set(0.0);
            }
            return;
        }
        spawn(async move {
            let (Ok((_, top)), Ok(size), Ok((_, sheet_top)), Ok(sheet_size)) = (
                anchor.client_offset().await,
                anchor.dimensions().await,
                sheet.client_offset().await,
                sheet.dimensions().await,
            ) else {
                return;
            };
            // A placed popover keeps its placed top and covers what a dropdown covers; only
            // the sheet, moved by its `!important`, covers the field.
            if popover
                .placed_top()
                .is_none_or(|placed_top| (placed_top - sheet_top).abs() < 1.0)
            {
                return;
            }
            let trigger_bottom = top + size.height;
            let mut bottom = trigger_bottom;
            if let Some(field) = field
                && let (Ok((_, field_top)), Ok(field_size)) =
                    (field.client_offset().await, field.dimensions().await)
            {
                bottom = bottom.max(field_top + field_size.height);
            }
            if sheet_top < bottom && sheet_top + sheet_size.height > top {
                clearance.set(sheet_size.height + gap + bottom - trigger_bottom);
            }
        });
    }));
    use_effect(move || {
        if clearance() > 0.0 {
            let _ = anchor.scroll_into_view(false);
        }
    });
    clearance()
}

pub(super) struct DropdownSetup<'a> {
    pub(super) state: ComboboxState,
    /// The field's root, so the sheet clears its helper and error too.
    pub(super) field: Option<ElementHandle>,
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
        field,
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

    let sheet_clearance = use_sheet_clearance(anchor, field, popover, opened, gap);

    let search_box = use_box().framework_sx(&CASCADER_SEARCH_SX).prepare();
    let wrapper = use_box()
        // Always printed: a renderer never removes a declaration that stops being printed.
        .style(Some(format!("scroll-margin-bottom:{sheet_clearance}px;")))
        .prepare();
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
