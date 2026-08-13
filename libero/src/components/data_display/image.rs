use dioxus::prelude::*;

use crate::{
    SxLayer,
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::{Sx, StaticSx, ThemeAwareValue, sx},
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
    fit: Input<ImageFit>,
    #[props(default, into)]
    radius: Input<ThemeAwareValue>,
    #[props(default, into)]
    alt: String,
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let mut errored_src = use_signal(|| None::<String>);

    let show_fallback = errored_src.read().as_deref() == Some(props.src.as_str());
    let src = if show_fallback {
        props.fallback_src.clone().unwrap_or_else(|| props.src.clone())
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
    let class = class_list([props.class, framework_class, dynamic_class, static_class]);

    let on_error_src = props.src.clone();

    // Empty/missing alt means decorative: reinforce that for assistive tech
    // rather than relying solely on implicit alt="" semantics.
    let decorative_role = props.alt.is_empty().then_some("presentation");

    rsx! {
        img {
            class: class,
            "data-state": data_state,
            src: src,
            alt: props.alt.clone(),
            role: decorative_role,
            onerror: move |_| errored_src.set(Some(on_error_src.clone())),
            ..props.attributes,
        }
    }
}
