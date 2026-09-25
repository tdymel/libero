use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    reorder::SortableMove,
    use_sortable::{SortableOptions, use_labelled_sortable_item, use_sortable},
};
use crate::{
    CssLayer,
    components::{
        accessibility::VisuallyHidden,
        common::{
            Glyph, HtmlTag, Input, Orientation, Part, States, base_props, focus_ring_sx, parts_enum,
        },
        layout::use_box,
    },
    context::IconSlot,
    hooks::{current_localization, drag_handle_sx, use_css, use_id},
    sx::{StaticSx, Sx, sx},
};

static SORTABLE_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .gap("xs")
        .list_style("none")
        .margin("0")
        .padding("0")
        .when("vertical", sx().flex_direction("column"))
        .when("horizontal", sx().flex_direction("row"))
        .when("sorting", sx().user_select("none"))
});

pub(crate) static SORTABLE_ITEM_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("xs")
        .position("relative")
        // Only while another item drags: on drop the order changes and the
        // offsets reset in one frame, which must not animate.
        .when(
            "sorting && !dragging",
            sx().transition("transform 150ms ease")
                .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
        )
        .when("dragging", sx().z_index("1"))
});

pub(crate) static SORTABLE_HANDLE_SX: StaticSx =
    StaticSx::new(|| control_sx().and(drag_handle_sx()).cursor("grab"));

/// The handle's and the move buttons' box.
fn control_sx() -> Sx {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        // WCAG 2.5.8's 24px floor.
        .min_width("24px")
        .min_height("24px")
        .padding("0")
        .border("0")
        .border_radius("sm")
        .background("transparent")
        .color("inherit")
        .focus_visible(focus_ring_sx())
        .selector("& svg", sx().width("16px").height("16px"))
}

pub(crate) static SORTABLE_CONTENT_SX: StaticSx =
    StaticSx::new(|| sx().flex("1 1 auto").min_width("0"));

/// The move buttons: the handle's box, an arrow cursor, dimmed at the list's end.
pub(crate) static SORTABLE_MOVE_SX: StaticSx = StaticSx::new(|| {
    control_sx()
        .cursor("pointer")
        .selector("&:disabled", sx().opacity("0.4").cursor("default"))
        .rtl(sx().selector("& svg", sx().transform("scaleX(-1)")))
});

/// What a [`SortableItem`] reads from its [`Sortable`].
#[derive(Clone, PartialEq)]
struct SortableView {
    instructions: String,
    horizontal: bool,
    move_buttons: bool,
}

base_props! {
    pub struct SortableProps {
        /// `"vertical"` (default) stacks the items, `"horizontal"` lines them up.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Called on drop when an item changed place. Apply it to your data.
        onreorder: EventHandler<SortableMove>,
        /// Each item's two buttons moving it one slot without a drag (WCAG 2.5.7).
        /// Hidden, give the reader another way to reorder without dragging.
        #[props(default = true)]
        move_buttons: bool,
        /// [`SortableItem`]s, keyed by their data.
        children: Element,
    }
}

