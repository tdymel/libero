use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, OptionLabel, States,
        common::{attr, base_props},
        inputs::TextField,
        layout::use_box,
    },
    hooks::{use_element, use_id, use_theme},
    platform::ElementApi,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ComboboxDefaults, Size, Z_INDEX_FLOAT},
};

use super::{dropdown::ComboboxDropdown, target::ComboboxTarget};

/// Not user-facing - it exists so the dropdown has something to be absolute
/// against, which is why `sx` lands on the dropdown instead.
static COMBOBOX_WRAPPER_SX: StaticSx = StaticSx::new(|| sx().position("relative"));

static COMBOBOX_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    ComboboxDefaults::dropdown_theme_vars()
        .position("absolute")
        .top("100%")
        .left("0")
        .width("100%")
        .margin_top("4px")
        .z_index(Z_INDEX_FLOAT.value())
        .display("flex")
        .flex_direction("column")
        .gap("4px")
        .padding("4px")
        .background("white")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.3")
        .box_shadow("0 4px 8px rgba(0, 0, 0, 0.10), 0 8px 20px rgba(0, 0, 0, 0.14)")
});

base_props! {
    pub(super) struct ComboboxCoreProps {
        /// Every option, already resolved through `option_label`.
        labels: Vec<OptionLabel>,
        /// Indices of the selected options - a set, so multi-select later
        /// changes nothing here.
        selected: HashSet<usize>,
        onpick: EventHandler<usize>,
        onclear: EventHandler<()>,
        /// `(query, option index) -> keep`. Closes over the caller's `T`, so
        /// nothing below this point is generic.
        matches: Callback<(String, usize), bool>,
        target: Callback<ComboboxTarget, Element>,
        searchable: bool,
        #[props(default)]
        search_placeholder: Option<String>,
        #[props(default)]
        empty: Option<Element>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        radius: Input<Size>,
        /// Overrides the themed row height that virtualization assumes.
        #[props(default)]
        option_height: Option<f64>,
        #[props(default, into)]
        max_dropdown_height: Input<ThemeAwareValue>,
        disabled: bool,
    }
}

