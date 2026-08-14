use dioxus::prelude::*;

use crate::{
    SxLayer,
    components::{FocusTrap, Input, Overlay, States, common::class_list},
    context::use_sx,
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

const ZOOM_DIALOG_Z_INDEX: &str = "100";

static ZOOM_DIALOG_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .z_index(ZOOM_DIALOG_Z_INDEX)
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

    let framework_class = use_sx(&IMAGE_BASE_SX, SxLayer::Framework);
    let dynamic_class = use_sx(&dynamic_sx, SxLayer::UserDynamic);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));

    let data_state = props.states.as_ref().and_then(States::data_state);

    let on_error_src = props.src.clone();

    let decorative_role = props.alt.is_empty().then_some("presentation");

    if !props.zoomable {
        let class = class_list([props.class, framework_class, dynamic_class, static_class]);
        return rsx! {
            img {
                class: class,
                "data-state": data_state,
                src: src,
                alt: props.alt.clone(),
                role: decorative_role,
                onerror: move |_| errored_src.set(Some(on_error_src.clone())),
                ..props.attributes,
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
    let button_framework_class = use_sx(&ZOOM_BUTTON_SX, SxLayer::Framework);
    let button_dynamic_class = use_sx(&sx().cursor(cursor), SxLayer::UserDynamic);
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

    let overlay_button_framework_class = use_sx(&ZOOM_OVERLAY_BUTTON_SX, SxLayer::Framework);
    let overlay_button_cursor_class = use_sx(&sx().cursor("zoom-out"), SxLayer::UserDynamic);
    let overlay_button_class = class_list([
        overlay_button_framework_class,
        overlay_button_cursor_class,
    ]);
    let overlay_image_class = use_sx(&ZOOM_OVERLAY_IMAGE_SX, SxLayer::Framework);
    let dialog_class = use_sx(&ZOOM_DIALOG_SX, SxLayer::Framework);

    let portal_label = label.clone();
    use_portal(move || {
        if !zoomed() {
            return rsx! {};
        }

        let label = portal_label.clone();

        rsx! {
            div {
                role: "dialog",
                "aria-modal": "true",
                "aria-label": label.clone(),
                class: dialog_class.clone(),
                onkeydown: move |event: Event<KeyboardData>| {
                    if event.key() == Key::Escape {
                        close_zoom();
                    }
                },
                Overlay { onclick: move |_| close_zoom() }
                FocusTrap {
                    active: zoomed(),
                    button {
                        r#type: "button",
                        class: overlay_button_class.clone(),
                        "data-autofocus": true,
                        aria_label: label.clone(),
                        onclick: move |_| close_zoom(),
                        img {
                            class: overlay_image_class.clone(),
                            src: zoomed_src.clone(),
                            alt: "",
                            role: "presentation",
                        }
                    }
                }
            }
        }
    });

    rsx! {
        button {
            r#type: "button",
            class: button_class,
            "data-state": data_state,
            "aria-pressed": zoomed().to_string(),
            aria_label: label.clone(),
            onmounted: move |event: Event<MountedData>| focus_return.remember(event),
            onclick: move |_| zoomed.toggle(),
            img {
                class: img_class,
                src: src.clone(),
                alt: "",
                role: "presentation",
                onerror: move |_| errored_src.set(Some(on_error_src.clone())),
            }
        }
    }
}
