use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

/// The one platform subscription a hook holds: replacing or clearing it drops
/// the old one, and so does the component unmounting.
pub(crate) struct SubscriptionSlot<T: ?Sized>(Rc<RefCell<Option<Box<T>>>>);

impl<T: ?Sized> Clone for SubscriptionSlot<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> SubscriptionSlot<T> {
    /// Released before the old one drops, so its drop may reach this slot.
    pub(crate) fn set(&self, subscription: Option<Box<T>>) {
        let old = self.0.replace(subscription);
        drop(old);
    }

    pub(crate) fn clear(&self) {
        self.set(None);
    }

    pub(crate) fn is_some(&self) -> bool {
        self.0.borrow().is_some()
    }
}

/// An empty [`SubscriptionSlot`] cleared when the component unmounts.
pub(crate) fn use_subscription_slot<T: ?Sized + 'static>() -> SubscriptionSlot<T> {
    let slot = use_hook(|| SubscriptionSlot(Rc::new(RefCell::new(None))));
    use_drop({
        let slot = slot.clone();
        move || slot.clear()
    });
    slot
}
