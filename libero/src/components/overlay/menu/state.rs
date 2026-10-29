use dioxus::{core::AttributeValue, prelude::*};

use crate::{
    components::common::attr,
    hooks::{PressMarker, use_id, use_press_marker},
};

/// Which item takes focus when the menu opens. APG's menu button: ArrowUp on
/// the trigger opens on the last item, everything else on the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum MenuFocus {
    #[default]
    First,
    Last,
    /// The menu box, no item: a finger did not ask for a highlighted row (todo 4862).
    Container,
}

/// A [`Menu`](super::Menu)'s open state and the id tying the trigger to it.
#[derive(Clone, Copy, PartialEq)]
pub struct MenuState {
    id: Signal<String>,
    opened: Signal<bool>,
    /// Bumped per open, so the same item is focused again on a repeat request.
    request: Signal<(u64, MenuFocus)>,
    /// No items: an open state draws no menu, so the trigger says closed.
    empty: Signal<bool>,
    /// The trigger's own id, adopted by `a11y_attributes_on`. Written in the caller's
    /// render and read by the `Menu` in the same pass, so not reactive.
    adopted: CopyValue<Option<(String, Option<String>)>>,
    /// On the wrapper and on a trigger beside the `Menu`: a press on either is the toggle.
    marker: PressMarker,
}

/// One `Menu`'s state.
///
/// Docs: <https://libero-ui.dev/overlay/menu>
pub fn use_menu() -> MenuState {
    MenuState {
        id: use_id(),
        opened: use_signal(|| false),
        request: use_signal(|| (0, MenuFocus::First)),
        empty: use_signal(|| false),
        adopted: use_hook(|| CopyValue::new(None)),
        marker: use_press_marker(),
    }
}

impl MenuState {
    /// Not a hook, for a runtime number of menus (`Menubar`): `use_menu()` in a
    /// loop would hand hook slots between menus as the `Vec` changes.
    pub(crate) fn new(id: String) -> Self {
        Self {
            id: Signal::new(id),
            opened: Signal::new(false),
            request: Signal::new((0, MenuFocus::First)),
            empty: Signal::new(false),
            adopted: CopyValue::new(None),
            marker: PressMarker::new(),
        }
    }

    pub fn id(&self) -> String {
        (self.id)()
    }

    pub fn is_open(&self) -> bool {
        (self.opened)()
    }

    /// Opens the menu with its first item focused.
    pub fn open(&self) {
        self.open_at(MenuFocus::First);
    }

    pub fn close(&self) {
        let mut opened = self.opened;
        if *opened.peek() {
            opened.set(false);
        }
    }

    pub fn toggle(&self) {
        self.toggle_at(MenuFocus::First);
    }

    pub(crate) fn toggle_at(&self, focus: MenuFocus) {
        // Copied out: matching on `peek()` holds the borrow through the write.
        let opened = *self.opened.peek();
        match opened {
            true => self.close(),
            false => self.open_at(focus),
        }
    }

    pub(crate) fn open_at(&self, focus: MenuFocus) {
        let mut request = self.request;
        let next = request.peek().0.wrapping_add(1);
        request.set((next, focus));
        let mut opened = self.opened;
        if !*opened.peek() {
            opened.set(true);
        }
    }

    pub(crate) fn request(&self) -> (u64, MenuFocus) {
        (self.request)()
    }

    pub(super) fn set_empty(&self, empty: bool) {
        let mut flag = self.empty;
        if *flag.peek() != empty {
            flag.set(empty);
        }
    }

    /// The trigger's ARIA wiring. Spread it on the control inside the `Menu`:
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{Button, Menu, use_menu};
    /// # fn app() -> Element {
    /// # let menu = use_menu();
    /// # rsx! {
    /// Button { attributes: menu.a11y_attributes(), "Actions" }
    /// # } }
    /// ```
    pub fn a11y_attributes(&self) -> Vec<Attribute> {
        let mut attributes = vec![attr("id", trigger_id(&self.id()))];
        attributes.extend(self.wiring());
        attributes
    }

    /// `attributes` plus the trigger's ARIA wiring, for a trigger that already has an
    /// `id`, such as a [`FileField`](crate::components::FileField) trigger's: the menu
    /// keeps that id, anchors on it and is named as the trigger is. Without one, as
    /// [`a11y_attributes`](Self::a11y_attributes).
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{Button, FileField, FileTriggerArgs, Menu, MenuItem, use_menu};
    /// # fn app() -> Element {
    /// let menu = use_menu();
    /// rsx! {
    ///     FileField {
    ///         label: "Avatar",
    ///         trigger: move |args: FileTriggerArgs| {
    ///             let open = args.open;
    ///             rsx! {
    ///                 Menu {
    ///                     state: menu,
    ///                     items: vec![MenuItem::new("Upload").onselect(move |_| open.call(())).into()],
    ///                     Button { attributes: menu.a11y_attributes_on(args.attributes), "Change" }
    ///                 }
    ///             }
    ///         },
    ///     }
    /// }
    /// # }
    /// ```
    pub fn a11y_attributes_on(&self, mut attributes: Vec<Attribute>) -> Vec<Attribute> {
        let text = |name: &str| {
            attributes
                .iter()
                .rev()
                .find(|attribute| attribute.name == name)
                .and_then(|attribute| match &attribute.value {
                    AttributeValue::Text(text) if !text.trim().is_empty() => Some(text.clone()),
                    _ => None,
                })
        };
        // A name is not followed through two `aria-labelledby` hops, so the menu takes the trigger's.
        let own = text("id").map(|id| (id, text("aria-labelledby")));
        let mut adopted = self.adopted;
        if *adopted.peek() != own {
            adopted.set(own.clone());
        }
        if own.is_none() {
            attributes.push(attr("id", trigger_id(&self.id())));
        }
        attributes.extend(self.wiring());
        attributes
    }

    /// The trigger's `id`: the adopted one, else the menu's own.
    pub(super) fn trigger_element_id(&self) -> String {
        self.adopted
            .peek()
            .as_ref()
            .map_or_else(|| trigger_id(&self.id()), |(id, _)| id.clone())
    }

    /// What names the menu: an adopted trigger's `aria-labelledby`, else the trigger.
    pub(super) fn labelledby(&self) -> String {
        match &*self.adopted.peek() {
            Some((_, Some(labelledby))) => labelledby.clone(),
            _ => self.trigger_element_id(),
        }
    }

    fn wiring(&self) -> Vec<Attribute> {
        let opened = self.is_open() && !(self.empty)();
        let mut attributes = vec![
            attr("aria-haspopup", "menu"),
            attr("aria-expanded", opened.to_string()),
        ];
        // Only while open, when the menu is in the document.
        if opened {
            attributes.push(attr("aria-controls", menu_id(&self.id())));
            attributes.extend(self.marker.attribute());
        }
        attributes
    }

    pub(super) fn marker(&self) -> PressMarker {
        self.marker
    }
}

fn trigger_id(id: &str) -> String {
    format!("{id}-trigger")
}

pub(super) fn menu_id(id: &str) -> String {
    format!("{id}-menu")
}
