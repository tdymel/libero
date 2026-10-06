use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            ClassList, HtmlTag, Input, Part, SVG_FIT, States, Variables, base_props, css_string,
            focus_ring_sx, input_from_str, parts_enum, parts_under_sx, states, svg_fit, svg_fit_sx,
            svg_fit_variables, variables,
        },
        layout::use_box,
    },
    hooks::{LightboxItem, LightboxOptions, use_lightbox, use_localization, use_theme},
    localization::{ImageLabels, fill},
    platform,
    str_enum::str_enum,
    sx::{StaticSx, Sx, sx},
    theme::{CssVar, IMAGE_RADIUS, ImageDefaults, Size, SizeCss},
    utils::warn,
};

pub use crate::theme::ImageFit;

input_from_str!(ImageFit);

str_enum! {
    /// The `<img>`'s `loading`: `Lazy` defers an off-screen picture.
    pub enum ImageLoading {
        Lazy = "lazy",
        #[default]
        Eager = "eager",
    }
}

input_from_str!(ImageLoading);

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
        .when(SVG_FIT, svg_fit_sx())
        // After `SVG_FIT`: its `!important` background yields to this list.
        .when(
            FALLBACK_UNDER,
            sx().background_image(format!("{} !important", FALLBACK_LAYERS.value()))
                .background_size(format!("{} !important", FALLBACK_SIZE.value()))
                .background_position("center !important")
                .background_repeat("no-repeat !important"),
        )
});

/// `fallback_src` painted as the `<img>`'s background, for a renderer that
/// fires no `error`: it shows where the picture fails to load (todo 935).
const FALLBACK_UNDER: &str = "fallback-under";
const FALLBACK_LAYERS: CssVar = CssVar::new("--lsx-image-fallback-layers");
const FALLBACK_SIZE: CssVar = CssVar::new("--lsx-image-fallback-size");

/// The background layers: an SVG `src` already drawn as the background
/// ([`svg_fit`]) stays on top of the fallback.
fn fallback_under_variables(
    variables: Variables,
    src: &str,
    fallback: &str,
    fit: ImageFit,
) -> Variables {
    let fallback = format!("url({})", css_string(fallback));
    let layers = match svg_fit(src) {
        true => format!("url({}), {fallback}", css_string(src)),
        false => fallback,
    };
    variables
        .with(FALLBACK_LAYERS, layers)
        .with(FALLBACK_SIZE, background_size(fit).to_string())
}

/// `fit` as the `background-size` an SVG is drawn in natively. `scale-down` is
/// `contain` there: a background cannot stop at its natural size.
fn background_size(fit: ImageFit) -> &'static str {
    match fit {
        ImageFit::Fill => "100% 100%",
        ImageFit::Contain | ImageFit::ScaleDown => "contain",
        ImageFit::Cover => "cover",
        ImageFit::None => "auto",
    }
}

static ZOOM_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .padding("0")
        .border_width("0")
        .background("transparent")
        .outline("none")
        // A fallback's text keeps the page's font: a `<button>` inherits neither.
        .font_family("inherit")
        .letter_spacing("inherit")
        // The picture's radius, so the focus ring and a caller's background follow it.
        .border_radius(IMAGE_RADIUS.overridable())
        .cursor("zoom-in")
        // A disabled `Fieldset` disables the `<button>` (todo 514).
        .selector("&:disabled", sx().opacity("0.5").cursor("not-allowed"))
        .focus_visible(focus_ring_sx())
});

parts_enum! {
    /// [`Image`]'s inner part, with `zoomable` only: `sx` then styles the zoom button.
    pub enum ImagePart {
        /// The `<img>` inside the zoom button.
        Image = "image" => "& > [data-slot='image']",
    }
}