/// A list the user reorders by dragging each item's handle.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Sortable, SortableItem};
/// # use libero::hooks::SortableMove;
/// # fn app() -> Element {
/// let mut fruit = use_signal(|| vec!["apple", "pear", "plum"]);
/// rsx! {
///     Sortable { onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
///         for (index, name) in fruit().into_iter().enumerate() {
///             SortableItem { key: "{name}", index, "{name}" }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/sortable>
#[component]
pub fn Sortable(props: SortableProps) -> Element {
    let orientation = props.orientation.copied_or(Orientation::Vertical);
    let onreorder = props.onreorder;
    let list = use_sortable(SortableOptions {
        orientation,
        onreorder: use_callback(move |step| onreorder.call(step)),
    });
    let vertical = orientation == Orientation::Vertical;
    let words = current_localization().sortable;

    let instructions = use_id();
    let view = SortableView {
        instructions: instructions(),
        horizontal: !vertical,
        move_buttons: props.move_buttons,
    };
    let mut shared = use_context_provider(|| Signal::new(view.clone()));
    if *shared.peek() != view {
        shared.set(view);
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("sorting", (list.sorting)())
        .into();

    let items = use_box()
        .framework_sx(&SORTABLE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .element(&list.element)
        .event("onpointermove", list.onpointermove)
        .event("onpointerup", list.onpointerup)
        .event("onpointercancel", list.onpointercancel)
        .render(HtmlTag::Ul, props.attributes, props.children);

    rsx! {
        {items}
        // Hidden, not visually hidden: read only as the handles' description.
        div { id: "{instructions}", hidden: true, {words.instructions} }
        VisuallyHidden { role: "status", {list.announcement} }
    }
}

parts_enum! {
    /// [`SortableItem`]'s inner parts, for its `parts` prop. Each is a direct
    /// child, so a nested `Sortable` keeps its own styles.
    pub enum SortableItemPart {
        /// The drag handle.
        Handle = "handle" => "& > [data-slot='handle']",
        /// The wrapper round `children`.
        Content = "content" => "& > [data-slot='content']",
        /// The move up or back button.
        MoveEarlier = "move-earlier" => "& > [data-slot='move-earlier']",
        /// The move down or forward button.
        MoveLater = "move-later" => "& > [data-slot='move-later']",
    }
}

base_props! {
    parts(SortableItemPart);
    pub struct SortableItemProps {
        /// The item's current position, from 0. Key it by its data, not this.
        index: usize,
        /// Names the item in the announcements. Unset, "Item {n}" by where it was lifted.
        #[props(default, into)]
        label: Option<String>,
        children: Element,
    }
}

/// One item of a [`Sortable`]: a drag handle, `children`, then the move buttons.
///
/// Docs: <https://libero-ui.dev/data-display/sortable>
#[component]
pub fn SortableItem(props: SortableItemProps) -> Element {
    let item = use_labelled_sortable_item(props.index, props.label.clone());
    let view = use_context::<Signal<SortableView>>()();
    let words = current_localization().sortable;
    let dragging = (item.dragging)();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("dragging", dragging)
        .with("sorting", (item.sorting)())
        .into();

    let content_class = use_css(Some(&SORTABLE_CONTENT_SX), CssLayer::Framework);
    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    let handle = use_box()
        .framework_sx(&SORTABLE_HANDLE_SX)
        .prepare()
        .element(&item.handle)
        .attr("data-slot", SortableItemPart::Handle.slot())
        .attr("type", "button")
        .attr("aria-label", words.handle)
        .attr("aria-describedby", view.instructions.clone())
        .event("onpointerdown", item.onpointerdown)
        .event("onkeydown", item.onkeydown)
        .event("onblur", item.onblur)
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! { Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined } },
        );

    let moves = view.move_buttons.then(|| {
        let (earlier, later) = match view.horizontal {
            true => (words.move_backward, words.move_forward),
            false => (words.move_up, words.move_down),
        };
        let (onearlier, onlater) = (item.onearlier, item.onlater);
        rsx! {
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": SortableItemPart::MoveEarlier.slot(),
                "aria-label": earlier,
                disabled: (item.first)(),
                onmounted: item.earlier.mount(),
                onclick: move |event| onearlier.call(event),
                if view.horizontal {
                    Glyph { slot: IconSlot::ChevronLeft, icon: lucide::chevron_left::outlined }
                } else {
                    Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
                }
            }
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": SortableItemPart::MoveLater.slot(),
                "aria-label": later,
                disabled: (item.last)(),
                onmounted: item.later.mount(),
                onclick: move |event| onlater.call(event),
                if view.horizontal {
                    Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined }
                } else {
                    Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                }
            }
        }
    });

    use_box()
        .framework_sx(&SORTABLE_ITEM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(Some(item.style()))
        .prepare()
        .element(&item.element)
        .render(
            HtmlTag::Li,
            props.attributes,
            rsx! {
                {handle}
                div { class: content_class, "data-slot": SortableItemPart::Content.slot(), {props.children} }
                {moves}
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Stylesheet, components::common::part_table};

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<SortableItemPart>(),
            [
                ("handle", "& > [data-slot='handle']"),
                ("content", "& > [data-slot='content']"),
                ("move-earlier", "& > [data-slot='move-earlier']"),
                ("move-later", "& > [data-slot='move-later']"),
            ]
        );
    }

    #[test]
    fn the_neighbours_slide_only_while_sorting_and_never_under_reduced_motion() {
        let css = Stylesheet::from(&SORTABLE_ITEM_SX).as_str().to_string();

        assert!(css.contains("transition:transform 150ms ease"), "{css}");
        assert!(
            css.contains("@media (prefers-reduced-motion: reduce)"),
            "{css}"
        );
    }
}
