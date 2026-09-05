use dioxus::prelude::*;

use dioxus::core::AttributeValue;

use crate::components::Input;

macro_rules! html_tags {
    (
        default { $($dvariant:ident => $dtag:ident),* $(,)? }
        full { $($fvariant:ident => $ftag:ident),* $(,)? }
    ) => {
        /// Always the full HTML5 element set - `full-polymorphism` controls
        /// how many of `render_polymorphic`'s arms compile, not this type. So
        /// a `full`-tier variant always compiles; without the feature it just
        /// renders as `<div>`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum HtmlTag {
            $($dvariant,)*
            $($fvariant,)*
        }

        impl Default for HtmlTag {
            fn default() -> Self {
                Self::Div
            }
        }

        impl HtmlTag {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$dvariant => stringify!($dtag),)*
                    $(Self::$fvariant => stringify!($ftag),)*
                }
            }
        }

        impl From<&str> for HtmlTag {
            fn from(value: &str) -> Self {
                match value.to_lowercase().as_str() {
                    $(stringify!($dtag) => Self::$dvariant,)*
                    $(stringify!($ftag) => Self::$fvariant,)*
                    // No accepted-value list: 111 tags is not useful output.
                    _ => {
                        crate::utils::warn(&format!(
                            "HtmlTag: unrecognized tag {value:?}, rendering as <div>."
                        ));
                        Self::Div
                    }
                }
            }
        }

        impl From<String> for HtmlTag {
            fn from(value: String) -> Self {
                Self::from(value.as_str())
            }
        }

        impl From<HtmlTag> for Input<HtmlTag> {
            fn from(value: HtmlTag) -> Self {
                Input::Value(value)
            }
        }

        impl From<&str> for Input<HtmlTag> {
            fn from(value: &str) -> Self {
                Input::Value(HtmlTag::from(value))
            }
        }

        impl From<String> for Input<HtmlTag> {
            fn from(value: String) -> Self {
                Input::Value(HtmlTag::from(value))
            }
        }

        /// Renders on whichever element `component` selects. `attributes`
        /// carries event handlers too - `extends = GlobalAttributes` captures
        /// them generically, so each arm only has to spread it.
        ///
        /// Only the `default` tier's arms compile unconditionally; the rest
        /// need `full-polymorphism` and fall back to `<div>` without it. That
        /// is what keeps a consumer using a handful of tags from paying for
        /// all 111.
        pub(crate) fn render_polymorphic(
            component: HtmlTag,
            class: String,
            data_state: Option<String>,
            style: Option<String>,
            attributes: Vec<Attribute>,
            children: Vec<Element>,
        ) -> Element {
            let attributes = styling_attributes(Some(class), data_state, style, attributes);
            match component {
                $(HtmlTag::$dvariant => rsx! {
                    $dtag { ..attributes, {children.into_iter()} }
                },)*
                $(
                    #[cfg(feature = "full-polymorphism")]
                    HtmlTag::$fvariant => rsx! {
                        $ftag { ..attributes, {children.into_iter()} }
                    },
                )*
                // The value is fine, the build configuration is not - a
                // different fix from a typo, so a different message.
                #[cfg(not(feature = "full-polymorphism"))]
                _ => {
                    crate::utils::warn(&format!(
                        "HtmlTag::{component:?} (<{}>) needs the \"full-polymorphism\" \
                         feature, rendering as <div>.",
                        component.as_str()
                    ));
                    rsx! { div { ..attributes, {children.into_iter()} } }
                },
            }
        }
    };
}

/// What a component hands to `render` as its children.
///
/// Also normalizes a `children` prop: only the fork's `rsx!` can split a
/// caller's children into a `Vec<Element>`, so a component that wants one
/// entry per child declares the field twice under `cfg` and reads it through
/// here. Against upstream main it sees a single merged node.
///
/// Anything but a single `Element` splices its nodes straight into the
/// element's template. Wrapping them in one more `Element` - which is what an
/// `rsx! {}` block at the call site does - costs a dynamic node every render,
/// and an empty one costs it for nothing.
pub(crate) trait IntoChildren {
    fn into_children(self) -> Vec<Element>;
}

/// No children at all.
impl IntoChildren for () {
    fn into_children(self) -> Vec<Element> {
        Vec::new()
    }
}

impl IntoChildren for Element {
    fn into_children(self) -> Vec<Element> {
        vec![self]
    }
}

impl IntoChildren for Option<Element> {
    fn into_children(self) -> Vec<Element> {
        self.into_iter().collect()
    }
}

impl IntoChildren for Vec<Element> {
    fn into_children(self) -> Vec<Element> {
        self
    }
}

