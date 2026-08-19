use dioxus::prelude::*;

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
                // The value is fine, the build configuration is not - a
                // different fix from a typo, so a different message.
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

// `default` is every tag `libero` and `docs` actually render, so both work on
// default features. `full` is the rest of the HTML5 set, behind
// `full-polymorphism`; without it a `full`-tier tag still type-checks and
// renders as `<div>`. A new tag never grows this split, it only picks a side.
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
