use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::core::{AccordionView, SectionSpec, render_accordion};
use crate::{
    components::common::{
        HtmlTag, Input, OptionLabel, OptionSource, Options, base_props, input_from_str, parts_enum,
        use_name_warning,
    },
    hooks::{use_root_id, use_theme},
    theme::{AccordionChevronPosition, Responsive, Size},
    utils::warn,
};

input_from_str!(AccordionChevronPosition);

/// Which sections are open. The variant is the mode: `One` holds at most one.
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

parts_enum! {
    /// [`Accordion`]'s inner parts, for its `parts` prop. Direct paths, so an accordion in a panel keeps its own.
    pub enum AccordionPart {
        /// One section: its heading and panel.
        Item = "item" => "& > [data-slot='item']",
        /// The `h1`..`h6` round a trigger, or round a plain section's label.
        Heading = "heading" => "& > [data-slot='item'] > [data-slot='heading']",
        /// The section's button; `aria-expanded` is `true` while its panel is open.
        Trigger = "trigger" => "& > [data-slot='item'] > [data-slot='heading'] > [data-slot='trigger']",
        /// The label inside the trigger.
        Label = "label" => "& > [data-slot='item'] > [data-slot='heading'] > [data-slot='trigger'] > [data-slot='label']",
        /// The chevron at the trigger's end, or its start under `chevron_position`.
        Chevron = "chevron" => "& > [data-slot='item'] > [data-slot='heading'] > [data-slot='trigger'] > [data-slot='chevron']",
        /// The open section's body, inside its collapsing region.
        Panel = "panel" => "& > [data-slot='item'] > [role='region'] > * > [data-slot='panel']",
    }
}

base_props! {
    parts(AccordionPart);
    pub struct AccordionProps<T: Options> {
        /// The open sections; the variant picks one or many. Controlled, with `onchange`.
        #[props(default)]
        open: AccordionOpen<T>,
        /// Called with the whole new open set.
        #[props(default)]
        onchange: Option<EventHandler<AccordionOpen<T>>>,
        /// A section's body. A closed panel is never mounted, so it keeps no state.
        /// A closure that captures a plain value redraws only when its section's
        /// value or open state changes.
        #[props(default)]
        panel: Option<Callback<T, Element>>,
        /// Default: every section. A section it returns `false` for is a plain heading:
        /// no button, chevron or panel, outside the Tab order and the arrow keys.
        #[props(default)]
        has_panel: Option<Callback<T, bool>>,
        /// The sections; defaults to every `Options::options()`. Groups are flattened,
        /// and a pending source draws none. A disabled section the host opens through
        /// `open` cannot be closed from its trigger, so open only enabled ones.
        #[props(default, into)]
        options: OptionSource<T>,
        /// Overrides `Options::label`; `OptionLabel::rich` draws rsx.
        #[props(default)]
        option_label: Option<Callback<T, OptionLabel>>,
        /// The heading around each trigger, `h1`..`h6`: match the page outline.
        #[props(default, into)]
        heading: Input<HtmlTag>,
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Md)`.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
        /// Default `true`. `false` keeps the open section open in `One` mode: its trigger
        /// turns `aria-disabled`, and only opening another closes it. `Many` mode ignores it.
        #[props(default)]
        collapsible: Option<bool>,
        /// The chevron before (`start`) or after (`end`, the default) the label.
        #[props(default, into)]
        chevron_position: Input<AccordionChevronPosition>,
        /// Default `true`. `false` keeps the chevron pointing down when a section opens.
        #[props(default)]
        rotate_chevron: Option<bool>,
        /// Default `false`. `true` drops the horizontal padding of the triggers and panels, so
        /// the rows run edge to edge and line up with the content around the accordion.
        #[props(default)]
        flush: Option<bool>,
    }
}

/// Sections over an enum, each a heading whose button expands its panel.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Accordion, AccordionOpen, Options};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq, Options)] enum Step { Shipping, Payment }
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
/// # }
/// # #[component] fn AddressForm() -> Element { rsx! {} }
/// # #[component] fn CardForm() -> Element { rsx! {} }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/accordion>
#[component]
pub fn Accordion<T: Options>(props: AccordionProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();

    use_name_warning(
        props.onchange.is_some(),
        "Accordion: without `onchange` no section can ever open or close.",
    );
    use_name_warning(
        props.panel.is_some(),
        "Accordion: without `panel` an open section has nothing to show.",
    );

    let requested = props.heading.copied_or(HtmlTag::H3);
    let heading = match requested {
        level @ (HtmlTag::H1
        | HtmlTag::H2
        | HtmlTag::H3
        | HtmlTag::H4
        | HtmlTag::H5
        | HtmlTag::H6) => level,
        _ => HtmlTag::H3,
    };
    use_hook(|| {
        if heading != requested {
            warn(&format!(
                "Accordion: `heading` must be h1..h6, not {requested:?}; using h3."
            ));
        }
    });

    let list = props.options.or_static();
    let values = list.values();
    let option_disabled = list.disabled();
    let pinned = !props.collapsible.unwrap_or(true) && matches!(props.open, AccordionOpen::One(_));

    let plain: Vec<bool> = values
        .iter()
        .map(|value| {
            props
                .has_panel
                .as_ref()
                .is_some_and(|has_panel| !has_panel.call(value.clone()))
        })
        .collect();
    let warned = use_hook(|| Rc::new(Cell::new(false)));
    if !warned.get()
        && values
            .iter()
            .zip(&plain)
            .any(|(value, &plain)| plain && props.open.is_open(value))
    {
        warned.set(true);
        warn("Accordion: `open` names a section without a panel; it stays closed.");
    }

    let sections: Vec<SectionSpec<T>> = values
        .iter()
        .zip(&option_disabled)
        .zip(&plain)
        .map(|((value, &option_disabled), &plain)| SectionSpec {
            value: value.clone(),
            // Resolved here, never stale: only a rich label redraws its section each time.
            label: match &props.option_label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            },
            disabled: option_disabled,
            open: !plain && props.open.is_open(value),
            pinned: !plain && pinned && props.open.is_open(value),
            plain,
        })
        .collect();

    let open = props.open.clone();
    let onchange = props.onchange;
    let toggle = use_callback(move |index: usize| {
        if let Some(onchange) = &onchange
            && let Some(value) = values.get(index)
            && !plain.get(index).copied().unwrap_or(false)
            && !(pinned && open.is_open(value))
        {
            onchange.call(open.toggled(value.clone()));
        }
    });

    render_accordion(
        AccordionView {
            sections,
            panel: props.panel,
            ontoggle: toggle,
            heading,
            size: props.size.copied_or(theme.accordion.size.into()),
            chevron_position: props
                .chevron_position
                .copied_or(AccordionChevronPosition::default()),
            rotate_chevron: props.rotate_chevron.unwrap_or(true),
            flush: props.flush.unwrap_or(false),
            class: props.class,
            sx: props.sx,
            parts: props.parts,
            states: props.states,
            hidden_from: props.hidden_from,
            visible_from: props.visible_from,
            attributes: props.attributes,
        },
        root(),
    )
}