/// Merges the styling triple into a caller's attributes, so the element carries
/// one spread instead of three dynamic slots. A slot is diffed every render even
/// when its value is `None`; a `Vec` entry that is not there costs nothing.
///
/// A caller's own `class`/`style` is merged with ours rather than emitted twice:
/// a duplicate attribute silently drops one of the two, and which one depends on
/// the renderer (SSR keeps the first, the DOM the last). Ours go first, so a
/// caller's declarations win the cascade.
///
/// `class` is `None` for a caller that carries the class itself - the router
/// `Link`, which renders its own `class` slot. A caller's `class` attribute is
/// then left where it is, since there is nothing here to merge it into.
///
/// `aria-describedby` is merged for the same reason and by the same rule: it
/// is a list of ids, so racing the caller's against the component's drops one
/// of them - which took a field's validation message away from AT whenever the
/// caller pointed at a hint of their own. The first occurrence keeps its
/// place, and the component's attributes are appended after the caller's, so
/// the caller's ids come first.
pub(crate) fn styling_attributes(
    class: Option<String>,
    data_state: Option<String>,
    style: Option<String>,
    mut attributes: Vec<Attribute>,
) -> Vec<Attribute> {
    let mut class = class;
    let mut style = style;
    // Everything after the first `aria-describedby`, to append to it.
    let mut described: Option<String> = None;
    let mut describes = false;

    attributes.retain_mut(|attribute| match (attribute.name, &attribute.value) {
        ("class", AttributeValue::Text(value)) => match &mut class {
            Some(class) => {
                join(class, value, ' ');
                false
            }
            None => true,
        },
        ("style", AttributeValue::Text(value)) => {
            match &mut style {
                Some(style) => join(style, value, ';'),
                None => style = Some(value.clone()),
            }
            false
        }
        ("aria-describedby", AttributeValue::Text(value)) if describes => {
            match &mut described {
                Some(described) => join(described, value, ' '),
                None => described = Some(value.clone()),
            }
            false
        }
        ("aria-describedby", AttributeValue::Text(_)) => {
            describes = true;
            true
        }
        _ => true,
    });

    if let Some(rest) = described
        && let Some(first) = attributes
            .iter_mut()
            .find(|attribute| attribute.name == "aria-describedby")
        && let AttributeValue::Text(value) = &mut first.value
    {
        join(value, &rest, ' ');
    }

    // Pushed then rotated to the front: prepending in place is a memmove,
    // where `splice` at index 0 is a generic reallocating path.
    let mut ours = 0;
    if let Some(class) = class {
        attributes.push(super::attr("class", class));
        ours += 1;
    }
    if let Some(data_state) = data_state {
        attributes.push(super::attr("data-state", data_state));
        ours += 1;
    }
    if let Some(style) = style {
        attributes.push(super::attr("style", style));
        ours += 1;
    }
    attributes.rotate_right(ours);
    attributes
}

/// Appends `value`, inserting `separator` only where one is missing.
fn join(into: &mut String, value: &str, separator: char) {
    if value.is_empty() {
        return;
    }
    if !into.is_empty() && !into.trim_end().ends_with(separator) {
        into.push(separator);
    }
    into.push_str(value);
}

