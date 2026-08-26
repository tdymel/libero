use dioxus::prelude::*;

use crate::{
    components::{
        Box, Dialog, HtmlTag, Input, States, Variables,
        common::{base_props, focus_ring_sx, input_from_str, states, variables},
        layout::use_box,
    },
    hooks::{ModalHandle, ModalScope, use_modal, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{IMAGE_RADIUS, ImageDefaults, SizeCss},
};

pub use crate::theme::ImageFit;

input_from_str!(ImageFit);

fn image_variables(radius: Option<&ThemeAwareValue>) -> Variables {
    variables().with(
        IMAGE_RADIUS.override_var(),
        radius.and_then(|v| v.resolve(Some(SizeCss::RADIUS))),
    )
}

static IMAGE_BASE_SX: StaticSx = StaticSx::new(|| {
    ImageDefaults::theme_vars()
        .display("block")
        .width("100%")
        .height("100%")
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

// Only rendered while zoomed, so its cursor is a fixed "zoom-out".
static IMAGE_ZOOM_DIALOG_SX: StaticSx = StaticSx::new(|| {
    sx().background("transparent")
        .width("auto")
        .padding("0")
        .box_shadow("none")
});

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

fn zoom_label(alt: &str, zoomed: bool) -> String {
    match (alt.is_empty(), zoomed) {
        (true, false) => "Zoom in".to_string(),
        (true, true) => "Zoom out".to_string(),
        (false, false) => format!("Zoom in: {alt}"),
        (false, true) => format!("Zoom out: {alt}"),
    }
}

/// The zoom overlay as its own modal, the way any dialog is built on
/// [`use_modal`]: `alt` is shared by every opening, the argument is the source
/// to show enlarged.
fn use_zoom_modal(alt: String) -> ModalHandle<String> {
    use_modal(move |s: ModalScope<String>| {
        let label = zoom_label(&alt, true);

        rsx! {
            Dialog {
                aria_label: label.clone(),
                // The picture itself is the close button.
                close_button: false,
                size: "none",
                sx: &IMAGE_ZOOM_DIALOG_SX,
                Box {
                    component: "button",
                    r#type: "button",
                    framework_sx: &ZOOM_OVERLAY_BUTTON_SX,
                    "data-autofocus": true,
                    aria_label: label,
                    onclick: move |_| s.close(),
                    Box {
                        component: "img",
                        framework_sx: &ZOOM_OVERLAY_IMAGE_SX,
                        src: s.args(),
                        alt: "",
                        role: "presentation",
                    }
                }
            }
        }
    })
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let mut errored_src = use_signal(|| None::<String>);
    // Above the early return, and unconditional: hook slots are positional.
    let zoom = use_zoom_modal(props.alt.clone());

    let show_fallback = errored_src.read().as_deref() == Some(props.src.as_str());
    let src = if show_fallback {
        props
            .fallback_src
            .clone()
            .unwrap_or_else(|| props.src.clone())
    } else {
        props.src.clone()
    };

    let fit = props.fit.copied_or(use_theme().image.fit);
    let variables: Input<Variables> = image_variables(props.radius.as_ref()).into();

    let on_error_src = props.src.clone();

    let decorative_role = props.alt.is_empty().then_some("presentation");

    // Both paths' styling is resolved here, above the branch: `prepare` is a
    // hook. The root is the `<img>` itself when it can't zoom, and the zoom
    // `<button>` when it can, so the two differ in every input.
    let img_states: Input<States> = match props.zoomable {
        true => states().with(fit.state_name(), true).into(),
        false => props
            .states
            .clone()
            .unwrap_or_default()
            .with(fit.state_name(), true)
            .into(),
    };
    let button_states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with("zoomed", zoom.is_open())
        .into();

    let (root_sx, root_states, root_variables) = match props.zoomable {
        true => (&ZOOM_BUTTON_SX, &button_states, &Input::None),
        false => (&IMAGE_BASE_SX, &img_states, &variables),
    };
    let root = use_box()
        .framework_sx(root_sx)
        .class(&props.class)
        .sx(&props.sx)
        .states(root_states)
        .variables(root_variables)
        .prepare();
    // Only the zoomable path renders it, but the hook runs either way.
    let inner_image = use_box()
        .framework_sx(&IMAGE_BASE_SX)
        .states(&img_states)
        .variables(&variables)
        .prepare();

    if !props.zoomable {
        return root
            .attr("src", src)
            .attr("alt", props.alt)
            .attr("role", decorative_role)
            .event("onerror", move |_: Event<ImageData>| {
                errored_src.set(Some(on_error_src.clone()))
            })
            .render(HtmlTag::Img, props.attributes, ());
    }
    let zoomed_src = props.zoomed_src.clone().unwrap_or_else(|| src.clone());
    let label = zoom_label(&props.alt, zoom.is_open());

    let image = inner_image
        .attr("src", src.clone())
        .attr("alt", "")
        .attr("role", "presentation")
        .event("onerror", move |_: Event<ImageData>| {
            errored_src.set(Some(on_error_src.clone()))
        })
        .render(HtmlTag::Img, Vec::new(), ());

    root.attr("type", "button")
        .attr("aria-pressed", zoom.is_open().to_string())
        .attr("aria-label", label)
        .event("onclick", move |_: Event<MouseData>| {
            zoom.open_with(zoomed_src.clone());
        })
        .render(HtmlTag::Button, props.attributes, image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Size;

    #[test]
    fn a_size_radius_resolves_through_the_radius_scale() {
        let radius = ThemeAwareValue::Size(Size::Md);
        let variables = image_variables(Some(&radius));

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                IMAGE_RADIUS.override_var().name(),
                SizeCss::RADIUS.value(Size::Md)
            )
        );
    }

    #[test]
    fn no_radius_emits_no_variable() {
        assert_eq!(image_variables(None).to_string(), "");
    }
}
