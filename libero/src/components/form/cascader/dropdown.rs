use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{ClassList, ComboboxState, HtmlTag, Input, Part, Parts, States},
        form::{
            DropdownPart,
            combobox::COMBOBOX_DROPDOWN_SX,
            phone_sheet::{
                phone_sheet_sx, sheet_clearance_style, use_sheet_clearance, use_sheet_room,
            },
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{
        ElementHandle, PopoverHandle, PopoverOptions, PopoverWidth, use_element,
        use_field_list_layer, use_popover_on,
    },
    sx::{StaticSx, Sx},
    theme::Size,
};

use super::{
    core::CascaderLayout,
    keys::CascaderKeys,
    search::{CASCADER_SEARCH_SX, use_focus_search},
};

/// The dropdown; narrow, the shared phone sheet.
pub(super) static CASCADER_DROPDOWN_SX: StaticSx =
    StaticSx::new(|| COMBOBOX_DROPDOWN_SX.clone().and(phone_sheet_sx()));

/// The open list's box, holding the search box and the rows.
pub(super) fn dropdown_box(
    style: BoxStyle,
    floating: &ElementHandle,
    keys: Rc<CascaderKeys>,
    attributes: Vec<Attribute>,
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
        .render(HtmlTag::Div, attributes, content)
}

pub(super) struct DropdownSetup<'a> {
    pub(super) state: ComboboxState,
    /// What the popover hangs from: the field's wrapper, or a custom trigger.
    pub(super) anchor: ElementHandle,
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
    /// `CascaderDropdown`'s own `class` and `sx`; the field styles its box by `dropdown_parts` only.
    pub(super) class: Option<&'a Input<ClassList>>,
    pub(super) sx: Option<&'a Input<Sx>>,
}

pub(super) struct Dropdown {
    pub(super) popover: PopoverHandle,
    pub(super) search_box: BoxStyle,
    pub(super) wrapper: BoxStyle,
    pub(super) dropdown: BoxStyle,
    pub(super) clearance: f64,
}

pub(super) fn use_cascader_dropdown(setup: DropdownSetup) -> Dropdown {
    let DropdownSetup {
        state,
        anchor,
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
        class,
        sx: own_sx,
    } = setup;

    // On the Escape stack while open, so a surrounding `HoverCard` leaves the press to it.
    use_field_list_layer(opened, use_callback(move |()| state.close()));
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
    let room = use_sheet_room(*popover.floating(), opened && popover.placed());

    let search_box = use_box().framework_sx(&CASCADER_SEARCH_SX).prepare();
    let wrapper = use_box()
        // Always printed: a renderer never removes a declaration that stops being printed.
        .style(Some(sheet_clearance_style(sheet_clearance)))
        .prepare();
    let dropdown_states: Input<States> = States::new()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("bordered", true)
        .with(layout.state_name(), true)
        .into();
    let mut dropdown = use_box()
        // `use_popover` caps it to the viewport; wide columns scroll inside instead of off-screen.
        .framework_sx(&CASCADER_DROPDOWN_SX)
        .parts(parts)
        .states(&dropdown_states)
        // Vars, not inline placement: `COMBOBOX_DROPDOWN_SX` places by class, so `sx` can move it.
        .style(Some(format!("{}{room}", popover.vars())));
    if let Some(class) = class {
        dropdown = dropdown.class(class);
    }
    if let Some(own_sx) = own_sx {
        dropdown = dropdown.sx(own_sx);
    }
    let dropdown = dropdown.prepare();

    use_focus_search(opened, searchable, popover.placed(), search);

    Dropdown {
        popover,
        search_box,
        wrapper,
        dropdown,
        clearance: sheet_clearance,
    }
}
