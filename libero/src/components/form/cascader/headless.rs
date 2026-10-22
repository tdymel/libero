use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            ComboboxState, HtmlTag, Input, Options, ScaleOrCss, base_props, css_string,
            has_shortcut_modifier,
        },
        form::{DropdownPart, use_combobox},
        layout::use_box,
    },
    hooks::{use_element, use_theme},
    platform::{SCROLL_MARGIN_BOTTOM_VAR, blur_counts, mounted_by_selector},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::Size,
    utils::warn,
};

use super::{
    cascader::{CascaderFilterArgs, CascaderNodeArgs, erase_filter, erase_node},
    core::{CascaderConfig, CascaderLayout, Engine, EngineSetup, Naming, use_cascader_engine},
    option::{CascaderOption, erase, indices_for_value, options_at},
    trigger::trigger_aria,
};

/// What the trigger says about the open list, written by `CascaderDropdown` as it moves.
#[derive(Clone, Default, PartialEq)]
struct TriggerAria {
    searchable: bool,
    disabled: bool,
    empty: bool,
    controls: String,
    descendant: Option<String>,
}

/// A headless cascader: the tree, the committed value, the open state and the trigger's wiring.
///
/// From [`use_cascader`]; render the cascade with [`CascaderDropdown`].
#[derive(Clone)]
pub struct CascaderState<T: Options> {
    data: Rc<Vec<CascaderOption<T>>>,
    value: Option<T>,
    onchange: Callback<Option<T>>,
    combobox: ComboboxState,
    aria: Signal<TriggerAria>,
}

/// Never equal across renders: `data` is fresh each time, so a stale tree can't survive memoization.
impl<T: Options> PartialEq for CascaderState<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data) && self.value == other.value
    }
}

/// One headless cascader over `data`, for a trigger of your own.
///
/// Strictly controlled, as `Cascader`: `value` and `data` are read fresh each render, and
/// `onchange` gets the next value, or `None` under `allow_deselect`. No hidden input and no
/// `Form` binding: the caller owns the value.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, CascaderDropdown, CascaderOption, use_cascader};
/// # fn app() -> Element {
/// let mut chosen = use_signal(|| None::<String>);
/// let cascader = use_cascader(
///     vec![CascaderOption::new("t1", "Tier 1").children(vec![
///         CascaderOption::new("iron", "Iron ore"),
///     ])],
///     chosen(),
///     move |next| chosen.set(next),
/// );
/// let label = cascader.labels().last().cloned().unwrap_or("Pick an ore".into());
/// rsx! {
///     CascaderDropdown { state: cascader.clone(),
///         Button { attributes: cascader.a11y_attributes(), "{label}" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/cascader>
pub fn use_cascader<T: Options>(
    data: Vec<CascaderOption<T>>,
    value: Option<T>,
    onchange: impl FnMut(Option<T>) + 'static,
) -> CascaderState<T> {
    CascaderState {
        data: Rc::new(data),
        value,
        onchange: use_callback(onchange),
        combobox: use_combobox(),
        aria: use_signal(TriggerAria::default),
    }
}

impl<T: Options> CascaderState<T> {
    pub fn id(&self) -> String {
        self.combobox.id()
    }

    pub fn is_open(&self) -> bool {
        self.combobox.is_open()
    }

    pub fn close(&self) {
        self.combobox.close();
    }

    pub fn value(&self) -> Option<T> {
        self.value.clone()
    }

    /// The committed option's values, root first; empty while nothing in `data` holds `value`.
    pub fn selected_path(&self) -> Vec<T> {
        self.path()
            .iter()
            .map(|option| option.value.clone())
            .collect()
    }

    /// The committed option's labels, root first, as [`selected_path`](Self::selected_path).
    pub fn labels(&self) -> Vec<String> {
        self.path()
            .iter()
            .map(|option| option.label.clone())
            .collect()
    }

    /// The trigger's id and combobox wiring. Spread it on the `button` inside the `CascaderDropdown`.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{Button, CascaderDropdown, use_cascader};
    /// # fn app() -> Element {
    /// # let cascader = use_cascader::<String>(Vec::new(), None, |_| {});
    /// # rsx! {
    /// CascaderDropdown { state: cascader.clone(),
    ///     Button { attributes: cascader.a11y_attributes(), "Pick" }
    /// }
    /// # } }
    /// ```
    pub fn a11y_attributes(&self) -> Vec<Attribute> {
        let aria = self.aria.read();
        let opened = self.is_open() && !aria.disabled;
        let mut attributes = vec![attr_id(&self.id())];
        attributes.extend(trigger_aria(
            aria.searchable,
            opened,
            aria.empty,
            &aria.controls,
            aria.descendant.clone(),
        ));
        attributes
    }

