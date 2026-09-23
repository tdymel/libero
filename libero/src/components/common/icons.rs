use dioxus::prelude::*;
use pictogram_core::Svg as SvgData;

use super::names_itself;
use crate::{context::IconSlot, hooks::use_icon};

const XMLNS: &str = "http://www.w3.org/2000/svg";

/// A glyph libero draws itself: the nearest `IconProvider`'s for `slot`, else `icon`.
/// Brand marks name a service, so they are no slot: pictogram's lobe and simple consts.
#[component]
pub(crate) fn Glyph(
    slot: IconSlot,
    icon: SvgData,
    #[props(extends = SvgAttributes)] attributes: Vec<Attribute>,
) -> Element {
    draw_svg(&use_icon(slot, icon), attributes)
}

/// `body` without a leading `<title>`: lobe and simple glyphs carry one, which adds
/// a hover tooltip and its words to the host's text. `aria_label` is the name.
fn untitled(body: &str) -> &str {
    body.strip_prefix("<title>")
        .and_then(|rest| rest.split_once("</title>"))
        .map_or(body, |(_, rest)| rest)
}

/// `icon` as an inline `<svg>`, `own` winning over its attributes. `Pictogram`'s body.
pub(crate) fn draw_svg(icon: &SvgData, own: Vec<Attribute>) -> Element {
    let attributes = attributes(icon, &[], own);
    rsx! {
        svg {
            view_box: icon.view_box,
            xmlns: XMLNS,
            dangerous_inner_html: untitled(icon.body),
            ..attributes,
        }
    }
}

/// Later wins by name: defaults < icon < provider < caller.
fn attributes(icon: &SvgData, provider: &[Attribute], own: Vec<Attribute>) -> Vec<Attribute> {
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
    fn untitled_drops_only_a_leading_title() {
        assert_eq!(
            untitled(r#"<title>GitHub</title><path d="M1 1"/>"#),
            r#"<path d="M1 1"/>"#
        );
        assert_eq!(untitled(STROKED.body), STROKED.body);
        assert_eq!(untitled("<title>open"), "<title>open");
        assert_eq!(
            untitled(r#"<path d="M1 1"/><title>late</title>"#),
            r#"<path d="M1 1"/><title>late</title>"#
        );
    }

    #[test]
    fn a_named_pictogram_is_an_image_not_hidden() {
        let named = attributes(&STROKED, &[], vec![attr("aria-label", "Home")]);
        assert_eq!(value(&named, "aria-hidden"), None);
        assert_eq!(value(&named, "role"), Some(&text("img")));
    }
}
