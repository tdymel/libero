use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Orientation, Part, States, TOOLBAR_ITEM, ToolbarScope, base_props,
            has_shortcut_modifier, names_itself, neighbour, parts_enum, use_name_warning,
            use_provide_toolbar, use_toolbar,
        },
        layout::use_box,
    },
    hooks::{ElementHandle, use_element},
    platform::{ElementApi, arrow_target, key_taken, logical_key, typing_target},
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, Size, SizeCss},
};

parts_enum! {
    /// [`Toolbar`]'s inner parts, for its `parts` prop.
    pub enum ToolbarPart {
        /// A [`ToolbarGroup`].
        Group = "group" => "& [data-slot='group']",
        /// A [`ToolbarSeparator`].
        Separator = "separator" => "& [data-slot='separator']",
    }
}

// Groups and separators are styled from the bar, so a toolbar costs one class.
static TOOLBAR_SX: StaticSx = StaticSx::new(|| {
    let gap = SizeCss::SPACING.value(Size::Xs);
    let line = ColorCss::MUTED.value(ColorShade::S4);
    let group = ToolbarPart::Group.selector();
    let separator = ToolbarPart::Separator.selector();

    sx().display("flex")
        .align_items("center")
        .gap(gap.clone())
        .max_width("100%")
        .selector(
            group,
            sx().display("flex").align_items("center").gap(gap.clone()),
        )
        .selector(separator, sx().flex("0 0 auto").background(line))
        .when(
            Orientation::Horizontal.state_name(),
            // A long bar wraps rather than widening the page (1.4.10).
            sx().flex_wrap("wrap")
                .selector(group, sx().flex_wrap("wrap"))
                .selector(separator, sx().width("1px").align_self("stretch")),
        )
        .when(
            Orientation::Vertical.state_name(),
            sx().flex_direction("column")
                .align_items("stretch")
                .selector(group, sx().flex_direction("column").align_items("stretch"))
                .selector(separator, sx().height("1px")),
        )
});

base_props! {
    parts(ToolbarPart);
    pub struct ToolbarProps {
        /// Unset, `"horizontal"`: Left and Right move. `"vertical"`: Up and Down.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Whether the arrow keys wrap at the ends. Unset, `true`.
        #[props(default)]
        loop_focus: Option<bool>,
        /// `Button`s, `ActionIcon`s, `Select`s, `ButtonGroup`s, [`ToolbarGroup`]s and [`ToolbarSeparator`]s.
        children: Element,
    }
}

/// A row of controls that is one Tab stop: the arrow keys move between them,
/// Home and End jump to the ends. Name it with an `aria-label`.
///
/// Disabled `Button`s and `ActionIcon`s inside stay focusable, so a keyboard
/// user still finds them.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ActionIcon, Toolbar, ToolbarGroup, ToolbarSeparator};
/// # fn app() -> Element {
/// rsx! {
///     Toolbar { "aria-label": "Formatting",
///         ToolbarGroup { "aria-label": "Style",
///             ActionIcon { aria_label: "Bold", "B" }
///             ActionIcon { aria_label: "Italic", "I" }
///         }
///         ToolbarSeparator {}
///         ActionIcon { aria_label: "Undo", disabled: true, "↶" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/toolbar>
#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let loop_focus = props.loop_focus.unwrap_or(true);
    let scope = use_provide_toolbar(orientation);
    let bar = use_element();

    use_name_warning(
        names_itself(&props.attributes),
        "Toolbar: no `aria-label` or `aria-labelledby`, so it is announced as just \"toolbar\".",
    );

    let onkeydown = move |event: KeyboardEvent| {
        // A press the item took (`Select`'s Home), text entry or a slider keeps its keys.
        if key_taken(&event)
            || typing_target(&event)
            || arrow_target(&event)
            || has_shortcut_modifier(&event)
        {
            return;
        }
        let vertical = orientation == Orientation::Vertical;
        let step = match logical_key(&event) {
            Key::ArrowRight if !vertical => Some(1),
            Key::ArrowLeft if !vertical => Some(-1),
            Key::ArrowDown if vertical => Some(1),
            Key::ArrowUp if vertical => Some(-1),
            Key::Home | Key::End => None,
            _ => return,
        };
        let Ok(items) = bar.query_selector_all(&format!("[{TOOLBAR_ITEM}]")) else {
            return;
        };
        let Some(at) = items.iter().position(|item| item.is_focused()) else {
            return;
        };
        // Disabled items stay focusable, so none is skipped.
        let target = match (step, event.key()) {
            (Some(step), _) if loop_focus => neighbour(&vec![false; items.len()], at, step),
            // At an end that does not wrap, focus stays.
            (Some(step), _) => at
                .checked_add_signed(step)
                .filter(|&next| next < items.len())
                .or(Some(at)),
            (None, Key::Home) => Some(0),
            (None, _) => items.len().checked_sub(1),
        };
        event.prevent_default();
        if let Some(target) = target.and_then(|target| items.get(target)) {
            // Here too: Blitz fires `focusin` before the focus moves.
            if let Some(id) = marked_id(target.as_ref()) {
                scope.focused(id);
            }
            let _ = target.focus();
        }
    };

    // Whichever item takes focus, by arrow, click or script, becomes the tab stop.
    // Blitz fires `focusin` before the focus moves: there an item's own click reports it.
    let onfocusin = move |_: FocusEvent| follow_focus(&bar, scope);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .into();

    use_box()
        .framework_sx(&TOOLBAR_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .prepare()
        .element(&bar)
        .event("onkeydown", onkeydown)
        .event("onfocusin", onfocusin)
        .attr("role", "toolbar")
        // `horizontal` is the role's default.
        .attr(
            "aria-orientation",
            (orientation == Orientation::Vertical).then_some("vertical"),
        )
        .render(HtmlTag::Div, props.attributes, props.children)
}

