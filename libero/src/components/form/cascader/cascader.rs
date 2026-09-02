use dioxus::prelude::*;

use crate::{
    components::{
        Input, TreeLabel, TreeNode, TreeValue, common::field_props, form::use_bound,
        navigation::erase_nodes,
    },
    hooks::use_theme,
    utils::warn,
};

use super::{
    core::{CascaderCore, CascaderLayout, CascaderNodeArgsErased, CascaderRender},
    nodes::{flatten_paths, join_labels},
};

/// What `onchange` hands back: the path that was picked, and the nodes on it.
///
/// Both, not one - the path is what posts and what `value` takes back, and the
/// chain is what a caller would otherwise look up itself to read "the label of
/// the third level". An empty `path` is the cleared selection.
#[derive(Clone, PartialEq)]
pub struct CascaderPick<T> {
    /// The picked node's ids, root to leaf.
    pub path: Vec<String>,
    /// The `data` of each node on that path, in the same order.
    pub nodes: Vec<T>,
}

/// One row under the `node` callback's brush. The row itself - its highlight,
/// its `aria-selected`, its chevron, its click - is the component's.
#[derive(Clone, PartialEq)]
pub struct CascaderNodeArgs<T> {
    pub data: T,
    /// 0 for a root.
    pub level: usize,
    /// Whether the column to the right is this node's children.
    pub expanded: bool,
    /// Whether this node is the committed path's leaf.
    pub selected: bool,
}

/// One path under test, handed to `filter` while searching.
#[derive(Clone, PartialEq)]
pub struct CascaderFilterArgs<T> {
    /// What is currently typed in the search box.
    pub query: String,
    /// The path's labels, already joined by `separator` - what the default
    /// filter matches against.
    pub label: String,
    /// The path's ids, root to leaf.
    pub path: Vec<String>,
    /// The `data` of each node on that path.
    pub nodes: Vec<T>,
}

field_props! {
    pub struct CascaderProps<T: TreeValue> {
        /// The tree to walk. Ids are unique across the whole tree, not just
        /// among siblings - the value is a path of them.
        data: Vec<TreeNode<T>>,
        /// The selected path's ids, root to leaf. Strictly controlled - pair
        /// it with `onchange`, or bind it with `name`.
        #[props(default)]
        value: Option<Vec<String>>,
        /// Called with the path to select next. An empty `path` means the
        /// selection was cleared - by the x, or by `allow_deselect`.
        #[props(default)]
        onchange: Option<EventHandler<CascaderPick<T>>>,
        /// Mantine's `changeOnSelect`: a branch commits as well as expanding.
        /// Off, only a leaf can be picked.
        #[props(default)]
        any_level: Option<bool>,
        /// Picking the committed path again clears it. On by default.
        #[props(default)]
        allow_deselect: Option<bool>,
        /// `"columns"` (default) or `"paths"`. A search renders `"paths"`
        /// whatever this says.
        #[props(default, into)]
        layout: Input<CascaderLayout>,
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the paths while searching. Defaults to a case-insensitive
        /// `contains` over the joined path.
        #[props(default)]
        filter: Option<Callback<CascaderFilterArgs<T>, bool>>,
        /// Between labels, in the trigger and in a `"paths"` row. `" / "` by
        /// default.
        #[props(default)]
        separator: Option<String>,
        /// Overrides the joined labels in the trigger. A `String`, not a node:
        /// the value slot clips for `text-overflow: ellipsis`.
        #[props(default)]
        format_value: Option<Callback<Vec<T>, String>>,
        /// Draws one row's content. Defaults to `TreeLabel::tree_label`.
        #[props(default)]
        node: Option<Callback<CascaderNodeArgs<T>, Element>>,
        /// One column's width. Defaults to the theme's; `"max-content"` is
        /// how a column takes the width of its longest row.
        #[props(default)]
        column_width: Option<String>,
        /// Shows an x that clears the selection, which is what makes
        /// `onchange` fire an empty path.
        #[props(default)]
        clearable: Option<bool>,
        /// Shown while nothing is selected.
        #[props(default)]
        placeholder: Option<String>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
        /// Emits one hidden input of that name per level, so the path posts
        /// with a native form. A path - `Listing::FIELDS.category()` - also
        /// binds the selection to the surrounding `Form`'s value when there is
        /// no `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Vec<String>>,
        /// Rules over the path, shown once the cascader loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Vec<String>>,
    }
}

