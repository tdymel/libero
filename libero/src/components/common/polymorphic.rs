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

        /// Renders `children`/`class`/`data-state`/`attributes`/`onmounted`
        /// on whichever element `component` selects (a runtime value, so
        /// this dispatches at runtime - each arm is trivial, so that cost is
        /// negligible). `attributes` carries event handlers too, not just
        /// plain attributes - `Box`'s own `extends = GlobalAttributes`
        /// captures whatever event a caller writes generically, so every arm
        /// just needs to spread it, not name each event it might contain.
        ///
        /// `onmounted` is its own literal field rather than folded into
        /// `attributes` - a `dioxus_core::AttributeValue::Listener` built by
        /// hand in plain Rust code (as opposed to one the `rsx!` macro
        /// builds itself from an `onmounted: closure` field) silently
        /// receives the wrong event data when dispatched. Writing it as a
        /// real field here, like any other `onmounted: closure`, goes
        /// through the same macro-generated path as a directly-written
        /// `onmounted:` at a normal call site (already used successfully
        /// elsewhere - `Select`, `FocusTrap`, `NavLink`, `Image`), which
        /// does not hit that bug.
        pub(crate) fn render_polymorphic(
            component: HtmlTag,
            class: Option<String>,
            data_state: Option<String>,
            attributes: Vec<Attribute>,
            onmounted: impl FnMut(Event<MountedData>) + 'static,
            children: Element,
        ) -> Element {
            match component {
                $(HtmlTag::$variant => rsx! {
                    $tag {
                        class: class,
                        "data-state": data_state,
                        onmounted: onmounted,
                        ..attributes,
                        {children}
                    }
                }),*
            }
        }
    };
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
