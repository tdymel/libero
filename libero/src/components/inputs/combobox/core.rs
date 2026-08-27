use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{COMBOBOX_PADDING, ComboboxDefaults, Size, Z_INDEX_FLOAT},
};

use super::{dropdown::ComboboxDropdown, option::ComboboxContext};

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
        .padding(COMBOBOX_PADDING)
        // A row still has square-ish corners next to an `xxl` radius, so the
        // dropdown clips rather than trusting them to nest.
        .overflow("hidden")
        .background("white")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.3")
        .box_shadow("0 4px 8px rgba(0, 0, 0, 0.10), 0 8px 20px rgba(0, 0, 0, 0.14)")
});

base_props! {
    pub(super) struct ComboboxCoreProps {
        /// The rows, already drawn - which is what erases the caller's `T`,
        /// and what stops anything below here from memoizing.
        rows: Vec<Element>,
        active: usize,
        onactive: EventHandler<usize>,
        opened: bool,
        onopened: EventHandler<bool>,
        /// The `ComboboxState`'s id, which the aria wiring is built from.
        id: String,
        #[props(default)]
        empty: Option<Element>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        radius: Input<Size>,
        disabled: bool,
        children: Element,
    }
}

#[component]
pub(super) fn ComboboxCore(props: ComboboxCoreProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.combobox.size);
    let radius = props.radius.copied_or(theme.combobox.radius);

    // Written during render rather than in an effect: a row rendered this pass
    // must already read these this pass, and a provider only runs once.
    let id = use_signal(|| props.id.clone());
    let mut shared_size = use_signal(|| size);
    if *shared_size.peek() != size {
        shared_size.set(size);
    }
    let mut shared_radius = use_signal(|| radius);
    if *shared_radius.peek() != radius {
        shared_radius.set(radius);
    }
    let active_pick = use_signal(|| None);
    use_context_provider(|| ComboboxContext {
        id,
        size: shared_size,
        radius: shared_radius,
        active_pick,
    });

    let opened = props.opened;
    let count = props.rows.len();
    let active_row = props.active.min(count.saturating_sub(1));
    let set_active = props.onactive;

    let disabled = props.disabled;
    let onopened = props.onopened;
    let request = move |next: bool| onopened.call(next);

    let onkeydown = move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        let last = count.saturating_sub(1);
        let go_to = |row: usize| {
            set_active.call(row);
            if !opened {
                request(true);
            }
        };

        match event.key() {
            Key::ArrowDown => {
                event.prevent_default();
                match opened {
                    true => go_to((active_row + 1).min(last)),
                    false => go_to(active_row),
                }
            }
            Key::ArrowUp if opened => {
                event.prevent_default();
                go_to(active_row.saturating_sub(1));
            }
            Key::Home if opened => {
                event.prevent_default();
                go_to(0);
            }
            Key::End if opened => {
                event.prevent_default();
                go_to(last);
            }
            Key::Enter if opened && count > 0 => {
                event.prevent_default();
                if let Some(pick) = active_pick() {
                    pick.call(());
                }
                request(false);
            }
            Key::Escape if opened => {
                event.prevent_default();
                request(false);
            }
            Key::Tab if opened => request(false),
            _ => {}
        }
    };

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

    // A linear map of the active row across the scroll range. It costs no
    // measurement and still always lands the row inside the viewport - the
    // row's offset from the top works out to `row * (viewport - row height) /
    // (rows - 1)`, which never exceeds the viewport.
    let scroll_y = (count > 1).then(|| active_row as f64 / (count - 1) as f64 * 100.0);

    // An open list with nothing to show and no `empty` renders nothing at all -
    // an empty bordered box is not a state worth drawing, and it is what would
    // otherwise force callers to derive `opened` from the option count.
    let showing = opened && (count > 0 || props.empty.is_some());
    let dropdown = showing.then(|| {
        dropdown.render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                ComboboxDropdown {
                    rows: props.rows,
                    active: active_row,
                    id: id(),
                    max_height: theme.combobox.max_dropdown_height,
                    scroll_y,
                    empty: props.empty,
                }
            },
        )
    });

    wrapper
        // The trigger is the caller's, so the keys are caught where they
        // bubble to rather than on a field this component owns.
        .event("onkeydown", onkeydown)
        .render(
            HtmlTag::Div,
            Vec::new(),
            vec![props.children, dropdown.unwrap_or_else(|| rsx! {})],
        )
}