/// Makes the focused item, if any, the tab stop.
fn follow_focus(bar: &ElementHandle, scope: ToolbarScope) {
    let Ok(items) = bar.query_selector_all(&format!("[{TOOLBAR_ITEM}]")) else {
        return;
    };
    let Some(focused) = items.iter().find(|item| item.is_focused()) else {
        return;
    };
    // A WebView reads no attribute: ask each id's element instead.
    let id = marked_id(focused.as_ref()).or_else(|| {
        scope.items().into_iter().find(|id| {
            bar.query_selector(&format!("[{TOOLBAR_ITEM}=\"{id}\"]"))
                .is_ok_and(|item| item.is_focused())
        })
    });
    if let Some(id) = id {
        scope.focused(id);
    }
}

/// `item`'s [`TOOLBAR_ITEM`] id, `None` where the renderer reads no attribute.
fn marked_id(item: &dyn ElementApi) -> Option<u64> {
    item.attribute(TOOLBAR_ITEM).ok()??.parse().ok()
}

base_props! {
    pub struct ToolbarGroupProps {
        children: Element,
    }
}

/// A named section of a [`Toolbar`]: `role="group"`, so give it an `aria-label`.
/// Its controls stay in the toolbar's arrow order.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ActionIcon, Toolbar, ToolbarGroup};
/// # fn app() -> Element {
/// rsx! {
///     Toolbar { "aria-label": "Editor",
///         ToolbarGroup { "aria-label": "History",
///             ActionIcon { aria_label: "Undo", "↶" }
///             ActionIcon { aria_label: "Redo", "↷" }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/toolbar>
#[component]
pub fn ToolbarGroup(props: ToolbarGroupProps) -> Element {
    use_box()
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("role", "group")
        .attr("data-slot", ToolbarPart::Group.slot())
        .render(HtmlTag::Div, props.attributes, props.children)
}

base_props! {
    pub struct ToolbarSeparatorProps {}
}

/// A line between two sections of a [`Toolbar`], across its axis. Not focusable.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ActionIcon, Toolbar, ToolbarSeparator};
/// # fn app() -> Element {
/// rsx! {
///     Toolbar { "aria-label": "Editor",
///         ActionIcon { aria_label: "Cut", "✂" }
///         ToolbarSeparator {}
///         ActionIcon { aria_label: "Undo", "↶" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/toolbar>
#[component]
pub fn ToolbarSeparator(props: ToolbarSeparatorProps) -> Element {
    // Perpendicular to the bar; `horizontal` is the role's default.
    let across = match use_toolbar().map(|toolbar| toolbar.orientation()) {
        Some(Orientation::Vertical) | None => None,
        Some(Orientation::Horizontal) => Some("vertical"),
    };
    use_box()
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("role", "separator")
        .attr("aria-orientation", across)
        .attr("data-slot", ToolbarPart::Separator.slot())
        .render(HtmlTag::Div, props.attributes, rsx! {})
}