/// A listbox over a tree, whose value is a path rather than a node: `Vec<String>`
/// root to leaf, with the resolved nodes handed back beside it.
///
/// It takes `Tree`'s own `TreeNode<T>`, so a caller who already has a tree
/// drops it straight in. Unlike `Tree` it is a field: it sits in a frame with a
/// label, it posts, it validates, and focus never leaves its trigger - the
/// columns are listboxes the trigger points at with `aria-activedescendant`.
///
/// ```rust,ignore
/// Cascader {
///     label: "Category",
///     data: categories(),
///     value: chosen(),
///     onchange: move |pick: CascaderPick<Category>| chosen.set(Some(pick.path)),
///     searchable: true,
/// }
/// ```
#[component]
pub fn Cascader<T: TreeValue>(props: CascaderProps<T>) -> Element {
    let theme = use_theme();

    let bound = use_bound(&props.name, props.onchange.is_some());
    let current: Vec<String> = bound
        .value()
        .or_else(|| props.value.clone())
        .unwrap_or_default();

    if props.onchange.is_none() && !bound.is_bound() {
        warn("Cascader: without `onchange` the selection can never change.");
    }
    if props.data.is_empty() {
        warn("Cascader: `data` is empty, so there is nothing to pick.");
    }

    let separator = props
        .separator
        .clone()
        .unwrap_or_else(|| String::from(" / "));
    let any_level = props.any_level.unwrap_or(false);
    let chain = chain_for(&props.data, &current);
    if !current.is_empty() && chain.is_none() {
        warn("Cascader: `value` is not a path through `data`, so nothing is selected.");
    }

    // The trigger's text. A `String` and not a node, so the value slot can
    // clip it for `text-overflow: ellipsis`.
    let display = match (&chain, &props.format_value) {
        (None, _) => String::new(),
        (Some(chain), Some(format)) => format.call(chain.clone()),
        (Some(chain), None) => {
            let labels: Vec<String> = chain.iter().map(TreeLabel::tree_label).collect();
            labels.join(&separator)
        }
    };

    // Erased once per render, on purpose - see `CascaderCoreProps::nodes`.
    let nodes = erase_nodes(&props.data);

    // The one place `T` is recovered. The core hands back an `Rc<dyn Any>` it
    // took from the very node this `T` was erased from, so the downcast cannot
    // fail; an unreachable arm is still cheaper than an `unwrap`.
    let draw = props.node;
    let node = CascaderRender::new(move |args: CascaderNodeArgsErased| {
        let Some(data) = args.data.downcast_ref::<T>() else {
            return rsx! {};
        };
        match &draw {
            Some(draw) => draw.call(CascaderNodeArgs {
                data: data.clone(),
                level: args.level,
                expanded: args.expanded,
                selected: args.selected,
            }),
            None => {
                let label = data.tree_label();
                rsx! { "{label}" }
            }
        }
    });

    // Only a *custom* filter needs `T`: the default matches the joined labels,
    // which the erased nodes already carry, so the common case costs no
    // callback and no second walk of the tree.
    //
    // Built per render, which `Callback`'s `PartialEq` normally makes a trap -
    // two callbacks from one scope compare equal, so a stale one can survive.
    // It cannot here: `CascaderCore` also takes `nodes`, whose payloads compare
    // by `Rc` pointer, so its props never compare equal either.
    let searchable = props.searchable.unwrap_or(false);
    let filter = props.filter;
    let matches = (searchable && filter.is_some()).then(|| {
        let data = props.data.clone();
        let erased = nodes.clone();
        let separator = separator.clone();
        Callback::new(move |query: String| {
            flatten_paths(&erased, any_level)
                .iter()
                .map(|path| match &filter {
                    Some(filter) => filter.call(CascaderFilterArgs {
                        query: query.clone(),
                        label: join_labels(&path.labels, &separator),
                        path: path.ids.clone(),
                        nodes: chain_for(&data, &path.ids).unwrap_or_default(),
                    }),
                    None => true,
                })
                // Annotated: a `Callback`'s return type is inferred through
                // `SpawnIfAsync`, which leaves a bare `collect` ambiguous.
                .collect::<Vec<bool>>()
        })
    });

    let onchange = props.onchange;
    let setter = bound.setter();
    let data = props.data.clone();
    let rules = props.validate.check(&current);

    rsx! {
        CascaderCore {
            nodes,
            value: current.clone(),
            onpick: move |path: Vec<String>| {
                if let Some(setter) = &setter {
                    setter.set(path.clone());
                }
                if let Some(onchange) = &onchange {
                    let nodes = chain_for(&data, &path).unwrap_or_default();
                    onchange.call(CascaderPick { path, nodes });
                }
            },
            node,
            display,
            separator,
            any_level,
            allow_deselect: props.allow_deselect.unwrap_or(true),
            layout: props.layout.copied_or_default(),
            searchable,
            column_width: props
                .column_width
                .clone()
                .unwrap_or_else(|| theme.cascader.column_width.to_string()),
            placeholder: props.placeholder,
            search_placeholder: props.search_placeholder,
            clearable: props.clearable.unwrap_or(false),
            name: bound.name().map(str::to_string),
            rules,
            matches,
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.cascader.size),
            radius: props.radius.copied_or(theme.cascader.radius),
            disabled: Some(bound.disabled(props.disabled)),
            required: props.required,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        }
    }
}

/// The `data` of each node on `ids`, or `None` when the path is not in the
/// tree. Generic, and one of only two things in this component that is - a
/// walk of `ids.len()` levels, not of the tree.
fn chain_for<T: Clone>(data: &[TreeNode<T>], ids: &[String]) -> Option<Vec<T>> {
    if ids.is_empty() {
        return None;
    }
    let mut chain = Vec::with_capacity(ids.len());
    let mut level = data;
    for id in ids {
        let node = level.iter().find(|node| &node.id == id)?;
        chain.push(node.data.clone());
        level = &node.children;
    }
    Some(chain)
}
