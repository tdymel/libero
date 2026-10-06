use super::menu::MenuPart;
use crate::{
    components::{
        common::{LogicalTextAlign, Part, disabled_look_sx, inset_focus_ring_sx},
        layout::paper_sx,
    },
    hooks::POPOVER_AVAILABLE_HEIGHT,
    sx::{StaticSx, sx},
    theme::{
        MENU_ITEM_FONT, MENU_ITEM_MIN_HEIGHT, MENU_ITEM_PAD_X, MENU_ITEM_RADIUS, MENU_LABEL_FONT,
        MENU_MAX_HEIGHT, MENU_PADDING, MenuDefaults, Size, SizeCss, Z_INDEX_POPOVER,
    },
};

// `paper_sx()` through `use_box`, for the popover's element and events. Items are
// styled from here, `Tabs`' shape: one class for forty items, not forty `use_box`.
pub(super) static MENU_SX: StaticSx = StaticSx::new(|| {
    let item = MenuPart::Item.selector();
    let glyphs = "& [data-slot='item'] > :is([data-slot='chevron'], [data-slot='check'])";
    paper_sx()
        .and(MenuDefaults::theme_vars())
        .z_index(Z_INDEX_POPOVER.value())
        .display("flex")
        .flex_direction("column")
        .padding(MENU_PADDING)
        // Never past the room on its side, so every item scrolls into view.
        .max_height(format!(
            "min({}, {})",
            MENU_MAX_HEIGHT.value(),
            POPOVER_AVAILABLE_HEIGHT.value_or(MENU_MAX_HEIGHT.value())
        ))
        .overflow_y("auto")
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        .selector(
            "& [role=\"group\"]",
            sx().display("flex").flex_direction("column"),
        )
        .selector(
            MenuPart::GroupLabel.selector(),
            sx().padding(format!("6px {}", MENU_ITEM_PAD_X.value()))
                .font_size(MENU_LABEL_FONT.value())
                .font_weight("600")
                .color("muted.7")
                .user_select("none"),
        )
        .selector(
            item,
            sx().display("flex")
                .align_items("center")
                .gap("10px")
                .width("100%")
                .flex_shrink("0")
                .min_height(MENU_ITEM_MIN_HEIGHT.value())
                .padding(format!("0 {}", MENU_ITEM_PAD_X.value()))
                .box_sizing("border-box")
                .appearance("none")
                .text_decoration("none")
                .border("0")
                .border_radius(MENU_ITEM_RADIUS.value())
                .background("transparent")
                .font("inherit")
                .letter_spacing("inherit")
                .font_size(MENU_ITEM_FONT.value())
                .color("inherit")
                .text_align_start()
                .white_space("nowrap")
                .cursor("pointer")
                .user_select("none"),
        )
        // Hover and focus share a tint: focus follows the pointer.
        .selector(
            format!("{item}:hover:not([aria-disabled=\"true\"])"),
            sx().background("muted.1"),
        )
        .selector(format!("{item}:focus"), sx().background("muted.1"))
        // Inset: the box clips at its padding edge while it scrolls.
        .selector(
            format!("{item}:focus-visible"),
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            format!("{item}[aria-disabled=\"true\"]"),
            disabled_look_sx("not-allowed"),
        )
        // Wraps, never an ellipsis: cut text is unreadable (WCAG 1.4.10).
        .selector(
            MenuPart::Label.selector(),
            sx().flex("1")
                .min_width("0")
                .padding("4px 0")
                .white_space("normal")
                .with("overflow-wrap", "anywhere"),
        )
        .selector(
            "& [data-slot='item'] > :is([data-slot='leading'], [data-slot='trailing'], [data-slot='shortcut'])",
            sx().display("inline-flex").align_items("center"),
        )
        .selector(
            glyphs,
            sx().display("inline-flex")
                .width("1em")
                .height("1em")
                .with("margin-inline-end", "-4px"),
        )
        .selector(format!("{glyphs} svg"), sx().width("100%").height("100%"))
        // The submenu opens on the left under RTL, so the chevron points there.
        .rtl(sx().selector(
            format!("{} svg", MenuPart::Chevron.selector()),
            sx().transform("scaleX(-1)"),
        ))
});
