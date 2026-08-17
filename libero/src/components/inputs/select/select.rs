use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States,
        common::{base_props, class_list, focus_ring_sx},
    },
    hooks::use_theme,
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
        .focus_visible(focus_ring_sx())
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

base_props! {
    pub struct SelectProps {
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
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    let theme = use_theme();
    let id = use_signal(|| format!("lsx-select-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));

    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.select.size,
    };
    let size_class = crate::hooks::use_css(get_size_sx(size), crate::CssLayer::Framework);

    let dynamic_sx = sx().apply_if(props.radius.as_ref(), |sx, radius| {
        sx.border_radius(radius.clone())
    });
    let dynamic_class = crate::hooks::use_css(&dynamic_sx, crate::CssLayer::UserDynamic);

    // `class`/`sx` land on the wrapper (the element that actually participates
    // in a parent flex/grid layout - e.g. `margin-left: auto`), not on the
    // inner select, which only ever carries its own visual chrome.
    let select_class = class_list().with(size_class).with(dynamic_class);

    // `<option>`s come from `props.children` - a dynamic node mounted in the
    // same pass as `<select>` itself, so on the *creating* render there's no
    // matching option yet for the browser to select against `value` (it
    // silently falls back to the first option instead). Rendering no
    // `value` attribute at all on that first render, then flipping
    // `mounted` true in `use_effect` (which only runs once the whole
    // subtree - options included - is actually committed), defers the real
    // value application to a subsequent *update*, where the plain `value:`
    // attribute path already works correctly (dioxus-web's own
    // `set_attribute.js` sets `.value` as a DOM property, not just an
    // attribute - it just needs the options to already exist, which they do
    // by then).
    let mut mounted = use_signal(|| false);
    use_effect(move || mounted.set(true));
    let value = mounted().then(|| props.value.clone());

    rsx! {
        Box {
            component: "label",
            class: props.class,
            sx: props.sx,
            framework_sx: &SELECT_WRAPPER_SX,
            if let Some(label) = &props.label {
                Box { component: "span", framework_sx: &SELECT_LABEL_SX, {label.clone()} }
            }
            Box {
                component: "select",
                id: "{id}",
                class: select_class,
                states: props.states,
                framework_sx: &SELECT_BASE_SX,
                value: value,
                onchange: move |event: FormEvent| props.onchange.call(event.value()),
                attributes: props.attributes,
                {props.children}
            }
        }
    }
}
