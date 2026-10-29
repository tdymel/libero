use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Parts, ScaleOrCss, aria_label_of, input_from_str, names_itself,
            use_name_warning,
        },
        form::{
            Asks, ComboboxState, DropdownPart, PreparedField, PreparedFrame, clear_button,
            field_parts_enum, field_props, use_combobox, use_field, use_field_frame,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, use_element, use_theme},
    str_enum::str_enum,
    theme::Size,
};

mod engine;
mod list;

pub(super) use self::engine::{CascaderConfig, Engine, EngineSetup, Naming, use_cascader_engine};
use super::{
    nodes::{FlatPath, flatten_paths},
    option::CascaderNode,
    trigger::{CASCADER_TRIGGER_SX, Trigger, cascader_trigger, trigger_control},
};

str_enum! {
    /// How an open `Cascader` draws its tree.
    #[state_prefix = "layout"]
    pub enum CascaderLayout {
        /// One listbox per level, side by side.
        #[default]
        Columns = "columns",
        /// One row per full path, joined by `separator`. A search always renders this.
        Paths = "paths",
    }
}

input_from_str!(CascaderLayout);

/// The value-free tree, compared by `Rc` pointer; see `CascaderCoreProps::options`.
#[derive(Clone)]
pub(super) struct CascaderTree(pub Rc<Vec<CascaderNode>>);

impl PartialEq for CascaderTree {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// One row by position; `Cascader<T>` looks its option up by `indices`.
pub(super) struct CascaderRowArgs {
    pub indices: Vec<usize>,
    pub expanded: bool,
    pub selected: bool,
}

/// Always equal, like `Tree`'s `ErasedRenderNode`; the never-equal `options` keeps it fresh.
#[derive(Clone)]
pub(super) struct CascaderRender(pub Rc<dyn Fn(CascaderRowArgs) -> Element>);

impl PartialEq for CascaderRender {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Query, index path and joined label in; whether the path survives out.
type MatchFn = dyn Fn(&str, &[usize], String) -> bool;

/// A caller's `filter`, erased the same way as `CascaderRender`.
#[derive(Clone)]
pub(super) struct CascaderMatch(pub Rc<MatchFn>);

impl PartialEq for CascaderMatch {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Below the theme's `sm`, where two columns no longer fit a phone.
pub(super) use crate::components::form::phone_sheet::narrow_query;

field_parts_enum! {
    /// [`Cascader`](super::Cascader)'s inner parts, for its `parts` prop: a
    /// field's, and the value.
    pub enum CascaderPart framed {
        /// The joined path or the placeholder, in the trigger.
        Value = "value" => "& > * > [data-slot='frame'] > [data-slot='control'] > [data-slot='value']",
    }
}

field_props! {
    parts(CascaderPart);
    pub(crate) struct CascaderCoreProps {
        /// Wrapped fresh each render on purpose: never equal, so a stale `node` or `filter` can't
        /// survive memoization ([[codebase/dioxus-memoization-traps]]).
        options: CascaderTree,
        committed: Option<Vec<usize>>,
        /// The option to commit next, by index path; `None` clears.
        onpick: EventHandler<Option<Vec<usize>>>,
        /// `None` draws the label.
        #[props(default)]
        node: Option<CascaderRender>,
        /// The trigger's text. Empty shows `placeholder`.
        display: String,
        separator: String,
        any_level: bool,
        allow_deselect: bool,
        layout: CascaderLayout,
        searchable: bool,
        column_width: String,
        #[props(default)]
        placeholder: Option<String>,
        #[props(default)]
        search_placeholder: Option<String>,
        #[props(default)]
        clearable: bool,
        /// A hidden input carrying `form_value`: the trigger is a `div` and can't post.
        #[props(default)]
        name: Option<String>,
        /// `None` posts nothing.
        #[props(default)]
        form_value: Option<String>,
        #[props(default)]
        rules: Option<crate::components::form::FieldStatus>,
        /// `None` is a case-insensitive `contains` over the joined labels.
        #[props(default)]
        filter: Option<CascaderMatch>,
        #[props(default, into)]
        dropdown_parts: Input<Parts<DropdownPart>>,
    }
}

/// The `T`-free field under `Cascader`: index paths in and out.
/// Focus stays on the trigger (or the open search box), so its blur closes the list.
#[component]
pub(crate) fn CascaderCore(props: CascaderCoreProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.cascader.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.cascader.radius);
    let disabled = props.disabled.unwrap_or(false);
    // As `SelectCore`: read-only stays focusable and posting, but never opens.
    let readonly = props.readonly.unwrap_or(false);
    let required = crate::components::form::fieldset_required(props.required);

