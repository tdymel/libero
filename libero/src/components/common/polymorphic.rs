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
            onclick: EventHandler<MouseEvent>,
            children: Element,
        ) -> Element {
            match component {
                $(HtmlTag::$variant => rsx! {
                    $tag {
                        class: class,
                        "data-state": data_state,
                        onclick: move |event| onclick.call(event),
                        ..attributes,
                        {children}
                    }
                }),*
            }
        }
    };
}

html_tags! {
    Div => div,
    Span => span,
    A => a,
    Hr => hr,
    Button => button,
    Li => li,
    Ul => ul,
    Ol => ol,
    Label => label,
    P => p,
    Nav => nav,
    Section => section,
    Article => article,
    Aside => aside,
    Header => header,
    Footer => footer,
    H1 => h1,
    H2 => h2,
    H3 => h3,
    H4 => h4,
    H5 => h5,
    H6 => h6,
}