// The `default` tier is every tag a page is written out of: the sectioning,
// text-level, list, table and form elements, plus every tag `libero` and
// `docs` render themselves. It compiles unconditionally, so `Box`'s promise
// that it "renders as any tag via `component`" holds on default features for
// the tags a caller actually reaches for.
//
// `full` is what is left: document metadata (`head`, `meta`, `title`,
// `script`, `style`, `link`, `base`, `body`, `noscript`), embedded and media
// content (`iframe`, `canvas`, `audio`, `video`, `picture`, `source`,
// `track`, `embed`, `object`, `param`, `map`, `area`), web components
// (`template`, `slot`) and the bidi/ruby annotation set (`bdi`, `bdo`,
// `ruby`, `rp`, `rt`). None of them is something a component library styles,
// and each still type-checks without the feature - it renders as `<div>` and
// `warn()`s. A new tag never grows this split, it only picks a side.
html_tags! {
    default {
        A => a,
        Abbr => abbr,
        Address => address,
        Article => article,
        Aside => aside,
        B => b,
        Blockquote => blockquote,
        Br => br,
        Button => button,
        Caption => caption,
        Cite => cite,
        Code => code,
        Col => col,
        Colgroup => colgroup,
        Data => data,
        Datalist => datalist,
        Dd => dd,
        Del => del,
        Details => details,
        Dfn => dfn,
        Dialog => dialog,
        Div => div,
        Dl => dl,
        Dt => dt,
        Em => em,
        Fieldset => fieldset,
        Figcaption => figcaption,
        Figure => figure,
        Footer => footer,
        Form => form,
        H1 => h1,
        H2 => h2,
        H3 => h3,
        H4 => h4,
        H5 => h5,
        H6 => h6,
        Header => header,
        Hgroup => hgroup,
        Hr => hr,
        I => i,
        Img => img,
        Input => input,
        Ins => ins,
        Kbd => kbd,
        Label => label,
        Legend => legend,
        Li => li,
        Main => main,
        Mark => mark,
        Menu => menu,
        Meter => meter,
        Nav => nav,
        Ol => ol,
        Optgroup => optgroup,
        Option => option,
        Output => output,
        P => p,
        Pre => pre,
        Progress => progress,
        Q => q,
        S => s,
        Samp => samp,
        Section => section,
        Select => select,
        Small => small,
        Span => span,
        Strong => strong,
        Sub => sub,
        Summary => summary,
        Sup => sup,
        Table => table,
        Tbody => tbody,
        Td => td,
        Textarea => textarea,
        Tfoot => tfoot,
        Th => th,
        Thead => thead,
        Time => time,
        Tr => tr,
        U => u,
        Ul => ul,
        Var => var,
        Wbr => wbr,
    }
    full {
        Area => area,
        Audio => audio,
        Base => base,
        Bdi => bdi,
        Bdo => bdo,
        Body => body,
        Canvas => canvas,
        Embed => embed,
        Head => head,
        Iframe => iframe,
        Link => link,
        Map => map,
        Meta => meta,
        Noscript => noscript,
        Object => object,
        Param => param,
        Picture => picture,
        Rp => rp,
        Rt => rt,
        Ruby => ruby,
        Script => script,
        Slot => slot,
        Source => source,
        Style => style,
        Template => template,
        Title => title,
        Track => track,
        Video => video,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(name: &'static str, value: &str) -> Attribute {
        super::super::attr(name, value.to_string())
    }

    fn named(attributes: &[Attribute]) -> Vec<(&str, String)> {
        attributes
            .iter()
            .map(|attribute| match &attribute.value {
                AttributeValue::Text(value) => (attribute.name, value.clone()),
                _ => (attribute.name, String::new()),
            })
            .collect()
    }

    #[test]
    fn ours_come_first_then_the_callers() {
        let out = styling_attributes(
            Some("ours".to_string()),
            Some("size-md".into()),
            None,
            vec![text("id", "x"), text("aria-label", "y")],
        );

        assert_eq!(
            named(&out),
            [
                ("class", "ours".into()),
                ("data-state", "size-md".into()),
                ("id", "x".into()),
                ("aria-label", "y".into()),
            ]
        );
    }

    #[test]
    fn a_callers_class_joins_ours_instead_of_duplicating() {
        let out = styling_attributes(
            Some("ours".to_string()),
            None,
            None,
            vec![text("class", "theirs")],
        );

        assert_eq!(named(&out), [("class", "ours theirs".into())]);
    }

    #[test]
    fn a_callers_style_joins_ours_and_wins_the_cascade() {
        let out = styling_attributes(
            Some(String::new()),
            None,
            Some("color:red;".into()),
            vec![text("style", "color:blue;")],
        );

        assert_eq!(
            named(&out),
            [
                ("class", String::new()),
                ("style", "color:red;color:blue;".into())
            ]
        );
    }

    #[test]
    fn a_missing_semicolon_is_inserted_between_the_two() {
        let out = styling_attributes(
            Some(String::new()),
            None,
            Some("color:red".into()),
            vec![text("style", "color:blue")],
        );

        assert_eq!(named(&out)[1], ("style", "color:red;color:blue".into()));
    }

    #[test]
    fn a_callers_style_survives_when_we_have_none() {
        let out = styling_attributes(
            Some(String::new()),
            None,
            None,
            vec![text("style", "color:blue;")],
        );

        assert_eq!(named(&out)[1], ("style", "color:blue;".into()));
    }

    /// The `Link` path: the class stays a prop, so a caller's `class`
    /// attribute is left alone rather than merged into nothing.
    #[test]
    fn without_a_class_of_ours_the_callers_is_untouched() {
        let out = styling_attributes(
            None,
            Some("size-md".into()),
            Some("color:red;".into()),
            vec![text("class", "theirs"), text("style", "color:blue;")],
        );

        assert_eq!(
            named(&out),
            [
                ("data-state", "size-md".into()),
                ("style", "color:red;color:blue;".into()),
                ("class", "theirs".into()),
            ]
        );
    }
}
