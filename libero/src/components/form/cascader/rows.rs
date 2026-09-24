use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{Glyph, HtmlTag, States},
        form::{ComboboxOption, ComboboxState},
        layout::{Box, BoxStyle, ScrollArea},
    },
    context::IconSlot,
    localization::{CascaderLabels, fill},
    sx::{StaticSx, Sx, sx},
    theme::{CssVar, Size},
};

use super::{
    core::{CascaderRender, CascaderRowArgs, narrow_query, option_id},
    nodes::{FlatPath, children_at, disabled_at, first_enabled, node_at},
    option::CascaderNode,
};

/// The label/chevron split and the committed mark; the rest is `ComboboxOption`'s.
fn cascader_rows_sx() -> Sx {
    sx().selector(
        "& [data-slot='label']",
        sx().flex("1 1 auto")
            .min_width("0")
            .overflow("hidden")
            .text_overflow("ellipsis")
            .white_space("nowrap"),
    )
    .selector(
        "& [data-slot='branch']",
        sx().flex("0 0 auto")
            .display("inline-flex")
            .width("1em")
            .height("1em")
            .transform("rotate(-90deg)")
            // The next column opens to the left.
            .rtl(sx().transform("rotate(90deg)")),
    )
    .selector(
        "& [data-slot='branch'] > svg",
        sx().width("1em").height("1em"),
    )
    // `selected` follows the cursor here, so the committed path needs its own mark.
    .selector("& [data-state~='committed']", sx().font_weight("700"))
    // On top of the row's half opacity, so a disabled label reads as dimmed text.
    .selector("& [data-state~='disabled']", sx().color("text-dimmed"))
}

/// `column_width`, set on each column's `style` so the narrow rule can override the width.
const COLUMN_WIDTH: CssVar = CssVar::new("--lsx-cascader-column-width");

