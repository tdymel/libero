use dioxus::prelude::*;

use super::item::{ItemProps, NotificationItem};
use crate::{
    components::{
        common::{HtmlTag, Input},
        layout::{Box, Float},
    },
    sx::{StaticSx, sx},
    theme::{
        NOTIFICATION_GAP, NOTIFICATION_OFFSET, NOTIFICATION_WIDTH, Placement, Z_INDEX_NOTIFICATION,
    },
};

/// One stack. It lets the pointer through: only its notifications take clicks.
static STACK_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .width(format!(
            "min({}, calc(100% - 2 * {}))",
            NOTIFICATION_WIDTH.value(),
            NOTIFICATION_OFFSET.value()
        ))
        .pointer_events("none")
});

/// One live region, always rendered. Spaced here, not by a `gap`, so an empty
/// one takes no space.
static REGION_SX: StaticSx = StaticSx::new(|| {
    sx().selector(
        "&:not(:empty) + &:not(:empty)",
        sx().margin_top(NOTIFICATION_GAP.value()),
    )
});

static LIST_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(NOTIFICATION_GAP.value())
        .margin("0")
        .padding("0")
        .list_style("none")
});

#[derive(Props, Clone, PartialEq)]
pub(super) struct StackProps {
    placement: Placement,
    fixed: bool,
    assertive: Vec<ItemProps>,
    polite: Vec<ItemProps>,
}

/// Its own scope: a list write redraws only the stack it changed.
pub(super) fn NotificationStack(props: StackProps) -> Element {
    let placement = props.placement;
    let regions = [("assertive", props.assertive), ("polite", props.polite)];

    rsx! {
        Float {
            fixed: props.fixed,
            placement: Input::Value(placement),
            offset_x: edge_offset(placement, Axis::Horizontal),
            offset_y: edge_offset(placement, Axis::Vertical),
            z_index: Z_INDEX_NOTIFICATION.value(),
            sx: &STACK_SX,
            for (live, items) in regions {
                Box {
                    key: "{live}",
                    framework_sx: &REGION_SX,
                    "aria-live": live,
                    if !items.is_empty() {
                        Box {
                            component: HtmlTag::Ol,
                            framework_sx: &LIST_SX,
                            // Both: Safari with VoiceOver drops list
                            // semantics from a `list-style: none` list.
                            role: "list",
                            for item in items {
                                NotificationItem {
                                    key: "{item.id.0}",
                                    store: item.store,
                                    id: item.id,
                                    draw: item.draw,
                                    auto_close: item.auto_close,
                                    leaving: item.leaving,
                                    exit_ms: item.exit_ms,
                                    live: item.live,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum Axis {
    Horizontal,
    Vertical,
}

/// `Float`'s offsets are a translate: negative from an end or bottom edge.
fn edge_offset(placement: Placement, axis: Axis) -> String {
    use Placement::*;
    let from_start = match axis {
        Axis::Horizontal => match placement {
            TopStart | CenterStart | BottomStart => Some(true),
            TopEnd | CenterEnd | BottomEnd => Some(false),
            TopCenter | CenterCenter | BottomCenter => None,
        },
        Axis::Vertical => match placement {
            TopStart | TopCenter | TopEnd => Some(true),
            BottomStart | BottomCenter | BottomEnd => Some(false),
            CenterStart | CenterCenter | CenterEnd => None,
        },
    };
    match from_start {
        Some(true) => NOTIFICATION_OFFSET.value(),
        Some(false) => format!("calc(-1 * {})", NOTIFICATION_OFFSET.value()),
        None => "0px".to_string(),
    }
}