#[component]
pub(super) fn ComboboxCore(props: ComboboxCoreProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.combobox.size);
    let radius = props.radius.copied_or(theme.combobox.radius);

    let id = use_id();
    let root = use_element();
    let mut opened = use_signal(|| false);
    let mut search = use_signal(String::new);
    // A position in `visible`, not an option index: the row under the cursor
    // is what the arrows move, and filtering renumbers them.
    let mut active = use_signal(|| 0usize);

    let count = props.labels.len();
    let query = search();
    let visible: Vec<usize> = if props.searchable && !query.is_empty() {
        (0..count)
            .filter(|index| props.matches.call((query.clone(), *index)))
            .collect()
    } else {
        (0..count).collect()
    };
    let active_row = active().min(visible.len().saturating_sub(1));

    let disabled = props.disabled;
    let onpick = props.onpick;

    // The target is the only element carrying `aria-haspopup`, so it is
    // findable without asking the caller to wire a handle back to us.
    let focus_target = move || {
        if let Ok(element) = root.query_selector("[aria-haspopup='listbox']") {
            let _ = element.focus();
        }
    };

    // Plain closures over `Copy` signals, not `use_callback`: focusing the
    // target blurs the search field *synchronously*, so `onblur` re-enters
    // this before it has returned - which a `Callback`'s borrow cannot
    // survive. The guard makes that second pass a no-op.
    let close = move |()| {
        // Re-copied so the closure stays `Fn`, and so `Copy`: it is handed to
        // several handlers at once.
        let (mut opened, mut search, mut active) = (opened, search, active);
        if !opened() {
            return;
        }
        opened.set(false);
        search.set(String::new());
        active.set(0);
        focus_target();
    };

    let visible_for_key = visible.clone();
    let handle_key = use_callback(move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        let is_open = opened();
        let last = visible_for_key.len().saturating_sub(1);
        let mut go_to = |row: usize| {
            active.set(row);
            if !is_open {
                opened.set(true);
            }
        };

        match event.key() {
            Key::ArrowDown => {
                event.prevent_default();
                if is_open {
                    go_to((active_row + 1).min(last));
                } else {
                    go_to(active_row);
                }
            }
            Key::ArrowUp if is_open => {
                event.prevent_default();
                go_to(active_row.saturating_sub(1));
            }
            Key::Home if is_open => {
                event.prevent_default();
                go_to(0);
            }
            Key::End if is_open => {
                event.prevent_default();
                go_to(last);
            }
            Key::Enter => {
                event.prevent_default();
                match is_open {
                    true => {
                        if let Some(&index) = visible_for_key.get(active_row) {
                            onpick.call(index);
                            close(());
                        }
                    }
                    false => go_to(active_row),
                }
            }
            // Space opens, but only from the target - inside the search field
            // it is a character like any other.
            Key::Character(ref character) if character == " " && !is_open => {
                event.prevent_default();
                go_to(active_row);
            }
            Key::Escape if is_open => {
                event.prevent_default();
                close(());
            }
            Key::Tab if is_open => close(()),
            _ => {}
        }
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("disabled", disabled)
        .into();

    // Both are hooks, so both run before anything branches on `opened`.
    let wrapper = use_box().framework_sx(&COMBOBOX_WRAPPER_SX).prepare();
    let dropdown = use_box()
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare();

    let listbox_id = format!("{}-listbox", id());
    let aria = vec![
        attr("id", id()),
        attr("aria-haspopup", "listbox"),
        attr("aria-expanded", opened().to_string()),
        attr("aria-controls", listbox_id),
    ];

    let onclear = props.onclear;
    let target = props.target.call(ComboboxTarget {
        labels: props
            .selected
            .iter()
            .filter_map(|index| props.labels.get(*index).cloned())
            .collect(),
        opened: opened(),
        disabled,
        aria,
        onclick: EventHandler::new(move |_: MouseEvent| {
            if disabled {
                return;
            }
            match opened() {
                true => close(()),
                false => opened.set(true),
            }
        }),
        onkeydown: EventHandler::new(move |event: KeyboardEvent| handle_key.call(event)),
        onclear: EventHandler::new(move |_: MouseEvent| onclear.call(())),
    });

    // A linear map of the active row across the scroll range. It costs no
    // measurement and still always lands the row inside the viewport - the
    // row's offset from the top works out to `row * (viewport - row height) /
    // (rows - 1)`, which never exceeds the viewport.
    let scroll_y =
        (visible.len() > 1).then(|| active_row as f64 / (visible.len() - 1) as f64 * 100.0);

    let max_height = props
        .max_dropdown_height
        .resolve(None)
        .unwrap_or_else(|| theme.combobox.max_dropdown_height.to_string());

    let search_placeholder = props.search_placeholder.clone();
    let searchable = props.searchable;
    let dropdown = opened().then(|| {
        dropdown.render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                if searchable {
                    TextField {
                        size,
                        value: search(),
                        placeholder: search_placeholder,
                        onchange: move |next| {
                            search.set(next);
                            active.set(0);
                        },
                        onkeydown: move |event| handle_key.call(event),
                        onblur: move |_| close(()),
                        onmounted: move |event: Event<MountedData>| {
                            spawn(async move {
                                let _ = event.data().set_focus(true).await;
                            });
                        },
                        "role": "combobox",
                        "aria-autocomplete": "list",
                        "aria-controls": format!("{}-listbox", id()),
                        "aria-activedescendant": visible
                            .get(active_row)
                            .map(|index| format!("{}-option-{index}", id())),
                    }
                }
                ComboboxDropdown {
                    labels: props.labels,
                    visible,
                    selected: props.selected,
                    active: active_row,
                    id: id(),
                    size,
                    item_size: props
                        .option_height
                        .unwrap_or_else(|| theme.combobox.row_height(size)),
                    max_height,
                    scroll_y,
                    onpick: move |index| {
                        onpick.call(index);
                        close(());
                    },
                    empty: props.empty,
                }
            },
        )
    });

    wrapper.element(&root).render(
        HtmlTag::Div,
        Vec::new(),
        vec![target, dropdown.unwrap_or_else(|| rsx! {})],
    )
}
