use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{KBD_BACKGROUND, KBD_BORDER, KBD_COLOR, KBD_FONT_FAMILY, Size, SizeCss},
};

// Shared chrome every size level renders identically - only `font-size`
// (and the em-relative padding riding along with it) actually varies.
fn kbd_chrome_sx() -> Sx {
    let border = format!("1px solid {}", KBD_BORDER.value());
    let border_bottom = format!("3px solid {}", KBD_BORDER.value());

    sx().display("inline-block")
        .font_family(KBD_FONT_FAMILY.value())
        .font_weight("700")
        .background(KBD_BACKGROUND.value())
        .color(KBD_COLOR.value())
        .border_top(border.clone())
        .border_left(border.clone())
        .border_right(border)
        // A touch thicker than the other 3 sides - reads as a keycap with
        // some depth instead of a flat pill.
        .border_bottom(border_bottom)
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .padding("0.12em 0.45em")
        .text_align("center")
}

static KBD_XS_SX: StaticSx =
    StaticSx::new(|| kbd_chrome_sx().font_size(SizeCss::KBD_FONT_SIZE.value(Size::Xs)));
static KBD_SM_SX: StaticSx =
    StaticSx::new(|| kbd_chrome_sx().font_size(SizeCss::KBD_FONT_SIZE.value(Size::Sm)));
static KBD_MD_SX: StaticSx =
    StaticSx::new(|| kbd_chrome_sx().font_size(SizeCss::KBD_FONT_SIZE.value(Size::Md)));
static KBD_LG_SX: StaticSx =
    StaticSx::new(|| kbd_chrome_sx().font_size(SizeCss::KBD_FONT_SIZE.value(Size::Lg)));
static KBD_XL_SX: StaticSx =
    StaticSx::new(|| kbd_chrome_sx().font_size(SizeCss::KBD_FONT_SIZE.value(Size::Xl)));

fn get_size_sx(size: &ThemeAwareValue) -> &'static StaticSx {
    match size {
        ThemeAwareValue::Size(s) => match s {
            Size::Xs => &KBD_XS_SX,
            Size::Sm => &KBD_SM_SX,
            Size::Md => &KBD_MD_SX,
            Size::Lg => &KBD_LG_SX,
            Size::Xl => &KBD_XL_SX,
        },
        // Matches Mantine's own default.
        _ => &KBD_SM_SX,
    }
}

base_props! {
    pub struct KbdProps {
        /// Font size - `sm` by default. The rest of the look (background,
        /// border, text color, font family) is theme-only (`Theme::kbd`);
        /// there's no per-instance color prop.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// A single keyboard key, rendered as a real `<kbd>`.
#[component]
pub fn Kbd(props: KbdProps) -> Element {
    let effective_size = props
        .size
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::String("sm".to_string()));
    let size_sx = get_size_sx(&effective_size);

    rsx! {
        Box {
            component: "kbd",
            class: props.class,
            sx: props.sx,
            states: props.states,
            framework_sx: size_sx,
            attributes: props.attributes,
            {props.children}
        }
    }
}
