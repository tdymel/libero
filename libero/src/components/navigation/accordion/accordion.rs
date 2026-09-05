use dioxus::prelude::*;

use super::core::{AccordionView, SectionSpec, render_accordion};
use crate::{
    components::{HtmlTag, Input, OptionLabel, Options, common::base_props},
    hooks::{use_root_id, use_theme},
    theme::Size,
    utils::warn,
};

/// Which sections are open. **The variant is the mode**: `One` can hold at most
/// one section, so "single mode with two open" is not a state a caller can
/// build, and opening a section in `One` closes the other by construction.
#[derive(Clone, Debug, PartialEq)]
pub enum AccordionOpen<T> {
    /// At most one open section. Opening another closes it.
    One(Option<T>),
    /// Any number open, each toggled on its own.
    Many(Vec<T>),
}

impl<T> Default for AccordionOpen<T> {
    fn default() -> Self {
        Self::One(None)
    }
}

impl<T> From<Option<T>> for AccordionOpen<T> {
    fn from(open: Option<T>) -> Self {
        Self::One(open)
    }
}

impl<T> From<Vec<T>> for AccordionOpen<T> {
    fn from(open: Vec<T>) -> Self {
        Self::Many(open)
    }
}

impl<T: PartialEq + Clone> AccordionOpen<T> {
    /// The open section in `One` mode. `None` in `Many` mode, whatever is open.
    pub fn one(&self) -> Option<&T> {
        match self {
            Self::One(open) => open.as_ref(),
            Self::Many(_) => None,
        }
    }

    /// Every open section, in either mode.
    pub fn values(&self) -> &[T] {
        match self {
            Self::One(open) => open.as_slice(),
            Self::Many(open) => open,
        }
    }

    pub fn is_open(&self, value: &T) -> bool {
        self.values().contains(value)
    }

    /// The set after `value`'s trigger is pressed, in the same mode.
    pub fn toggled(&self, value: T) -> Self {
        match self {
            Self::One(open) if open.as_ref() == Some(&value) => Self::One(None),
            Self::One(_) => Self::One(Some(value)),
            Self::Many(open) if open.contains(&value) => {
                Self::Many(open.iter().filter(|v| **v != value).cloned().collect())
            }
            Self::Many(open) => {
                let mut open = open.clone();
                open.push(value);
                Self::Many(open)
            }
        }
    }
}

base_props! {
    pub struct AccordionProps<T: Options> {
        /// Which sections are expanded, and through the variant whether one or
        /// many may be. Strictly controlled - pair it with `onchange`.
        #[props(default)]
        open: AccordionOpen<T>,
        /// Called with the whole new open set, ready to store.
        #[props(default)]
        onchange: Option<EventHandler<AccordionOpen<T>>>,
        /// A section's body. Called for every section on each render, but a
        /// closed panel's rsx is never mounted: its components do not run and
        /// keep no state, and a half-typed form does not survive a close.
        #[props(default)]
        panel: Option<Callback<T, Element>>,
        /// The sections to show. Defaults to every `Options::options()`.
        #[props(default)]
        sections: Option<Vec<T>>,
        /// Overrides `Options::label`, like `Tabs::option_label`. `OptionLabel::rich`
        /// draws a trigger as rsx and still names it.
        #[props(default)]
        option_label: Option<Callback<T, OptionLabel>>,
        /// Sections that render but cannot be toggled.
        #[props(default)]
        disabled_options: Vec<T>,
        /// The heading element around each trigger, `h1`..`h6`. Pick the level
        /// the page outline needs; `h3` is a default, not an answer.
        #[props(default, into)]
        heading: Input<HtmlTag>,
        #[props(default, into)]
        size: Input<Size>,
    }
}

/// A list of sections over an enum, each a heading whose button expands its
/// panel. Controlled: it renders `open` and asks for a new set through
/// `onchange`.
///
/// The sections are `T::options()` unless `sections` narrows them, and `panel`
/// is a match over `T` - so a forgotten section is a compile error.
///
/// ```ignore
/// let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
/// rsx! {
///     Accordion {
///         open: open(),
///         onchange: move |next| open.set(next),
///         panel: |step: Step| match step {
///             Step::Shipping => rsx! { AddressForm {} },
///             Step::Payment => rsx! { CardForm {} },
///         },
///     }
/// }
/// ```
#[component]
pub fn Accordion<T: Options>(props: AccordionProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();

    if props.onchange.is_none() {
        warn("Accordion: without `onchange` no section can ever open or close.");
    }
    if props.panel.is_none() {
        warn("Accordion: without `panel` an open section has nothing to show.");
    }

    let heading = match props.heading.copied_or(HtmlTag::H3) {
        level @ (HtmlTag::H1
        | HtmlTag::H2
        | HtmlTag::H3
        | HtmlTag::H4
        | HtmlTag::H5
        | HtmlTag::H6) => level,
        other => {
            warn(&format!(
                "Accordion: `heading` must be h1..h6, not {other:?}; using h3."
            ));
            HtmlTag::H3
        }
    };

    let values = props
        .sections
        .clone()
        .unwrap_or_else(|| T::options().to_vec());

    let sections: Vec<SectionSpec> = values
        .iter()
        .map(|value| {
            let label = match &props.option_label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            };
            let name = label.name;
            SectionSpec {
                content: label.content.unwrap_or_else(|| rsx! { "{name}" }),
                name,
                disabled: props.disabled_options.contains(value),
                open: props.open.is_open(value),
                // Not only while open: a closing panel animates out around its
                // content, and only `Collapse` knows when that ends. It mounts
                // this only while open or closing.
                panel: match &props.panel {
                    Some(panel) => panel.call(value.clone()),
                    None => rsx! {},
                },
            }
        })
        .collect();

    let open = props.open.clone();
    let onchange = props.onchange;
    let toggle = use_callback(move |index: usize| {
        if let Some(onchange) = &onchange
            && let Some(value) = values.get(index)
        {
            onchange.call(open.toggled(value.clone()));
        }
    });

    render_accordion(
        AccordionView {
            sections,
            ontoggle: toggle,
            heading,
            size: props.size.copied_or(theme.accordion.size),
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        },
        root(),
    )
}
