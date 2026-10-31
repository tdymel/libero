use dioxus::prelude::*;

use super::item::{ItemProps, NotificationItem};
use crate::{
    components::{
        common::{HtmlTag, Input},
        layout::{Box, Float},
    },
    sx::{StaticSx, sx},
    theme::{NOTIFICATION_GAP, NOTIFICATION_OFFSET, NOTIFICATION_WIDTH, Placement},
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
        // Logical margins, not `Float`'s physical offset: they flip with `end` under RTL.
        .when(
            "horizontal-start",
            sx().margin_inline_start(NOTIFICATION_OFFSET.value()),
        )
        .when(
            "horizontal-end",
            sx().margin_inline_end(NOTIFICATION_OFFSET.value()),
        )
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
    z_index: String,
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
            offset_y: vertical_offset(placement),
            z_index: props.z_index,
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
                                    pause_scope: item.pause_scope,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `Float`'s offsets are a translate: negative from the bottom edge.
fn vertical_offset(placement: Placement) -> String {
    use Placement::*;
    match placement {
        TopStart | TopCenter | TopEnd => NOTIFICATION_OFFSET.value(),
        BottomStart | BottomCenter | BottomEnd => {
            format!("calc(-1 * {})", NOTIFICATION_OFFSET.value())
        }
        CenterStart | CenterCenter | CenterEnd => "0px".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 5069: a physical translate pushed an end stack off screen under RTL.
    #[test]
    fn the_inline_offset_is_a_logical_margin() {
        let css = crate::css::Stylesheet::from(&*STACK_SX);
        let css = css.as_str();

        assert!(css.contains("margin-inline-start:"), "{css}");
        assert!(css.contains("margin-inline-end:"), "{css}");
        assert!(
            !css.contains("margin-left") && !css.contains("margin-right"),
            "{css}"
        );
    }
}