    fn path(&self) -> Vec<&CascaderOption<T>> {
        self.value
            .as_ref()
            .and_then(|value| indices_for_value(&self.data, value))
            .map(|indices| options_at(&self.data, &indices))
            .unwrap_or_default()
    }
}

fn trigger_id(id: &str) -> String {
    format!("{id}-trigger")
}

fn attr_id(id: &str) -> Attribute {
    crate::components::common::attr("id", trigger_id(id))
}

/// Takes no box of its own: the caller's trigger lays out as if unwrapped.
static CASCADER_WRAPPER_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

base_props! {
    parts(DropdownPart);
    pub struct CascaderDropdownProps<T: Options> {
        /// From [`use_cascader`].
        state: CascaderState<T>,
        /// A branch commits its own value as well as expanding. Off, only a leaf can be picked.
        #[props(default)]
        any_level: Option<bool>,
        /// Picking the committed option again clears it. Off by default, as in APG.
        #[props(default)]
        allow_deselect: Option<bool>,
        /// `"columns"` (default) or `"paths"`. A search always renders `"paths"`.
        #[props(default, into)]
        layout: Input<CascaderLayout>,
        /// Puts a search box at the top of the list.
        #[props(default)]
        searchable: Option<bool>,
        /// Narrows the paths while searching. Defaults to a case-insensitive `contains` over the joined path.
        #[props(default)]
        filter: Option<Callback<CascaderFilterArgs<T>, bool>>,
        /// Between labels in a `"paths"` row. Defaults to `" / "`.
        #[props(default)]
        separator: Option<String>,
        /// Draws one row's content. Defaults to the label.
        #[props(default)]
        node: Option<Callback<CascaderNodeArgs<T>, Element>>,
        /// One column's minimum width; `"max-content"` fits the longest row. Defaults to the theme's.
        #[props(default)]
        column_width: Option<String>,
        /// What the search box says while empty.
        #[props(default)]
        search_placeholder: Option<String>,
        /// Row height and font size.
        #[props(default, into)]
        size: Input<Size>,
        /// The box's corner radius: a size word or any CSS.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// The trigger opens nothing, and an open list closes.
        #[props(default)]
        disabled: Option<bool>,
        /// The trigger, carrying `state.a11y_attributes()`: a `button`, so Enter and Space click it.
        /// `sx` and co. style the dropdown box.
        children: Element,
    }
}

