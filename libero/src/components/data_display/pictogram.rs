use dioxus::prelude::*;

use crate::components::common::draw_svg;

/// An svg glyph as data: `viewBox`, root attributes and body, all `'static`.
/// pictogram's own type, so `pictogram_icons_lucide::house::outlined` (or any
/// pictogram set) passes straight in; `SvgData::new(include_str!("x.svg"))` for your own.
pub use pictogram_core::Svg as SvgData;

#[derive(Props, Clone, PartialEq)]
pub struct PictogramProps {
    /// The glyph to draw.
    pub icon: SvgData,
    /// Names the glyph: `role="img"` instead of `aria-hidden`. Leave unset next to a text label.
    #[props(default, into)]
    pub aria_label: Option<String>,
    /// Svg attributes, winning over the icon's own: `stroke_width: "3"` replaces its attribute.
    /// No size by default: the host's CSS (`Icon`, `ActionIcon`) sizes it, or set `width`/`height`.
    #[props(extends = SvgAttributes)]
    pub attributes: Vec<Attribute>,
}

/// An inline `<svg>` drawn from [`SvgData`], in `currentColor`; decorative unless given `aria_label`.
/// Other aria attributes go by name in quotes: `"aria-labelledby": "title"`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Pictogram;
/// # fn app() -> Element {
/// rsx! {
///     Pictogram { icon: pictogram_icons_lucide::house::outlined, width: "24px", height: "24px" }
///     Pictogram { icon: pictogram_icons_lucide::check::outlined, stroke_width: "3" }
///     Pictogram { icon: pictogram_icons_lucide::house::outlined, aria_label: "Home" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/pictogram>
#[component]
pub fn Pictogram(props: PictogramProps) -> Element {
    let mut attributes = props.attributes;
    // An empty name would make an unnamed `role="img"`, so it stays decorative.
    if let Some(label) = props.aria_label.filter(|label| !label.is_empty()) {
        attributes.push(Attribute::new("aria-label", label, None, false));
    }
    draw_svg(&props.icon, attributes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STROKED: SvgData = SvgData::new(
        r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 1"/></svg>"#,
    );

    #[test]
    fn it_renders_the_icon_body_inside_an_svg() {
        let html = dioxus_ssr::render_element(rsx! {
            Pictogram { icon: STROKED, "stroke-width": "3", width: "24px" }
        });
        assert_eq!(
            html,
            r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="3" width="24px"><path d="M1 1"/></svg>"#
        );
    }

    #[test]
    fn svg_props_set_attributes_not_css() {
        let html = dioxus_ssr::render_element(rsx! {
            Pictogram { icon: STROKED, stroke_width: "3", width: "24px", class: "x", "aria-label": "Home" }
        });
        assert_eq!(
            html,
            r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-label="Home" class="x" fill="none" role="img" stroke="currentColor" stroke-width="3" width="24px"><path d="M1 1"/></svg>"#
        );
    }

    #[test]
    fn brand_marks_render_without_a_title() {
        let html = dioxus_ssr::render_element(rsx! {
            Pictogram { icon: pictogram_icons_simple::github::regular }
            Pictogram { icon: pictogram_icons_lobe::openai::mono }
        });
        assert!(!html.contains("<title"), "{html}");
        assert!(html.contains("<path"), "{html}");
    }

    #[test]
    fn an_empty_aria_label_stays_decorative() {
        let html = dioxus_ssr::render_element(rsx! {
            Pictogram { icon: STROKED, aria_label: "" }
        });
        assert!(html.contains(r#"aria-hidden="true""#), "{html}");
        assert!(!html.contains("role="), "{html}");
    }

    #[test]
    fn aria_label_names_it_an_image() {
        let html = dioxus_ssr::render_element(rsx! {
            Pictogram { icon: STROKED, aria_label: "Home" }
        });
        assert!(html.contains(r#"role="img""#), "{html}");
        assert!(html.contains(r#"aria-label="Home""#), "{html}");
        assert!(!html.contains("aria-hidden"), "{html}");
    }
}
