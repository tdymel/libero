use dioxus::prelude::*;

use super::list::ListContext;
use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    tokens::NamedColorCss,
    utils::warn,
};

const ORDERED_WARNING: &str = "ListSubheader: in an ordered List the numbers skip it but a screen reader counts it as an item; nest one ordered List per group";

// Block, not `list-item`, so an ordered list's numbering skips it. The smaller text is
// the inner span's: the `li` keeps the list's `em`, which pulls it out of the number gutter.
static LIST_SUBHEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .padding("0")
        .color(NamedColorCss::TEXT_DIMMED.value())
        .font_weight("600")
        .selector("& > span", sx().font_size("sm"))
});

base_props! {
    pub struct ListSubheaderProps {
        children: Element,
    }
}

/// A dimmed caption `<li>` that opens a run of items in a [`List`](super::List). It is
/// read as one more item; for groups a screen reader names, nest one `List` per group,
/// each with an `aria-label`. In an ordered `List` the numbers skip it but a screen reader
/// counts it, so nest one ordered `List` per group instead (a debug build warns).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem, ListSubheader};
/// # fn app() -> Element {
/// rsx! {
///     List {
///         ListSubheader { "Fruit" }
///         ListItem { "Apple" }
///         ListSubheader { "Vegetables" }
///         ListItem { "Carrot" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/list>
#[component]
pub fn ListSubheader(props: ListSubheaderProps) -> Element {
    let ordered = try_use_context::<ListContext>().is_some_and(|list| list.ordered);
    use_hook(|| {
        if ordered {
            warn(ORDERED_WARNING);
        }
    });
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("subheader", true)
        .into();

    use_box()
        .framework_sx(&LIST_SUBHEADER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(
            HtmlTag::Li,
            props.attributes,
            rsx! {
                span { {props.children} }
            },
        )
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::{LiberoProvider, utils::warnings_of};

    use super::super::List;

    /// Todo 4629: a screen reader counts a subheader in an ordered list, the numbers do not.
    #[test]
    fn a_subheader_in_an_ordered_list_warns_once() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    List { ordered: true,
                        ListSubheader { "Fruit" }
                    }
                }
            }
        }

        assert_eq!(warnings_of(app), [ORDERED_WARNING]);
    }

    #[test]
    fn a_subheader_in_a_bare_list_does_not_warn() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    List {
                        ListSubheader { "Fruit" }
                    }
                }
            }
        }

        assert!(warnings_of(app).is_empty());
    }
}