/// The cascade of a [`use_cascader`], dropping from any trigger: a round icon button, a chip, a row.
///
/// The keyboard is `Cascader`'s, on the trigger: arrows walk the tree, Enter commits, Escape closes.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, CascaderDropdown, CascaderOption, use_cascader};
/// # fn app() -> Element {
/// let mut ore = use_signal(|| None::<String>);
/// let cascader = use_cascader(
///     vec![CascaderOption::new("t1", "Tier 1").children(vec![
///         CascaderOption::new("iron", "Iron ore"),
///     ])],
///     ore(),
///     move |next| ore.set(next),
/// );
/// rsx! {
///     CascaderDropdown { state: cascader.clone(), searchable: true,
///         Button { attributes: cascader.a11y_attributes(), aria_label: "Ore", "⛏" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/cascader>
#[component]
pub fn CascaderDropdown<T: Options>(props: CascaderDropdownProps<T>) -> Element {
    let theme = use_theme();
    let state = props.state.clone();
    let combobox = state.combobox;
    let disabled = props.disabled.unwrap_or(false);
    let opened = combobox.is_open() && !disabled;
    let searchable = props.searchable.unwrap_or(false) && !disabled;
    let size = props.size.copied_or(theme.cascader.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.cascader.radius);
    let trigger_id = trigger_id(&combobox.id());

    // Found by id, as `Menu`'s wrapper: the trigger is the caller's element.
    let trigger = use_element();
    let selector = format!("[id={}]", css_string(&trigger_id));
    use_effect(use_reactive!(|selector, opened| {
        let found = mounted_by_selector(&selector);
        if opened && found.is_none() {
            warn(
                "CascaderDropdown: open, but no element carries `state.a11y_attributes()`, so the list hangs from nothing.",
            );
        }
        trigger.point_at(found);
    }));
    // Disabled closes the list; re-enabling does not reopen it.
    use_effect(use_reactive!(|(disabled, opened)| {
        let _ = opened;
        if disabled && combobox.is_open() {
            combobox.close();
        }
    }));

    let data = state.data.clone();
    let onchange = state.onchange;
    let onpick = {
        let data = data.clone();
        use_callback(move |next: Option<Vec<usize>>| {
            let value = next.and_then(|indices| {
                options_at(&data, &indices)
                    .last()
                    .map(|option| option.value.clone())
            });
            onchange.call(value);
        })
    };
    let committed = state
        .value
        .as_ref()
        .and_then(|value| indices_for_value(&data, value));
    let config = CascaderConfig {
        options: Rc::new(erase(&data)),
        committed,
        onpick,
        node: props.node.map(|draw| erase_node(data.clone(), draw)),
        filter: props
            .filter
            .map(|filter| erase_filter(data.clone(), filter)),
        separator: props
            .separator
            .clone()
            .unwrap_or_else(|| String::from(" / ")),
        any_level: props.any_level.unwrap_or(false),
        allow_deselect: props.allow_deselect.unwrap_or(false),
        layout: props.layout.copied_or_default(),
        column_width: props
            .column_width
            .clone()
            .unwrap_or_else(|| theme.cascader.column_width.to_string()),
        search_placeholder: props.search_placeholder.clone(),
    };

    let Engine {
        keys,
        open,
        controlled_id,
        descendant,
        empty,
        wrapper: _,
        clearance,
        status,
    } = use_cascader_engine(EngineSetup {
        config: &config,
        names: Naming {
            trigger_id: trigger_id.clone(),
            // The trigger's own name labels the listboxes and the search box.
            label_id: Some(trigger_id.clone()),
            aria_label: None,
            describedby: None,
            invalid: false,
            required: false,
        },
        state: combobox,
        trigger,
        anchor: trigger,
        field: None,
        opened,
        searchable,
        inert: disabled,
        size,
        radius: radius.size,
        parts: &props.parts,
        class: Some(&props.class),
        sx: Some(&props.sx),
        attributes: props.attributes,
    });

    let snapshot = TriggerAria {
        searchable,
        disabled,
        empty,
        controls: controlled_id,
        descendant,
    };
    let mut aria = state.aria;
    use_effect(use_reactive!(|snapshot| {
        if *aria.peek() != snapshot {
            aria.set(snapshot);
        }
    }));

    // The wrapper draws no box, so the sheet's clearance lands on the caller's trigger (todo 3914).
    let clearance_sx: Input<Sx> = sx()
        .selector(
            format!("& {selector}"),
            sx().with("scroll-margin-bottom", format!("{clearance}px"))
                .with(SCROLL_MARGIN_BOTTOM_VAR, format!("{clearance}px")),
        )
        .into();
    let wrapper = use_box()
        .framework_sx(&CASCADER_WRAPPER_SX)
        .sx(&clearance_sx)
        .prepare();
    let toggle = open.clone();
    rsx! {
        {
            wrapper
                // The trigger's click bubbles here; on a `button`, so do Enter and Space.
                .event("onclick", move |_: MouseEvent| {
                    if !disabled {
                        toggle(!combobox.is_open());
                    }
                })
                // Keeps the open search box focused, so the click reads the list open and closes it.
                .event("onmousedown", move |event: MouseEvent| {
                    if searchable && combobox.is_open() {
                        event.prevent_default();
                    }
                })
                .event("onkeydown", move |event: KeyboardEvent| {
                    // A closed list opens on the trigger's own click, or it would toggle twice.
                    let activates = matches!(event.key(), Key::Named(NamedKey::Enter))
                        || matches!(event.key(), Key::Character(ref key) if key == " ");
                    if activates && !combobox.is_open() && !has_shortcut_modifier(&event) {
                        return;
                    }
                    keys.handle(event);
                })
                // While searchable, focus moves to the search box, whose blur closes instead.
                .event("onfocusout", move |event: FocusEvent| {
                    if !searchable && blur_counts(&event) {
                        combobox.close();
                    }
                })
                .render(HtmlTag::Div, Vec::new(), props.children)
        }
        {status}
    }
}