/// The narrow header: back to the parent's level. A `button` inherits neither font nor color.
fn drill_back_sx() -> Sx {
    sx().display("flex")
        .align_items("center")
        .gap("4px")
        .width("100%")
        .padding("4px 8px")
        .border("none")
        .border_bottom("1px solid")
        .border_color("muted.2")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .font_weight("600")
        .line_height("1.5")
        .text_align("start")
        .cursor("pointer")
        .hover(sx().background("muted.1"))
        .selector(
            "& > [data-slot='back']",
            sx().flex("0 0 auto")
                .display("inline-flex")
                .width("1em")
                .height("1em")
                .transform("rotate(90deg)")
                .rtl(sx().transform("rotate(-90deg)")),
        )
        .selector(
            "& > [data-slot='back'] > svg",
            sx().width("1em").height("1em"),
        )
        .selector(
            "& > [data-slot='title']",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
}

/// One listbox per level, side by side, each scrolling on its own.
/// Narrow, only the cursor's column shows, under a back header (todo 1084).
pub(super) static CASCADER_COLUMNS_SX: StaticSx = StaticSx::new(|| {
    cascader_rows_sx()
        .display("flex")
        .align_items("stretch")
        .gap("4px")
        // Only where the dropdown is narrower than its columns; never on a desktop.
        .overflow_x("auto")
        // Grows into a wider trigger's room, or the rows' chevrons stop short of the edge.
        .selector(
            "& > [data-slot='column']",
            sx().flex("1 0 auto")
                .width(COLUMN_WIDTH.value())
                .min_width("0")
                .display("flex")
                .flex_direction("column"),
        )
        .selector(
            "& > [data-slot='column'] + [data-slot='column']",
            sx().border_left("1px solid")
                .border_color("muted.2")
                .rtl(sx().border_left("none").border_right("1px solid")),
        )
        .selector("& > [data-slot='drill-back']", sx().display("none"))
        .selector("& [data-slot='pick-parent']", sx().display("none"))
        .media(
            narrow_query(),
            sx().flex_direction("column")
                .gap("0")
                .selector("& > [data-slot='drill-back']", drill_back_sx())
                .selector(
                    "& > [data-slot='column']:not(:last-child)",
                    sx().display("none"),
                )
                .selector(
                    "& > [data-slot='column']",
                    sx().width("auto").min_width(COLUMN_WIDTH.value()),
                )
                .selector(
                    "& > [data-slot='column'] + [data-slot='column']",
                    sx().border_left("none").rtl(sx().border_right("none")),
                )
                .selector("& [data-slot='pick-parent']", sx().display("flex")),
        )
});

static CASCADER_PATHS_SX: StaticSx = StaticSx::new(cascader_rows_sx);

/// Draws the rows of both layouts, for one render.
pub(super) struct CascaderRows {
    pub(super) nodes: Rc<Vec<CascaderNode>>,
    pub(super) node: Option<CascaderRender>,
    pub(super) separator: String,
    /// The combobox's id, the base of every option id.
    pub(super) id: String,
    pub(super) cursor: Signal<Vec<usize>>,
    pub(super) cursor_now: Vec<usize>,
    /// Empty for none.
    pub(super) committed: Vec<usize>,
    pub(super) commit: Rc<dyn Fn(Vec<usize>, bool)>,
    pub(super) any_level: bool,
    pub(super) size: Size,
    pub(super) radius: Size,
    pub(super) labels: CascaderLabels,
    pub(super) state: ComboboxState,
}

impl CascaderRows {
    /// One node through the caller's `node`; a `Paths` row draws one per level.
    fn node(&self, prefix: &[usize]) -> Element {
        let cursor = &self.cursor_now;
        match (node_at(&self.nodes, prefix), &self.node) {
            (Some(_), Some(draw)) => (draw.0)(CascaderRowArgs {
                indices: prefix.to_vec(),
                expanded: cursor.starts_with(prefix) && cursor.len() > prefix.len(),
                selected: self.committed.as_slice() == prefix,
            }),
            (Some(node), None) => rsx! { "{node.label}" },
            (None, _) => rsx! {},
        }
    }

    fn row(&self, indices: Vec<usize>, level: usize, index: usize, whole_path: bool) -> Element {
        let nodes = &self.nodes;
        let node = node_at(nodes, &indices);
        let has_children = node.is_some_and(|node| !node.children.is_empty());
        let row_disabled = disabled_at(nodes, &indices);
        // One row per column: the active descendant always carries `aria-selected` (APG).
        let on_cursor = self.cursor_now.starts_with(&indices);
        let is_cursor = self.cursor_now == indices;
        // `Paths` is one listbox holding the cursor's ancestors too, so only the cursor row is marked.
        let marked = match whole_path {
            true => is_cursor,
            false => on_cursor,
        };
        let is_committed = self.committed == indices;
        let content = match whole_path {
            false => self.node(&indices),
            true => {
                let levels: Vec<Element> = (1..=indices.len())
                    .map(|depth| self.node(&indices[..depth]))
                    .collect();
                let separator = &self.separator;
                rsx! {
                    for (at , drawn) in levels.into_iter().enumerate() {
                        if at > 0 {
                            span { "data-slot": "separator", "{separator}" }
                        }
                        {drawn}
                    }
                }
            }
        };
        let picked = indices.clone();
        // A clicked branch moves the cursor to its first child, since the cursor row never expands.
        let next_cursor = match first_enabled(children_at(nodes, &indices)) {
            Some(child) => {
                let mut next = indices.clone();
                next.push(child);
                next
            }
            None => indices.clone(),
        };
        let (commit, any_level, mut cursor) = (self.commit.clone(), self.any_level, self.cursor);
        rsx! {
            ComboboxOption {
                key: "{level}-{index}",
                id: option_id(&self.id, level, index),
                size: self.size,
                radius: self.radius,
                selected: marked,
                active: is_cursor,
                states: States::new().with("committed", is_committed),
                disabled: row_disabled,
                onpick: move |_| {
                    cursor.set(next_cursor.clone());
                    match has_children {
                        false => commit(picked.clone(), true),
                        true if any_level => commit(picked.clone(), false),
                        true => {}
                    }
                },
                span { "data-slot": "label", {content} }
                if has_children && !whole_path {
                    span { "data-slot": "branch", Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined } }
                }
            }
        }
    }

    /// The narrow header over a child level: pops the cursor, as Left does.
    /// Never focused (`tabindex=-1`, the box cancels `mousedown`): focus stays on the trigger.
    fn drill_back(&self) -> Option<Element> {
        let (_, parents) = self.cursor_now.split_last()?;
        let parent = node_at(&self.nodes, parents)?;
        let label = parent.label.clone();
        let name = fill(self.labels.back, &[("label", &label)]);
        let (mut cursor, back_to) = (self.cursor, parents.to_vec());
        Some(rsx! {
            button {
                "data-slot": "drill-back",
                r#type: "button",
                tabindex: "-1",
                "aria-label": name,
                onclick: move |_| cursor.set(back_to.clone()),
                span { "data-slot": "back", Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined } }
                span { "data-slot": "title", "{label}" }
            }
        })
    }

    /// Under `any_level`, the narrow list's first row picks the parent: a tap on it only drilled in.
    /// Outside the keyboard's rows: Enter on the parent already picks it.
    fn pick_parent(&self, level: usize) -> Option<Element> {
        let parents = self.cursor_now.get(..level).filter(|_| level > 0)?;
        let parent = node_at(&self.nodes, parents).filter(|_| self.any_level)?;
        if disabled_at(&self.nodes, parents) {
            return None;
        }
        let text = fill(self.labels.select, &[("label", &parent.label)]);
        let is_committed = self.committed.as_slice() == parents;
        let (commit, state, picked) = (self.commit.clone(), self.state, parents.to_vec());
        Some(rsx! {
            ComboboxOption {
                key: "{level}-parent",
                id: format!("{}-option-{level}-parent", self.id),
                size: self.size,
                radius: self.radius,
                selected: false,
                states: States::new().with("committed", is_committed),
                "data-slot": "pick-parent",
                onpick: move |_| match is_committed {
                    // Re-picking would clear it under `allow_deselect`.
                    true => state.close(),
                    false => commit(picked.clone(), true),
                },
                span { "data-slot": "label", "{text}" }
            }
        })
    }

    /// One column per level the cursor reached (the roots at least); never ahead of the cursor.
    pub(super) fn columns(
        &self,
        strip: BoxStyle,
        listbox_id: &str,
        label_id: Option<String>,
        width: &str,
        max_height: &'static str,
    ) -> Element {
        let cursor = &self.cursor_now;
        let depth = cursor.len().max(1);
        let columns: Vec<Element> = (0..depth)
            .map(|level| {
                let parents = cursor[..level].to_vec();
                let column = children_at(&self.nodes, &parents);
                let highlighted = cursor.get(level).copied();
                let scroll_y = highlighted
                    .filter(|_| column.len() > 1)
                    .map(|index| index as f64 / (column.len() - 1) as f64 * 100.0);
                // Named by its parent row, so no English literal is needed.
                let labelled_by = match level {
                    0 => label_id.clone(),
                    _ => Some(option_id(&self.id, level - 1, cursor[level - 1])),
                };
                let rows: Vec<Element> = column
                    .iter()
                    .enumerate()
                    .map(|(index, _)| {
                        let mut indices = parents.clone();
                        indices.push(index);
                        self.row(indices, level, index, false)
                    })
                    .collect();
                let pick_parent = (level + 1 == depth)
                    .then(|| self.pick_parent(level))
                    .flatten();
                rsx! {
                    div {
                        key: "{level}",
                        "data-slot": "column",
                        style: "{COLUMN_WIDTH.name()}:{width}",
                        ScrollArea {
                            sx: sx().max_height(max_height),
                            scroll_position_y: scroll_y,
                            id: format!("{listbox_id}-{level}"),
                            "role": "listbox",
                            "aria-labelledby": labelled_by,
                            {pick_parent}
                            for row in rows {
                                {row}
                            }
                        }
                    }
                }
            })
            .collect();
        let back = self.drill_back();
        strip
            .attr("id", listbox_id.to_string())
            .attr("role", "presentation")
            .render(
                HtmlTag::Div,
                Vec::new(),
                rsx! {
                    {back}
                    for column in columns {
                        {column}
                    }
                },
            )
    }

    /// One row per path in `visible`; `path_row` is the cursor's.
    pub(super) fn paths(
        &self,
        visible: &[FlatPath],
        path_row: Option<usize>,
        listbox_id: &str,
        label_id: Option<String>,
        max_height: &'static str,
    ) -> Element {
        let rows: Vec<Element> = visible
            .iter()
            .enumerate()
            .map(|(index, path)| self.row(path.indices.clone(), 0, index, true))
            .collect();
        let scroll_y = path_row
            .filter(|_| visible.len() > 1)
            .map(|row| row as f64 / (visible.len() - 1) as f64 * 100.0);
        rsx! {
            Box {
                framework_sx: &CASCADER_PATHS_SX,
                ScrollArea {
                    sx: sx().max_height(max_height),
                    scroll_position_y: scroll_y,
                    id: listbox_id.to_string(),
                    "role": "listbox",
                    "aria-labelledby": label_id,
                    for row in rows {
                        {row}
                    }
                }
            }
        }
    }
}