    let state = use_combobox();
    let opened = state.is_open() && !disabled && !readonly;
    let searchable = props.searchable && !disabled;
    let trigger_element = use_element();
    let anchor = use_element();

    let CascaderField {
        field,
        frame,
        control,
        clear,
    } = use_cascader_field(
        &props,
        state,
        trigger_element,
        size,
        &radius,
        disabled,
        readonly,
    );

    // The caller's `aria-label` names the open search box and listboxes the trigger no longer carries.
    let aria_label = aria_label_of(&props.attributes);
    use_name_warning(
        !searchable
            || field.label_id().is_some()
            || aria_label.is_some()
            || names_itself(&props.attributes),
        "Cascader: searchable without `label` or `aria_label`, so the search box is named by its placeholder only.",
    );

    let config = CascaderConfig {
        options: props.options.0.clone(),
        committed: props.committed.clone(),
        onpick: props.onpick,
        node: props.node.clone(),
        filter: props.filter.clone(),
        separator: props.separator.clone(),
        any_level: props.any_level,
        allow_deselect: props.allow_deselect,
        layout: props.layout,
        column_width: props.column_width.clone(),
        search_placeholder: props.search_placeholder.clone(),
    };
    let Engine {
        keys,
        open,
        controlled_id,
        descendant,
        empty,
        wrapper,
        clearance: _,
        status,
        hold,
        pressed,
        held,
        blurred,
    } = use_cascader_engine(EngineSetup {
        config: &config,
        names: Naming {
            trigger_id: field.id().to_string(),
            label_id: field.label_id(),
            aria_label,
            describedby: field.describedby(),
            invalid: field.invalid(),
            required,
        },
        state,
        trigger: trigger_element,
        anchor,
        field: field.root(),
        opened,
        searchable,
        inert: disabled || readonly,
        size,
        radius: radius.size,
        parts: &props.dropdown_parts,
        class: None,
        sx: None,
        attributes: Vec::new(),
    });

    let trigger = cascader_trigger(
        trigger_control(control, &field, searchable && opened && !held),
        Trigger {
            element: trigger_element,
            state,
            open,
            keys,
            disabled,
            readonly,
            searchable,
            hold: searchable.then_some(hold),
            pressed,
            held,
            blurred,
            empty,
            chevron: clear.is_none(),
            display: props.display.clone(),
            placeholder: props.placeholder.clone(),
            controlled_id,
            descendant,
        },
        opened,
        props.attributes,
    );

    let hidden = hidden_input(props.name.clone(), props.form_value.clone(), disabled);

