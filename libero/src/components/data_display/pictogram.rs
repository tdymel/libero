use dioxus::prelude::*;

use crate::components::common::names_itself;

/// An svg glyph as data: `viewBox`, root attributes and body, all `'static`.
/// pictogram's own type, so `pictogram_icons_lucide::house::outlined` (or any
/// pictogram set) passes straight in; `SvgData::new(include_str!("x.svg"))` for your own.
pub use pictogram_core::Svg as SvgData;

const XMLNS: &str = "http://www.w3.org/2000/svg";

#[derive(Props, Clone, PartialEq)]
pub struct PictogramProps {
    /// The glyph to draw.
    pub icon: SvgData,
    /// Win over the icon's own: `"stroke-width": "3"` replaces its attribute. No size
    /// by default: the host's CSS (`Icon`, `ActionIcon`) sizes it, or set `width`/`height`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// An inline `<svg>` drawn from [`SvgData`], in `currentColor`; decorative unless given `aria_label`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Pictogram;
/// # fn app() -> Element {
/// rsx! {
///     Pictogram { icon: pictogram_icons_lucide::house::outlined, width: "24px", height: "24px" }
///     Pictogram { icon: pictogram_icons_lucide::check::outlined, "stroke-width": "3" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/pictogram>
#[component]
pub fn Pictogram(props: PictogramProps) -> Element {
    let attributes = attributes(&props.icon, &[], props.attributes);
    rsx! {
        svg {
            view_box: props.icon.view_box,
            xmlns: XMLNS,
            dangerous_inner_html: props.icon.body,
            ..attributes,
        }
    }
}

/// Later wins by name: defaults < icon < provider < caller.
pub(crate) fn attributes(
    icon: &SvgData,
    provider: &[Attribute],
    own: Vec<Attribute>,
) -> Vec<Attribute> {
    let named = names_itself(&own) || names_itself(provider);
    let mut defaults = vec![if named {
        Attribute::new("role", "img", None, false)
    } else {
        Attribute::new("aria-hidden", "true", None, false)
    }];
    if !icon.attributes().any(|(name, _)| name == "fill") {
        defaults.push(Attribute::new("fill", "currentColor", None, false));
    }
    let from_icon = icon
        .attributes()
        .map(|(name, value)| Attribute::new(name, value, None, false));
    merge(
        defaults
            .into_iter()
            .chain(from_icon)
            .chain(provider.iter().cloned())
            .chain(own),
    )
}

fn merge(attributes: impl IntoIterator<Item = Attribute>) -> Vec<Attribute> {
    let mut merged: Vec<Attribute> = Vec::new();
    for attribute in attributes {
        match merged
            .iter_mut()
            .find(|a| a.name == attribute.name && a.namespace == attribute.namespace)
        {
            Some(existing) => *existing = attribute,
            None => merged.push(attribute),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::AttributeValue;

    const STROKED: SvgData = SvgData::new(
        r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 1"/></svg>"#,
    );
    const UNFILLED: SvgData = SvgData::new(r#"<svg viewBox="0 0 16 16"><path d="M1 1"/></svg>"#);

    fn attr(name: &'static str, value: &'static str) -> Attribute {
        Attribute::new(name, value, None, false)
    }

    fn value<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a AttributeValue> {
        attributes.iter().find(|a| a.name == name).map(|a| &a.value)
    }

    fn text(value: &str) -> AttributeValue {
        AttributeValue::Text(value.into())
    }

    #[test]
    fn a_later_attribute_replaces_an_earlier_one_in_place() {
        let merged = merge([attr("fill", "a"), attr("stroke", "b"), attr("fill", "c")]);
        let names: Vec<_> = merged.iter().map(|a| a.name).collect();
        assert_eq!(names, ["fill", "stroke"]);
        assert_eq!(merged[0].value, text("c"));
    }

    #[test]
    fn the_namespace_is_part_of_an_attributes_identity() {
        let merged = merge([
            attr("fill", "a"),
            Attribute::new("fill", "b", Some("style"), false),
        ]);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn the_icons_attributes_beat_the_defaults() {
        let attributes = attributes(&STROKED, &[], vec![]);
        assert_eq!(value(&attributes, "fill"), Some(&text("none")));
        assert_eq!(value(&attributes, "stroke-width"), Some(&text("2")));
    }

    #[test]
    fn an_icon_without_a_fill_fills_in_current_color() {
        let attributes = attributes(&UNFILLED, &[], vec![]);
        assert_eq!(value(&attributes, "fill"), Some(&text("currentColor")));
    }

    #[test]
    fn the_provider_beats_the_icon_and_the_caller_beats_the_provider() {
        let provider = [attr("stroke-width", "1.5")];
        let attributes_of = |own| attributes(&STROKED, &provider, own);
        assert_eq!(
            value(&attributes_of(vec![]), "stroke-width"),
            Some(&text("1.5"))
        );
        assert_eq!(
            value(
                &attributes_of(vec![attr("stroke-width", "3")]),
                "stroke-width"
            ),
            Some(&text("3"))
        );
    }

    #[test]
    fn no_size_is_set_by_default() {
        let attributes = attributes(&STROKED, &[], vec![]);
        assert_eq!(value(&attributes, "width"), None);
        assert_eq!(value(&attributes, "height"), None);
    }

    #[test]
    fn it_is_hidden_by_default_and_the_caller_can_unhide_it() {
        let hidden = attributes(&STROKED, &[], vec![]);
        assert_eq!(value(&hidden, "aria-hidden"), Some(&text("true")));
        let shown = attributes(&STROKED, &[], vec![attr("aria-hidden", "false")]);
        assert_eq!(value(&shown, "aria-hidden"), Some(&text("false")));
    }

    #[test]
    fn it_renders_the_icon_body_inside_an_svg() {
        let html = dioxus_ssr::render_element(rsx! {
            Pictogram { icon: STROKED, "stroke-width": "3", width: "24px" }
        });
        assert_eq!(
            html,
            r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="3" style="width:24px;"><path d="M1 1"/></svg>"#
        );
    }

    #[test]
    fn a_named_pictogram_is_an_image_not_hidden() {
        let named = attributes(&STROKED, &[], vec![attr("aria-label", "Home")]);
        assert_eq!(value(&named, "aria-hidden"), None);
        assert_eq!(value(&named, "role"), Some(&text("img")));
    }
}
