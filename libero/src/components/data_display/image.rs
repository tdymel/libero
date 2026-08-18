use dioxus::prelude::*;

use crate::{
    components::{
        Box, Dialog, Input, Modal, States, Variables,
        common::{base_props, focus_ring_sx, states, variables},
    },
    hooks::{use_focus_return, use_portal},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, SizeCss},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageFit {
    Fill,
    Contain,
    #[default]
    Cover,
    None,
    ScaleDown,
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

fn fit_token(fit: ImageFit) -> &'static str {
    match fit {
        ImageFit::Fill => "fit-fill",
        ImageFit::Contain => "fit-contain",
        ImageFit::Cover => "fit-cover",
        ImageFit::None => "fit-none",
        ImageFit::ScaleDown => "fit-scale-down",
    }
}

const IMAGE_RADIUS_VAR: CssVar = CssVar::new("--lsx-image-radius");

fn image_variables(radius: Option<&ThemeAwareValue>) -> Variables {
    variables().with(
        IMAGE_RADIUS_VAR,
        radius.and_then(|v| v.resolve(Some(SizeCss::RADIUS))),
    )
}

static IMAGE_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .border_radius(IMAGE_RADIUS_VAR.value_or(0))
        .when("fit-fill", sx().object_fit("fill"))
        .when("fit-contain", sx().object_fit("contain"))
        .when("fit-cover", sx().object_fit("cover"))
        .when("fit-none", sx().object_fit("none"))
        .when("fit-scale-down", sx().object_fit("scale-down"))
});

static ZOOM_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .padding("0")
        .border_width("0")
        .background("transparent")
        .outline("none")
        .cursor("zoom-in")
        .when("zoomed", sx().cursor("zoom-out"))
        .focus_visible(focus_ring_sx())
});

// Only ever rendered while `zoomed` is true (it's the button inside the
// zoomed overlay itself), so its cursor is a fixed "zoom-out" - not a
// per-instance dynamic value.
static ZOOM_OVERLAY_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .position("relative")
        .padding("0")
        .border_width("0")
        .background("transparent")
        .outline("none")
        .cursor("zoom-out")
        .focus_visible(focus_ring_sx())
});

static ZOOM_OVERLAY_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("auto")
        .height("auto")
        .max_width("90vw")
        .max_height("90vh")
        .object_fit("contain")
});

base_props! {
    pub struct ImageProps {
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
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let mut errored_src = use_signal(|| None::<String>);
    let mut zoomed = use_signal(|| false);
    // Hoisted above the non-zoomable early return, and paired with the
    // `use_portal(None)` below it: hook slots are positional, so both paths
    // have to reach the same hooks in the same order.
    let mut focus_return = use_focus_return();

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
    let variables = image_variables(props.radius.as_ref());

    let on_error_src = props.src.clone();

    let decorative_role = props.alt.is_empty().then_some("presentation");

    if !props.zoomable {
        let states = props
            .states
            .as_ref()
            .cloned()
            .unwrap_or_default()
            .with(fit_token(fit), true);
        use_portal(None);
        return rsx! {
            Box {
                component: "img",
                class: props.class,
                sx: props.sx,
                states,
                variables,
                framework_sx: &IMAGE_BASE_SX,
                src: src,
                alt: props.alt,
                role: decorative_role,
                onerror: move |_| errored_src.set(Some(on_error_src.clone())),
                attributes: props.attributes,
            }
        };
    }

    let img_states = states().with(fit_token(fit), true);
    let zoomed_src = props.zoomed_src.clone().unwrap_or_else(|| src.clone());

    let mut close_zoom = move || {
        zoomed.set(false);
        focus_return.restore();
    };

    let button_states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("zoomed", zoomed());

    let label = match (props.alt.is_empty(), zoomed()) {
        (true, false) => "Zoom in".to_string(),
        (true, true) => "Zoom out".to_string(),
        (false, false) => format!("Zoom in: {}", props.alt),
        (false, true) => format!("Zoom out: {}", props.alt),
    };

    let portal_content = zoomed().then(|| {
        rsx! {
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
                        framework_sx: &ZOOM_OVERLAY_BUTTON_SX,
                        "data-autofocus": true,
                        aria_label: label.clone(),
                        onclick: move |_| close_zoom(),
                        Box {
                            component: "img",
                            framework_sx: &ZOOM_OVERLAY_IMAGE_SX,
                            src: zoomed_src.clone(),
                            alt: "",
                            role: "presentation",
                        }
                    }
                }
            }
        }
    });
    use_portal(portal_content);

    rsx! {
        Box {
            component: "button",
            r#type: "button",
            class: props.class,
            sx: props.sx,
            states: button_states,
            framework_sx: &ZOOM_BUTTON_SX,
            "aria-pressed": zoomed().to_string(),
            aria_label: label.clone(),
            onmounted: move |event: Event<MountedData>| focus_return.remember(event),
            onclick: move |_| zoomed.toggle(),
            Box {
                component: "img",
                states: img_states,
                variables,
                framework_sx: &IMAGE_BASE_SX,
                src: src.clone(),
                alt: "",
                role: "presentation",
                onerror: move |_| errored_src.set(Some(on_error_src.clone())),
            }
        }
    }
}