    field.render(rsx! {
        {
            wrapper
                .element(&anchor)
                .render(HtmlTag::Div, Vec::new(), frame.render(trigger))
        }
        {hidden}
        {status}
    })
}

/// [`visible_paths`] while open; closed, the tree is not walked each render (todo 4237).
pub(super) fn open_paths(
    opened: bool,
    config: &CascaderConfig,
    query: Option<&str>,
) -> Option<Rc<Vec<FlatPath>>> {
    opened.then(|| {
        Rc::new(visible_paths(
            &config.options,
            config.any_level,
            query,
            &config.separator,
            config.filter.as_ref(),
        ))
    })
}

/// Every path of the tree, narrowed to what `query` matches while searching.
pub(super) fn visible_paths(
    nodes: &[CascaderNode],
    any_level: bool,
    query: Option<&str>,
    separator: &str,
    filter: Option<&CascaderMatch>,
) -> Vec<FlatPath> {
    let all_paths = flatten_paths(nodes, any_level);
    let Some(query) = query else {
        return all_paths;
    };
    // Trimmed, as the other search boxes, for a custom `filter` too: a trailing space must not hide a match.
    let query = query.trim();
    let needle = query.to_lowercase();
    all_paths
        .into_iter()
        .filter(|path| {
            let label = path.labels.join(separator);
            match filter {
                Some(filter) => (filter.0)(query, &path.indices, label),
                None => label.to_lowercase().contains(&needle),
            }
        })
        .collect()
}

pub(super) fn option_id(id: &str, level: usize, index: usize) -> String {
    format!("{id}-option-{level}-{index}")
}

/// The narrow list's parent row, heading column `level`.
pub(super) fn parent_option_id(id: &str, level: usize) -> String {
    format!("{id}-option-{level}-parent")
}

/// What frames the trigger: the field, its frame, the control's style and the x.
struct CascaderField {
    field: PreparedField,
    frame: PreparedFrame,
    control: BoxStyle,
    clear: Option<Element>,
}

fn use_cascader_field(
    props: &CascaderCoreProps,
    state: ComboboxState,
    trigger: ElementHandle,
    size: Size,
    radius: &ScaleOrCss,
    disabled: bool,
    readonly: bool,
) -> CascaderField {
    // From the posted value: one no option holds still posts, so it fills the field (todo 4235).
    let empty = props.form_value.is_none();
    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.rules.clone())
        .name(props.name.as_deref())
        .required(crate::components::form::fieldset_required(props.required))
        .empty(empty)
        .asks(Asks::Select)
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let onpick = props.onpick;
    let clear = clear_button(
        props.clearable && !empty && !disabled && !readonly,
        size,
        trigger,
        Some(&field),
        move |_| {
            onpick.call(None);
            state.close();
        },
    );

    let frame = use_field_frame()
        .trailing(&clear)
        .states(field.states())
        .prepare();

    // The frame draws the ring.
    let control = use_box()
        .framework_sx(&CASCADER_TRIGGER_SX)
        .focus_ring(false)
        .states(field.states())
        .prepare();

    CascaderField {
        field,
        frame,
        control,
        clear,
    }
}

/// Posts the value alone, as `Select` does; nothing selected posts nothing.
fn hidden_input(
    name: Option<String>,
    form_value: Option<String>,
    disabled: bool,
) -> Option<Element> {
    name.map(|name| {
        rsx! {
            for value in form_value.iter().cloned() {
                input {
                    r#type: "hidden",
                    name: name.clone(),
                    value,
                    disabled: disabled.then_some(true),
                }
            }
        }
    })
}

/// The listbox `aria-controls` names: in `Columns` the cursor's column, since the strip is only `presentation`.
pub(super) fn controlled_id(listbox_id: &str, layout: CascaderLayout, cursor: &[usize]) -> String {
    match layout {
        CascaderLayout::Paths => listbox_id.to_string(),
        CascaderLayout::Columns => format!("{listbox_id}-{}", cursor.len().saturating_sub(1)),
    }
}

/// The row `aria-activedescendant` names: the cursor's deepest node, its `Paths` row, or the narrow parent row.
pub(super) fn active_descendant(
    id: &str,
    layout: CascaderLayout,
    cursor: &[usize],
    path_row: Option<usize>,
    parent_active: bool,
) -> Option<String> {
    match layout {
        CascaderLayout::Paths => path_row.map(|row| option_id(id, 0, row)),
        CascaderLayout::Columns => {
            let (last, parents) = cursor.split_last()?;
            Some(match parent_active {
                true => parent_option_id(id, parents.len()),
                false => option_id(id, parents.len(), *last),
            })
        }
    }
}