base_props! {
    parts(ImagePart);
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
        /// What the picture shows. `None` warns in debug builds unless `decorative`.
        #[props(default, into)]
        alt: Option<String>,
        /// Marks the picture as decoration: `alt=""` and `role="presentation"`.
        #[props(default)]
        decorative: bool,
        #[props(default)]
        zoomable: bool,
        /// The `<img>`'s `loading` attribute.
        #[props(default, into)]
        loading: Input<ImageLoading>,
    }
}

/// On a zoomable image these go to the `<img>`, not the `<button>` around it.
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

fn zoom_label(alt: &str, labels: &ImageLabels) -> String {
    match alt.is_empty() {
        true => labels.zoom.to_string(),
        false => fill(labels.zoom_named, &[("alt", &alt)]),
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

/// A picture with a fallback source, optional zoom into a lightbox, and a themed radius.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Image;
/// # fn app() -> Element {
/// rsx! {
///     Image { src: "/photo.jpg", alt: "A harbour at dusk", fit: "cover", zoomable: true }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/image>
#[component]
pub fn Image(props: ImageProps) -> Element {
    let mut errored_src = use_signal(|| None::<String>);
    let in_link = use_hook(|| try_consume_context::<InLink>().is_some());
    // Once per mount, not per render.
    use_hook(|| {
        match (props.alt.as_deref(), props.decorative) {
            (None, false) => warn(
                "Image: no `alt` - describe the picture, or set `decorative` when it carries \
                 nothing.",
            ),
            (Some(alt), true) if !alt.is_empty() => {
                warn("Image: `decorative` wins over `alt` - the picture renders alt=\"\".")
            }
            _ => {}
        }
        let named = !props.decorative && props.alt.as_deref().is_some_and(|alt| !alt.is_empty());
        if props.zoomable && !in_link && !named {
            warn(
                "Image: a zoomable picture needs an `alt` - the zoom button and the dialog are \
                 named from it.",
            );
        }
        if props.zoomable && in_link {
            warn(
                "Image: zoomable is ignored inside a linked ImageItem - the link wins, and a \
                 zoom button would nest a <button> in the <a>.",
            );
        }
    });
    let zoomable = props.zoomable && !in_link;

    // An empty fallback is no fallback, on every platform.
    let fallback_src = props
        .fallback_src
        .as_deref()
        .filter(|fallback| !fallback.is_empty());
    let show_fallback = errored_src.read().as_deref() == Some(props.src.as_str());
    let src = match fallback_src {
        Some(fallback) if show_fallback => fallback.to_string(),
        _ => props.src.clone(),
    };

    let fit = props.fit.copied_or(use_theme().image.fit);
    let fallback_under = fallback_src.filter(|_| !platform::fires_image_errors());
    let mut variables = svg_fit_variables(
        image_variables(props.radius.as_ref().copied()),
        &src,
        background_size(fit),
    );
    if let Some(fallback) = fallback_under {
        variables = fallback_under_variables(variables, &src, fallback, fit);
    }
    let variables: Input<Variables> = variables.into();

    let on_error_src = props.src.clone();

    // No `alt` at all is left for a checker to flag, not made decorative.
    let alt = match props.decorative {
        true => Some(String::new()),
        false => props.alt.clone(),
    };
    let decorative_role = (alt.as_deref() == Some("")).then_some("presentation");

    // One `use_box` on both paths, so `zoomable` can flip without moving a hook.
    // When zoomable, `ZoomButton` takes the caller's styling instead.
    let no_class = Input::None;
    let no_sx = Input::None;
    let (class, user_sx, img_states) = match zoomable {
        true => (&no_class, &no_sx, states()),
        false => (
            &props.class,
            &props.sx,
            props.states.clone().unwrap_or_default(),
        ),
    };
    let img_states: Input<States> = img_states
        .with(fit.state_name(), true)
        .with(SVG_FIT, svg_fit(&src))
        .with(FALLBACK_UNDER, fallback_under.is_some())
        .into();
    let image = use_box()
        .framework_sx(&IMAGE_BASE_SX)
        .class(class)
        .sx(user_sx)
        .states(&img_states)
        .variables(&variables)
        .prepare();
    let image = image
        .event("onerror", move |_: Event<ImageData>| {
            errored_src.set(Some(on_error_src.clone()))
        })
        .attr(
            "loading",
            props.loading.as_ref().map(|loading| loading.as_str()),
        );

    if !zoomable {
        return image
            .attr("src", src)
            .attr("alt", alt)
            .attr("role", decorative_role)
            .render(HtmlTag::Img, props.attributes, ());
    }
    let item = LightboxItem::new(
        props.zoomed_src.clone().unwrap_or_else(|| src.clone()),
        alt.clone().unwrap_or_default(),
    );
    let (img_attributes, attributes): (Vec<_>, Vec<_>) = props
        .attributes
        .into_iter()
        .partition(|attribute| IMG_ATTRIBUTES.contains(&attribute.name));
    let image = image
        .attr("src", src)
        .attr("alt", "")
        .attr("role", "presentation")
        .attr("data-slot", ImagePart::Image.slot())
        .render(HtmlTag::Img, img_attributes, ());

    rsx! {
        ZoomButton {
            item,
            alt: alt.unwrap_or_default(),
            variables: image_variables(props.radius.as_ref().copied()),
            class: props.class,
            sx: parts_under_sx(&props.parts, props.sx),
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
    variables: Variables,
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
    let variables: Input<Variables> = props.variables.clone().into();
    let root = use_box()
        .framework_sx(&ZOOM_BUTTON_SX)
        .variables(&variables)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare();

    let label = zoom_label(&props.alt, &use_localization().image);
    let item = props.item.clone();
    // A dialog opener, not a toggle: the open state lives in the `Lightbox`.
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

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            crate::components::common::part_table::<ImagePart>(),
            [("image", "& > [data-slot='image']")]
        );
    }

    #[test]
    fn no_radius_emits_no_variable() {
        assert_eq!(image_variables(None).to_string(), "");
    }

    use crate::utils::warnings_of;

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

    /// Todo 602: a forgotten `alt` warns; an explicit one or `decorative` does not.
    #[test]
    fn a_missing_alt_warns_unless_decorative() {
        let missing = warnings_of(|| rsx! { crate::LiberoProvider { Image { src: "/a.svg" } } });
        assert!(
            missing.iter().any(|w| w.starts_with("Image: no `alt`")),
            "{missing:?}"
        );

        let decorative = warnings_of(|| {
            rsx! { crate::LiberoProvider { Image { src: "/a.svg", decorative: true } } }
        });
        assert!(decorative.is_empty(), "{decorative:?}");

        let empty =
            warnings_of(|| rsx! { crate::LiberoProvider { Image { src: "/a.svg", alt: "" } } });
        assert!(empty.is_empty(), "{empty:?}");

        let both = warnings_of(|| {
            rsx! { crate::LiberoProvider { Image { src: "/a.svg", alt: "A", decorative: true } } }
        });
        assert!(
            both.iter().any(|w| w.contains("`decorative` wins")),
            "{both:?}"
        );
    }

    /// Todo 2526: the zoom button and its dialog are named from the `alt`.
    #[test]
    fn a_zoomable_image_without_an_alt_warns() {
        let zoomable = |app: fn() -> Element| {
            warnings_of(app)
                .iter()
                .any(|w| w.starts_with("Image: a zoomable picture"))
        };
        assert!(zoomable(|| rsx! {
            crate::LiberoProvider { Image { src: "/a.svg", alt: "", zoomable: true } }
        }));
        assert!(zoomable(|| rsx! {
            crate::LiberoProvider { Image { src: "/a.svg", decorative: true, zoomable: true } }
        }));
        assert!(zoomable(
            || rsx! { crate::LiberoProvider { Image { src: "/a.svg", zoomable: true } } }
        ));
        assert!(!zoomable(|| rsx! {
            crate::LiberoProvider { Image { src: "/a.svg", alt: "A", zoomable: true } }
        }));
    }
}
