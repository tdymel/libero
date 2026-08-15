use dioxus::prelude::*;

use crate::components::Input;

macro_rules! html_tags {
    ($($variant:ident => $tag:ident),* $(,)?) => {
        /// Root elements a polymorphic component can render as.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum HtmlTag {
            $($variant),*
        }

        impl Default for HtmlTag {
            fn default() -> Self {
                Self::Div
            }
        }

        impl HtmlTag {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($tag)),*
                }
            }
        }

        impl From<&str> for HtmlTag {
            fn from(value: &str) -> Self {
                match value.to_lowercase().as_str() {
                    $(stringify!($tag) => Self::$variant,)*
                    _ => Self::Div,
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

        /// Renders `children`/`class`/`data-state`/`attributes` on whichever
        /// element `component` selects. `component` is a runtime value
        /// shared by every caller (not known per call site), so this is one
        /// dispatch, not eliminated at compile time - each arm is trivial
        /// (same attrs, different element), so that cost is negligible.
        pub(crate) fn render_polymorphic(
            component: HtmlTag,
            class: Option<String>,
            data_state: Option<String>,
            attributes: Vec<Attribute>,
            events: BoxEvents,
            children: Element,
        ) -> Element {
            match component {
                $(HtmlTag::$variant => rsx! {
                    $tag {
                        class: class,
                        "data-state": data_state,
                        onclick: move |event| events.onclick.call(event),
                        onkeydown: move |event| events.onkeydown.call(event),
                        onmounted: move |event| events.onmounted.call(event),
                        onerror: move |event| events.onerror.call(event),
                        onblur: move |event| events.onblur.call(event),
                        onanimationend: move |event| events.onanimationend.call(event),
                        onchange: move |event| events.onchange.call(event),
                        ..attributes,
                        {children}
                    }
                }),*
            }
        }
    };
}

/// The event handlers [`render_polymorphic`]/[`crate::components::Box`]
/// support explicitly - kept as one bundle since a plain element accepts any
/// of these directly, but a component's `extends = GlobalAttributes` only
/// forwards non-event attributes, not arbitrary event handlers.
#[derive(Clone, Copy, Default)]
pub struct BoxEvents {
    pub onclick: EventHandler<MouseEvent>,
    pub onkeydown: EventHandler<KeyboardEvent>,
    pub onmounted: EventHandler<MountedEvent>,
    pub onerror: EventHandler<ImageEvent>,
    pub onblur: EventHandler<FocusEvent>,
    pub onanimationend: EventHandler<AnimationEvent>,
    pub onchange: EventHandler<FormEvent>,
}

// The full HTML5 element set dioxus_elements supports, so this list never
// needs to grow again for a new tag.
html_tags! {
    A => a,
    Abbr => abbr,
    Address => address,
    Area => area,
    Article => article,
    Aside => aside,
    Audio => audio,
    B => b,
    Base => base,
    Bdi => bdi,
    Bdo => bdo,
    Blockquote => blockquote,
    Body => body,
    Br => br,
    Button => button,
    Canvas => canvas,
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
    Embed => embed,
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
    Head => head,
    Header => header,
    Hgroup => hgroup,
    Hr => hr,
    I => i,
    Iframe => iframe,
    Img => img,
    Input => input,
    Ins => ins,
    Kbd => kbd,
    Label => label,
    Legend => legend,
    Li => li,
    Link => link,
    Main => main,
    Map => map,
    Mark => mark,
    Menu => menu,
    Meta => meta,
    Meter => meter,
    Nav => nav,
    Noscript => noscript,
    Object => object,
    Ol => ol,
    Optgroup => optgroup,
    Option => option,
    Output => output,
    P => p,
    Param => param,
    Picture => picture,
    Pre => pre,
    Progress => progress,
    Q => q,
    Rp => rp,
    Rt => rt,
    Ruby => ruby,
    S => s,
    Samp => samp,
    Script => script,
    Section => section,
    Select => select,
    Slot => slot,
    Small => small,
    Source => source,
    Span => span,
    Strong => strong,
    Style => style,
    Sub => sub,
    Summary => summary,
    Sup => sup,
    Table => table,
    Tbody => tbody,
    Td => td,
    Template => template,
    Textarea => textarea,
    Tfoot => tfoot,
    Th => th,
    Thead => thead,
    Time => time,
    Title => title,
    Tr => tr,
    Track => track,
    U => u,
    Ul => ul,
    Var => var,
    Video => video,
    Wbr => wbr,
}
