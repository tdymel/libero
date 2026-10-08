use dioxus::prelude::*;

use crate::hooks::{Align, ElementHandle, Side};

/// One stop of a [`use_tour`](super::use_tour): what to highlight and what to say.
/// A step without a [`target`](Self::target) shows its card in the middle of the screen.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::TourStep;
/// # use libero::hooks::{Side, use_element};
/// # fn app() -> Element {
/// let save = use_element();
/// let step = TourStep::new("save")
///     .target(save)
///     .title("Save")
///     .description("Keeps your changes.")
///     .side(Side::Top);
/// # rsx! { button { onmounted: save.mount(), "Save" } }
/// # }
/// ```
#[derive(Clone, PartialEq)]
pub struct TourStep {
    /// Tells the steps apart; the card is drawn afresh when it changes.
    pub key: String,
    /// The element the hole goes around.
    pub target: Option<ElementHandle>,
    /// A CSS selector for the target, looked up in the document when the step shows;
    /// [`target`](Self::target) wins. Not on a WebView, which centres the card.
    pub target_selector: Option<String>,
    /// The card's heading and, unless the tour has an `aria_label`, its name.
    pub title: Option<String>,
    pub description: Option<String>,
    /// Shown in place of `description`, for rich content.
    pub content: Option<Element>,
    /// The card's side of the target. It flips when that side has no room.
    pub side: Side,
    pub align: Align,
    /// Overrides [`TourDefaults::padding`](crate::theme::TourDefaults::padding).
    pub padding: Option<f64>,
    /// Overrides [`TourDefaults::radius`](crate::theme::TourDefaults::radius).
    pub radius: Option<f64>,
    /// Lets presses through the hole to the target, and Tab between the card and it.
    pub interactive: bool,
}

impl TourStep {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            target: None,
            target_selector: None,
            title: None,
            description: None,
            content: None,
            side: Side::Bottom,
            align: Align::Center,
            padding: None,
            radius: None,
            interactive: false,
        }
    }

    /// Sets [`target`](Self::target): the handle mounted on the element, `onmounted: handle.mount()`.
    pub fn target(mut self, target: ElementHandle) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets [`target_selector`](Self::target_selector), as `"#search"`: for a target
    /// in another component, without passing a handle down.
    pub fn target_selector(mut self, selector: impl Into<String>) -> Self {
        self.target_selector = Some(selector.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets [`content`](Self::content).
    pub fn content(mut self, content: Element) -> Self {
        self.content = Some(content);
        self
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets [`padding`](Self::padding), in pixels.
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Sets [`radius`](Self::radius), in pixels.
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Sets [`interactive`](Self::interactive): the target can be pressed, and
    /// Tab past the card's last control reaches it.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}
