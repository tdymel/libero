use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    reorder::SortableMove,
    use_sortable::{SortableOptions, use_sortable, use_sortable_item},
};
use crate::{
    CssLayer,
    components::{
        common::{Glyph, HtmlTag, Input, Orientation, States, base_props, focus_ring_sx},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{current_localization, drag_handle_sx, use_css},
    sx::{StaticSx, sx},
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

static SORTABLE_ITEM_SX: StaticSx = StaticSx::new(|| {
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

static SORTABLE_HANDLE_SX: StaticSx = StaticSx::new(|| {
    sx().and(drag_handle_sx())
        .display("inline-flex")
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
        .cursor("grab")
        .focus_visible(focus_ring_sx())
        .selector("& svg", sx().width("16px").height("16px"))
});

static SORTABLE_CONTENT_SX: StaticSx = StaticSx::new(|| sx().flex("1 1 auto").min_width("0"));

base_props! {
    pub struct SortableProps {
        /// `"vertical"` (default) stacks the items, `"horizontal"` lines them up.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Called on drop when an item changed place. Apply it to your data.
        onreorder: EventHandler<SortableMove>,
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

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("sorting", (list.sorting)())
        .into();

    use_box()
        .framework_sx(&SORTABLE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .element(&list.element)
        .event("onpointermove", list.onpointermove)
        .event("onpointerup", list.onpointerup)
        .event("onpointercancel", list.onpointercancel)
        .render(HtmlTag::Ul, props.attributes, props.children)
}

base_props! {
    pub struct SortableItemProps {
        /// The item's current position, from 0. Key it by its data, not this.
        index: usize,
        children: Element,
    }
}

/// One item of a [`Sortable`]: a drag handle, then `children`.
///
/// Docs: <https://libero-ui.dev/data-display/sortable>
#[component]
pub fn SortableItem(props: SortableItemProps) -> Element {
    let item = use_sortable_item(props.index);
    let dragging = (item.dragging)();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("dragging", dragging)
        .with("sorting", (item.sorting)())
        .into();

    let content_class = use_css(Some(&SORTABLE_CONTENT_SX), CssLayer::Framework);
    let handle = use_box()
        .framework_sx(&SORTABLE_HANDLE_SX)
        .prepare()
        .element(&item.handle)
        .attr("type", "button")
        .attr("aria-label", current_localization().sortable.handle)
        .event("onpointerdown", item.onpointerdown)
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! { Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined } },
        );

    use_box()
        .framework_sx(&SORTABLE_ITEM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(Some(item.style()))
        .prepare()
        .element(&item.element)
        .render(
            HtmlTag::Li,
            props.attributes,
            rsx! {
                {handle}
                div { class: content_class, {props.children} }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Stylesheet;

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
