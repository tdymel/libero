use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States, Variables,
        common::{base_props, focus_ring_sx, input_from_str, states, variables},
        layout::use_box,
    },
    hooks::{LightboxItem, LightboxOptions, use_lightbox, use_theme},
    sx::{StaticSx, Sx, sx},
    theme::{IMAGE_RADIUS, ImageDefaults, Size, SizeCss},
    utils::warn,
};

pub use crate::theme::ImageFit;

input_from_str!(ImageFit);

fn image_variables(radius: Option<Size>) -> Variables {
    variables().with(
        IMAGE_RADIUS.override_var(),
        radius.map(|radius| SizeCss::RADIUS.value(radius)),
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
        // A disabled `Fieldset` disables the `<button>` (todo 514).
        .selector("&:disabled", sx().opacity("0.5").cursor("not-allowed"))
        .focus_visible(focus_ring_sx())
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
        radius: Input<Size>,
        #[props(default, into)]
        alt: String,
        #[props(default)]
        zoomable: bool,
    }
}

/// What describes the picture rather than the root: on a zoomable image these
/// go to the `<img>`, not to the `<button>` around it.
const IMG_ATTRIBUTES: [&str; 8] = [
    "loading",
    "decoding",
    "fetchpriority",
    "srcset",
    "sizes",
    "crossorigin",
    "referrerpolicy",
    "usemap",
];

fn zoom_label(alt: &str) -> String {
    match alt.is_empty() {
        true => "Zoom in".to_string(),
        false => format!("Zoom in: {alt}"),
    }
}

#[derive(Clone, Copy)]
struct InLink;

/// Marks its children as inside a link, e.g. a linked `ImageItem`: an `Image`
/// there renders no zoom button, which would nest a `<button>` in the `<a>`.
#[component]
pub(crate) fn LinkedImageScope(children: Element) -> Element {
    use_context_provider(|| InLink);
    children
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let mut errored_src = use_signal(|| None::<String>);
    let in_link = use_hook(|| try_consume_context::<InLink>().is_some());
    // Once per mount, not per render.
    use_hook(|| {
        if props.zoomable && in_link {
            warn(
                "Image: zoomable is ignored inside a linked ImageItem - the link wins, and a \
                 zoom button would nest a <button> in the <a>.",
            );
        }
    });
    let zoomable = props.zoomable && !in_link;

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
    let variables: Input<Variables> = image_variables(props.radius.as_ref().copied()).into();

    let on_error_src = props.src.clone();

    let decorative_role = props.alt.is_empty().then_some("presentation");

    // One `use_box` on both paths, so `zoomable` can flip without moving a
    // hook: the `<img>` is the root when it can't zoom, and the picture inside
    // `ZoomButton`, which takes the caller's styling, when it can.
    let no_class = Input::None;
    let no_sx = Input::None;
    let (class, user_sx, img_states): (_, _, Input<States>) = match zoomable {
        true => (
            &no_class,
            &no_sx,
            states().with(fit.state_name(), true).into(),
        ),
        false => (
            &props.class,
            &props.sx,
            props
                .states
                .clone()
                .unwrap_or_default()
                .with(fit.state_name(), true)
                .into(),
        ),
    };
    let image = use_box()
        .framework_sx(&IMAGE_BASE_SX)
        .class(class)
        .sx(user_sx)
        .states(&img_states)
        .variables(&variables)
        .prepare();
    let image = image.event("onerror", move |_: Event<ImageData>| {
        errored_src.set(Some(on_error_src.clone()))
    });

    if !zoomable {
        return image
            .attr("src", src)
            .attr("alt", props.alt)
            .attr("role", decorative_role)
            .render(HtmlTag::Img, props.attributes, ());
    }
    let item = LightboxItem::new(
        props.zoomed_src.clone().unwrap_or_else(|| src.clone()),
        props.alt.clone(),
    );
    let (img_attributes, attributes): (Vec<_>, Vec<_>) = props
        .attributes
        .into_iter()
        .partition(|attribute| IMG_ATTRIBUTES.contains(&attribute.name));
    let image = image
        .attr("src", src)
        .attr("alt", "")
        .attr("role", "presentation")
        .render(HtmlTag::Img, img_attributes, ());

    rsx! {
        ZoomButton {
            item,
            alt: props.alt,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes,
            {image}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ZoomButtonProps {
    item: LightboxItem,
    alt: String,
    class: Input<ClassList>,
    sx: Input<Sx>,
    states: Input<States>,
    attributes: Vec<Attribute>,
    children: Element,
}

/// A zoomable `Image`'s root, split out so only a zoomable one pays for
/// `use_lightbox` and its modal registration.
#[component]
fn ZoomButton(props: ZoomButtonProps) -> Element {
    let zoom = use_lightbox(LightboxOptions {
        thumbnails: false,
        captions: false,
        controls: false,
        aria_label: (!props.alt.is_empty()).then(|| props.alt.clone()),
        ..LightboxOptions::default()
    });
    let root = use_box()
        .framework_sx(&ZOOM_BUTTON_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare();

    let label = zoom_label(&props.alt);
    let item = props.item.clone();
    // A dialog opener, not a toggle: the open state lives in the modal
    // `Lightbox`, never on this button.
    root.attr("type", "button")
        .attr("aria-haspopup", "dialog")
        .attr("aria-label", label)
        .event("onclick", move |_: Event<MouseData>| {
            zoom.open_with(item.clone());
        })
        .render(HtmlTag::Button, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_radius_resolves_through_the_radius_scale() {
        let variables = image_variables(Some(Size::Md));

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

    fn warnings_of(app: fn() -> Element) -> Vec<String> {
        crate::utils::take_warnings();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        crate::utils::take_warnings()
    }

    #[test]
    fn a_zoomable_image_in_a_link_warns() {
        let linked = warnings_of(|| {
            rsx! {
                crate::LiberoProvider {
                    LinkedImageScope { Image { src: "/a.svg", alt: "A", zoomable: true } }
                }
            }
        });
        assert!(linked.iter().any(|w| w.starts_with("Image:")), "{linked:?}");

        let alone = warnings_of(|| {
            rsx! {
                crate::LiberoProvider { Image { src: "/a.svg", alt: "A", zoomable: true } }
            }
        });
        assert!(!alone.iter().any(|w| w.starts_with("Image:")), "{alone:?}");
    }
}
