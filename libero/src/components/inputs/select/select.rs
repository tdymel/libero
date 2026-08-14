use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::{document, prelude::*};

use crate::{
    components::{Input, States, common::class_list},
    context::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{SelectDefaults, Size},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

static SELECT_WRAPPER_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").flex_direction("column").gap("4px"));

static SELECT_LABEL_SX: StaticSx = StaticSx::new(|| sx().font_size("0.75rem").color("grey.7"));

static SELECT_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.4")
        .background("white")
        .color("black")
        .cursor("pointer")
        .and(SelectDefaults::radius_sx())
        .hover(sx().border_color("grey.7"))
        .focus_visible(
            sx().outline("2px solid var(--lsx-primary-6)")
                .outline_offset("2px"),
        )
});

fn get_size_sx(size: Size) -> &'static Sx {
    static XS: StaticSx = StaticSx::new(SelectDefaults::xs_sx);
    static SM: StaticSx = StaticSx::new(SelectDefaults::sm_sx);
    static MD: StaticSx = StaticSx::new(SelectDefaults::md_sx);
    static LG: StaticSx = StaticSx::new(SelectDefaults::lg_sx);
    static XL: StaticSx = StaticSx::new(SelectDefaults::xl_sx);

    match size {
        Size::Xs => &XS,
        Size::Sm => &SM,
        Size::Md => &MD,
        Size::Lg => &LG,
        Size::Xl => &XL,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default, into)]
    radius: Input<ThemeAwareValue>,
    #[props(into)]
    value: String,
    #[props(default)]
    onchange: EventHandler<String>,
    #[props(default)]
    label: Option<String>,
    children: Element,
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    let theme = use_theme();
    let id = use_signal(|| format!("lsx-select-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));

    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.select.size,
    };
    let explicit_radius = match props.radius.as_ref() {
        Some(ThemeAwareValue::Size(size)) => Some(*size),
        _ => None,
    };

    let wrapper_class = crate::context::use_sx(&SELECT_WRAPPER_SX, crate::SxLayer::Framework);
    let label_class = crate::context::use_sx(&SELECT_LABEL_SX, crate::SxLayer::Framework);
    let size_class = crate::context::use_sx(get_size_sx(size), crate::SxLayer::Framework);
    let framework_class = crate::context::use_sx(&SELECT_BASE_SX, crate::SxLayer::Framework);

    let dynamic_sx = sx().apply_if(explicit_radius, |sx, radius| {
        sx.border_radius(ThemeAwareValue::Size(radius))
    });
    let dynamic_class = crate::context::use_sx(&dynamic_sx, crate::SxLayer::UserDynamic);

    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    // `class`/`sx` land on the wrapper (the element that actually participates
    // in a parent flex/grid layout - e.g. `margin-left: auto`), not on the
    // inner select, which only ever carries its own visual chrome.
    let wrapper_class = class_list([props.class, wrapper_class, static_class]);
    let select_class = class_list([framework_class, size_class, dynamic_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        label {
            class: wrapper_class,
            if let Some(label) = &props.label {
                span { class: label_class, {label.clone()} }
            }
            select {
                id: "{id}",
                class: select_class,
                value: props.value.clone(),
                "data-state": data_state,
                onchange: move |event| props.onchange.call(event.value()),
                onmounted: move |_| {
                    document::eval(&format!(
                        "var el = document.getElementById({:?}); if (el) el.value = {:?};",
                        id(),
                        props.value,
                    ));
                },
                ..props.attributes,
                {props.children}
            }
        }
    }
}
