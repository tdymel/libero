use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{Box, Dialog, Input, Modal, States, common::class_list},
    hooks::use_css,
    hooks::{use_focus_return, use_portal},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFit {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

impl Default for ImageFit {
    fn default() -> Self {
        Self::Cover
    }
}

impl ImageFit {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Fill => "fill",
            Self::Contain => "contain",
            Self::Cover => "cover",
            Self::None => "none",
            Self::ScaleDown => "scale-down",
        }
    }
}

impl From<&str> for ImageFit {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "fill" => Self::Fill,
            "contain" => Self::Contain,
            "none" => Self::None,
            "scale-down" | "scaledown" => Self::ScaleDown,
            _ => Self::Cover,
        }
    }
}

impl From<String> for ImageFit {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<ImageFit> {
    fn from(value: &str) -> Self {
        Input::Value(ImageFit::from(value))
    }
}

impl From<String> for Input<ImageFit> {
    fn from(value: String) -> Self {
        Input::Value(ImageFit::from(value))
    }
}

static IMAGE_BASE_SX: StaticSx =
    StaticSx::new(|| sx().display("block").width("100%").height("100%"));

static ZOOM_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .padding("0")
        .border_width("0")
        .background("transparent")
        .outline("none")
        .focus_visible(
            sx().outline("2px solid var(--lsx-primary-6)")
                .outline_offset("2px"),
        )
});

static ZOOM_OVERLAY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .position("relative")
        .padding("0")
        .border_width("0")
        .background("transparent")
        .outline("none")
        .focus_visible(
            sx().outline("2px solid var(--lsx-primary-6)")
                .outline_offset("2px"),
        )
});

static ZOOM_OVERLAY_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("auto")
        .height("auto")
        .max_width("90vw")
        .max_height("90vh")
        .object_fit("contain")
});

#[derive(Props, Clone, PartialEq)]
pub struct ImageProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(into)]
    src: String,
    #[props(default)]
    fallback_src: Option<String>,
    #[props(default, into)]
    zoomed_src: Option<String>,
    #[props(default, into)]
    fit: Input<ImageFit>,
    #[props(default, into)]
    radius: Input<ThemeAwareValue>,
    #[props(default, into)]
    alt: String,
    #[props(default)]
    zoomable: bool,
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let mut errored_src = use_signal(|| None::<String>);
    let mut zoomed = use_signal(|| false);

    let show_fallback = errored_src.read().as_deref() == Some(props.src.as_str());
    let src = if show_fallback {
        props
            .fallback_src
            .clone()
            .unwrap_or_else(|| props.src.clone())
    } else {
        props.src.clone()
    };

    let fit = props.fit.as_ref().copied().unwrap_or_default();
    let dynamic_sx = sx()
        .object_fit(fit.as_str())
        .apply_if(props.radius.as_ref(), |sx, radius| {
            sx.border_radius(radius.clone())
        });

    let framework_class = use_css(&IMAGE_BASE_SX, CssLayer::Framework);
    let dynamic_class = use_css(&dynamic_sx, CssLayer::UserDynamic);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_css(sx, CssLayer::UserStatic));

    let on_error_src = props.src.clone();

    let decorative_role = props.alt.is_empty().then_some("presentation");

    if !props.zoomable {
        let class = class_list([props.class, framework_class, dynamic_class, static_class]);
        return rsx! {
            Box {
                component: "img",
                class: class,
                states: props.states,
                src: src,
                alt: props.alt,
                role: decorative_role,
                onerror: move |_| errored_src.set(Some(on_error_src.clone())),
                attributes: props.attributes,
            }
        };
    }

    let img_class = class_list([framework_class, dynamic_class]);
    let zoomed_src = props.zoomed_src.clone().unwrap_or_else(|| src.clone());

    let mut focus_return = use_focus_return();
    let mut close_zoom = move || {
        zoomed.set(false);
        focus_return.restore();
    };

    let cursor = if zoomed() { "zoom-out" } else { "zoom-in" };
    let button_framework_class = use_css(&ZOOM_BUTTON_SX, CssLayer::Framework);
    let button_dynamic_class = use_css(&sx().cursor(cursor), CssLayer::UserDynamic);
    let button_class = class_list([
        props.class,
        button_framework_class,
        button_dynamic_class,
        static_class,
    ]);

    let label = match (props.alt.is_empty(), zoomed()) {
        (true, false) => "Zoom in".to_string(),
        (true, true) => "Zoom out".to_string(),
        (false, false) => format!("Zoom in: {}", props.alt),
        (false, true) => format!("Zoom out: {}", props.alt),
    };

    let overlay_button_framework_class = use_css(&ZOOM_OVERLAY_BUTTON_SX, CssLayer::Framework);
    let overlay_button_cursor_class = use_css(&sx().cursor("zoom-out"), CssLayer::UserDynamic);
    let overlay_button_class =
        class_list([overlay_button_framework_class, overlay_button_cursor_class]);
    let overlay_image_class = use_css(&ZOOM_OVERLAY_IMAGE_SX, CssLayer::Framework);

    let portal_label = label.clone();
    use_portal(move || {
        if !zoomed() {
            return None;
        }
        let label = portal_label.clone();

        Some(rsx! {
            Modal {
                onclose: move |_| close_zoom(),
                Dialog {
                    aria_label: label.clone(),
                    size: "none",
                    sx: sx()
                        .background("transparent")
                        .width("auto")
                        .padding("0")
                        .box_shadow("none"),
                    Box {
                        component: "button",
                        r#type: "button",
                        class: overlay_button_class.clone(),
                        "data-autofocus": true,
                        aria_label: label.clone(),
                        onclick: move |_| close_zoom(),
                        Box {
                            component: "img",
                            class: overlay_image_class.clone(),
                            src: zoomed_src.clone(),
                            alt: "",
                            role: "presentation",
                        }
                    }
                }
            }
        })
    });

    rsx! {
        Box {
            component: "button",
            r#type: "button",
            class: button_class,
            states: props.states,
            "aria-pressed": zoomed().to_string(),
            aria_label: label.clone(),
            onmounted: move |event: Event<MountedData>| focus_return.remember(event),
            onclick: move |_| zoomed.toggle(),
            Box {
                component: "img",
                class: img_class,
                src: src.clone(),
                alt: "",
                role: "presentation",
                onerror: move |_| errored_src.set(Some(on_error_src.clone())),
            }
        }
    }
}
