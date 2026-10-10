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
    hooks::{current_localization, drag_handle_sx, use_css, use_id, use_media_query},
    localization::fill,
    sx::{StaticSx, Sx, sx},
    theme::FOCUS_RING_HALO_SPREAD,
};

static SORTABLE_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .gap("xs")
        .list_style("none")
        .margin("0")
        .padding("0")
        .when("vertical", sx().flex_direction("column"))
        // A row cannot wrap while it sorts, so it scrolls inside itself (1.4.10);
        // the padding keeps the handles' focus rings clear of the scroller's clip,
        // also for a handle that focus scrolls in.
        .when(
            "horizontal",
            sx().flex_direction("row")
                .overflow_x("auto")
                .padding(FOCUS_RING_HALO_SPREAD.value())
                .with("scroll-padding-inline", FOCUS_RING_HALO_SPREAD.value()),
        )
        .when("sorting", sx().user_select("none"))
});

pub(crate) static SORTABLE_ITEM_SX: StaticSx = StaticSx::new(sortable_item_sx);

pub(crate) fn sortable_item_sx() -> Sx {
    sx().display("flex")
        .align_items("center")
        .gap("xs")
        .position("relative")
        // Only while the list sorts: on drop the order changes and the offsets
        // reset in one frame, which must not animate. `when` has no negation, so
        // the dragged item opts out below.
        .when(
            "sorting",
            sx().transition("transform 150ms ease")
                .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
        )
        .when("dragging", sx().z_index("1").transition("none"))
}

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
        .selector(
            "&:disabled, &[aria-disabled=\"true\"]",
            sx().opacity("0.4").cursor("default"),
        )
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
    let touch = use_media_query("(pointer: coarse)");

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
        // Safari drops a `list-style: none` list's role; a caller's `role` wins.
        .attr_default("role", "list")
        .element(&list.element)
        .event("onpointermove", list.onpointermove)
        .event("onpointerup", list.onpointerup)
        .event("onpointercancel", list.onpointercancel)
        .render(HtmlTag::Ul, props.attributes, props.children);

    // A touch screen reader's tap on the handle lifts nothing: point it to the buttons.
    let described = match touch() && props.move_buttons {
        true => words.touch_instructions,
        false => words.instructions,
    };

    rsx! {
        {items}
        // Hidden, not visually hidden: read only as the handles' description.
        div { id: "{instructions}", hidden: true, {described} }
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
        /// The item's current position: the items run exactly `0..n`, a gap stops every
        /// drag, a repeat warns and some items never move. Key it by its data, not this.
        index: usize,
        /// Names the item in its controls and the announcements. Unset, the handle reads the
        /// item's content, the rest "Item {n}" by where it was lifted.
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
    let name = item_name(words.item, props.label.as_deref(), props.index);
    let named = |template: &str| fill(template, &[("label", &name)]);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("dragging", dragging)
        .with("sorting", (item.sorting)())
        .into();

    let content_class = use_css(Some(&SORTABLE_CONTENT_SX), CssLayer::Framework);
    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    let handle_name = use_handle_name(words.handle, props.label.as_deref());
    let handle = use_box()
        .framework_sx(&SORTABLE_HANDLE_SX)
        .prepare()
        .element(&item.handle)
        .attr("data-slot", SortableItemPart::Handle.slot())
        .attr("type", "button")
        .attr("aria-label", handle_name.aria_label)
        .attr("aria-labelledby", handle_name.aria_labelledby)
        .attr("aria-describedby", view.instructions.clone())
        .event("onpointerdown", item.onpointerdown)
        .event("onkeydown", item.onkeydown)
        .event("onblur", item.onblur)
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! {
                Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined }
                {handle_name.words}
            },
        );

    let moves = view.move_buttons.then(|| {
        let (earlier, later) = match view.horizontal {
            true => (named(words.move_backward), named(words.move_forward)),
            false => (named(words.move_up), named(words.move_down)),
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
                div {
                    id: "{handle_name.content_id}",
                    class: content_class,
                    "data-slot": SortableItemPart::Content.slot(),
                    {props.children}
                }
                {moves}
            },
        )
}

/// What an item's controls call it: its `label`, else `item` at its current position.
pub(crate) fn item_name(item: &str, label: Option<&str>, index: usize) -> String {
    label.map_or_else(|| fill(item, &[("n", &(index + 1))]), str::to_owned)
}

/// A drag handle's name: `template` filled with the label, or unlabelled, the item's
/// content read inside the template's words (todos 1280, 2036).
pub(crate) struct HandleName {
    /// For the element holding the item's content.
    pub content_id: String,
    pub aria_label: Option<String>,
    pub aria_labelledby: Option<String>,
    /// The template's own words, hidden in the handle for `aria_labelledby`.
    pub words: Element,
}

pub(crate) fn use_handle_name(template: &'static str, label: Option<&str>) -> HandleName {
    let (content_id, before_id, after_id) = (use_id()(), use_id()(), use_id()());
    let (before, after) = template.split_once("{label}").unwrap_or((template, ""));
    let aria_labelledby = label.is_none().then(|| {
        [
            (before, &before_id),
            ("{label}", &content_id),
            (after, &after_id),
        ]
        .into_iter()
        .filter(|(words, _)| !words.trim().is_empty())
        .map(|(_, id)| id.as_str())
        .collect::<Vec<_>>()
        .join(" ")
    });
    let words = rsx! {
        if aria_labelledby.is_some() {
            span { id: "{before_id}", hidden: true, {before.trim()} }
            span { id: "{after_id}", hidden: true, {after.trim()} }
        }
    };
    HandleName {
        aria_label: label.map(|label| fill(template, &[("label", &label)])),
        content_id,
        aria_labelledby,
        words,
    }
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
