use dioxus::prelude::*;

use super::item::{ItemProps, NotificationItem};
use crate::{
    components::{
        common::{HtmlTag, Input},
        layout::{Box, FLOAT_SAFE_AREA_VAR, Float},
    },
    sx::{StaticSx, sx},
    theme::{CssVar, NOTIFICATION_GAP, NOTIFICATION_OFFSET, NOTIFICATION_WIDTH, Placement},
};

const NAVIGATION_BAR_HEIGHT_VAR: CssVar = CssVar::new("--lsx-navigation-bar-height");
// The viewport's insets on the stack's inline sides; zero in a contained host (5442).
const INSET_START_VAR: CssVar = CssVar::new("--lsx-notification-inset-start");
const INSET_END_VAR: CssVar = CssVar::new("--lsx-notification-inset-end");

/// One stack. It lets the pointer through: only its notifications take clicks.
static STACK_SX: StaticSx = StaticSx::new(|| {
    let offset = NOTIFICATION_OFFSET.value();
    let (start, end) = (
        INSET_START_VAR.value_or("0px"),
        INSET_END_VAR.value_or("0px"),
    );
    let insets = |start: &str, end: &str| {
        sx().var(
            INSET_START_VAR,
            format!("env(safe-area-inset-{start}, 0px)"),
        )
        .var(INSET_END_VAR, format!("env(safe-area-inset-{end}, 0px)"))
    };
    // The offsets and margins below clear the safe area, not `Float`.
    sx().var(FLOAT_SAFE_AREA_VAR, "0")
        .display("flex")
        .flex_direction("column")
        .width(format!(
            "min({}, calc(100% - 2 * {offset} - {start} - {end}))",
            NOTIFICATION_WIDTH.value(),
        ))
        .pointer_events("none")
        // Logical margins, not `Float`'s physical offset: they flip with `end` under RTL.
        .when(
            "horizontal-start",
            sx().margin_inline_start(format!("calc({offset} + {start})")),
        )
        .when(
            "horizontal-end",
            sx().margin_inline_end(format!("calc({offset} + {end})")),
        )
        .when(
            "fixed",
            insets("left", "right").rtl(insets("right", "left")),
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
            offset_y: vertical_offset(placement, props.fixed),
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

/// `Float`'s offsets are a translate: negative from the bottom edge. A fixed stack clears
/// the status bar, and a docked `NavigationBar` or else the home indicator, as the FAB does.
fn vertical_offset(placement: Placement, fixed: bool) -> String {
    use Placement::*;
    let offset = NOTIFICATION_OFFSET.value();
    match (placement, fixed) {
        (TopStart | TopCenter | TopEnd, false) => offset,
        (TopStart | TopCenter | TopEnd, true) => {
            format!("calc(env(safe-area-inset-top, 0px) + {offset})")
        }
        (BottomStart | BottomCenter | BottomEnd, false) => format!("calc(-1 * {offset})"),
        (BottomStart | BottomCenter | BottomEnd, true) => format!(
            "calc(-1 * ({} + {offset}))",
            NAVIGATION_BAR_HEIGHT_VAR.value_or("env(safe-area-inset-bottom, 0px)")
        ),
        (CenterStart | CenterCenter | CenterEnd, _) => "0px".to_string(),
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

    /// Todos 5442, 5445: a fixed stack clears the insets and a docked bar; a contained one not.
    #[test]
    fn a_fixed_stack_clears_the_safe_area_and_the_navigation_bar() {
        let css = crate::css::Stylesheet::from(&*STACK_SX);
        let css = css.as_str();
        assert!(css.contains("env(safe-area-inset-left, 0px)"), "{css}");
        assert!(css.contains("env(safe-area-inset-right, 0px)"), "{css}");

        let top = vertical_offset(Placement::TopEnd, true);
        assert!(top.contains("env(safe-area-inset-top"), "{top}");
        let bottom = vertical_offset(Placement::BottomEnd, true);
        assert!(
            bottom.contains("var(--lsx-navigation-bar-height, env(safe-area-inset-bottom"),
            "{bottom}"
        );
        assert!(!vertical_offset(Placement::BottomEnd, false).contains("env("));
        assert!(!vertical_offset(Placement::TopStart, false).contains("env("));
    }
}
