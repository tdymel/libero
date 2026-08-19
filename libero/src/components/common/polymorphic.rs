use dioxus::prelude::*;

use crate::components::Input;

macro_rules! html_tags {
    (
        default { $($dvariant:ident => $dtag:ident),* $(,)? }
        full { $($fvariant:ident => $ftag:ident),* $(,)? }
    ) => {
        /// Root elements a polymorphic component can render as. Always the
        /// full HTML5 element set regardless of the `full-polymorphism`
        /// feature - that feature only controls how many of
        /// `render_polymorphic`'s match arms actually compile, not this
        /// type's shape (so downstream code naming a `full`-tier variant
        /// still compiles either way, it just renders as `<div>` without
        /// the feature - see `render_polymorphic`'s fallback arm).
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
                    // No accepted-value list here, unlike `str_enum!`'s -
                    // the whole HTML5 element set is not useful output.
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

        /// Renders `children`/`class`/`data-state`/`attributes` on whichever
        /// element `component` selects (a runtime value, so this dispatches
        /// at runtime - each arm is trivial, so that cost is negligible).
        /// `attributes` carries event handlers too, not just plain
        /// attributes - `Box`'s own `extends = GlobalAttributes` captures
        /// whatever event a caller writes generically, so every arm just
        /// needs to spread it, not name each event it might contain.
        ///
        /// Only the `default` tier's arms (the tags `libero`'s own
        /// components actually render - see `polymorphic.rs`'s
        /// `html_tags!` invocation) compile unconditionally; the rest need
        /// the `full-polymorphism` feature, and fall back to `<div>`
        /// without it (same fallback `HtmlTag::from(&str)` already uses for
        /// an unrecognized tag name) - this is what keeps `Box`'s
        /// `render_polymorphic` from paying for all 111 HTML tags when a
        /// consumer only ever asks for a handful.
        pub(crate) fn render_polymorphic(
            component: HtmlTag,
            class: crate::components::ClassList,
            data_state: Option<String>,
            style: Option<String>,
            attributes: Vec<Attribute>,
            children: Element,
        ) -> Element {
            let class = class.to_string();
            match component {
                $(HtmlTag::$dvariant => rsx! {
                    $dtag {
                        class: class,
                        "data-state": data_state,
                        style: style,
                        ..attributes,
                        {children}
                    }
                },)*
                $(
                    #[cfg(feature = "full-polymorphism")]
                    HtmlTag::$fvariant => rsx! {
                        $ftag {
                            class: class,
                            "data-state": data_state,
                            style: style,
                            ..attributes,
                            {children}
                        }
                    },
                )*
                // A `full`-tier tag without the feature: the value is
                // fine, the build configuration is not - which is a
                // different fix from a typo, so it gets its own message.
                #[cfg(not(feature = "full-polymorphism"))]
                _ => {
                    crate::utils::warn(&format!(
                        "HtmlTag::{component:?} (<{}>) needs the \"full-polymorphism\" \
                         feature, rendering as <div>.",
                        component.as_str()
                    ));
                    rsx! {
                        div {
                            class: class,
                            "data-state": data_state,
                            style: style,
                            ..attributes,
                            {children}
                        }
                    }
                },
            }
        }
    };
}

// `default` is every tag `libero`'s own components render internally, plus
// every tag `docs` itself needs (grep for `component: "..."` / `HtmlTag::...`
// across `libero/src` and `docs/src` - see
// [[project_wasm_bundle_size_findings]]) - so both work fully with default
// features, `docs` included (no reason to make the site demonstrating the
// library pay the same "opt into more tags" cost an external consumer
// would). `full` is the rest of the HTML5 element set dioxus_elements
// supports - opt into the `full-polymorphism` feature for those; without it
// `Box`/`Title`/`Text`/etc still accept any `HtmlTag`, they just render as
// `<div>` for a `full`-tier tag (see `render_polymorphic`'s fallback arm).
// This split never needs to grow for a new tag - only which group a tag
// moves to, if `libero` or `docs` starts using one.
html_tags! {
    default {
        A => a,
        Button => button,
        Code => code,
        Dd => dd,
        Div => div,
        Dl => dl,
        Dt => dt,
        H1 => h1,
        H2 => h2,
        H3 => h3,
        H4 => h4,
        H5 => h5,
        H6 => h6,
        Header => header,
        Img => img,
        Kbd => kbd,
        Label => label,
        Li => li,
        Main => main,
        Mark => mark,
        Option => option,
        P => p,
        Pre => pre,
        Section => section,
        Select => select,
        Span => span,
        Ul => ul,
    }
    full {
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
        Canvas => canvas,
        Caption => caption,
        Cite => cite,
        Col => col,
        Colgroup => colgroup,
        Data => data,
        Datalist => datalist,
        Del => del,
        Details => details,
        Dfn => dfn,
        Dialog => dialog,
        Em => em,
        Embed => embed,
        Fieldset => fieldset,
        Figcaption => figcaption,
        Figure => figure,
        Footer => footer,
        Form => form,
        Head => head,
        Hgroup => hgroup,
        Hr => hr,
        I => i,
        Iframe => iframe,
        Input => input,
        Ins => ins,
        Legend => legend,
        Link => link,
        Map => map,
        Menu => menu,
        Meta => meta,
        Meter => meter,
        Nav => nav,
        Noscript => noscript,
        Object => object,
        Ol => ol,
        Optgroup => optgroup,
        Output => output,
        Param => param,
        Picture => picture,
        Progress => progress,
        Q => q,
        Rp => rp,
        Rt => rt,
        Ruby => ruby,
        S => s,
        Samp => samp,
        Script => script,
        Slot => slot,
        Small => small,
        Source => source,
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
        Var => var,
        Video => video,
        Wbr => wbr,
    }
}
