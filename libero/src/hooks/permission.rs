use dioxus::prelude::*;

use crate::platform::{PermissionKind, PermissionState, PermissionSubscription, permission};

/// One permission's state, kept current from [`follow`](Self::follow) until unmount.
#[derive(Clone, Copy)]
pub(crate) struct FollowedPermission {
    kind: PermissionKind,
    state: Signal<PermissionState>,
    listener: CopyValue<Option<Box<dyn PermissionSubscription>>>,
}

impl FollowedPermission {
    /// Reactive.
    pub(crate) fn get(&self) -> PermissionState {
        (self.state)()
    }

    pub(crate) fn set(&mut self, state: PermissionState) {
        if *self.state.peek() != state {
            self.state.set(state);
        }
    }

    /// Listens for the platform's reports. Call it after mount: a server cannot know the answer.
    pub(crate) fn follow(&mut self) {
        let Some(api) = permission() else {
            return;
        };
        let this = *self;
        let changes = api.on_change(
            self.kind,
            Box::new(move |state| {
                let mut this = this;
                this.report(state);
            }),
        );
        self.listener.set(Some(changes));
    }

    /// `Unknown` (the platform does not know the name) keeps the answer a grant, denial or probe gave.
    fn report(&mut self, state: PermissionState) {
        if state != PermissionState::Unknown {
            self.set(state);
        }
    }
}

/// A [`FollowedPermission`] for `kind`, `Unknown` until told or followed.
pub(crate) fn use_permission(kind: PermissionKind) -> FollowedPermission {
    let followed = FollowedPermission {
        kind,
        state: use_signal(PermissionState::default),
        listener: use_hook(|| CopyValue::new(None)),
    };
    use_drop(move || drop(followed.listener.write_unchecked().take()));
    followed
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    thread_local! {
        static HANDLE: Cell<Option<FollowedPermission>> = const { Cell::new(None) };
    }

    #[test]
    fn an_unknown_report_keeps_the_known_state() {
        let mut dom = VirtualDom::new(|| {
            HANDLE.set(Some(use_permission(PermissionKind::Camera)));
            rsx! {}
        });
        dom.rebuild_in_place();
        let mut followed = HANDLE.get().expect("rendered");
        dom.in_runtime(|| {
            followed.report(PermissionState::Granted);
            followed.report(PermissionState::Unknown);
            assert_eq!(followed.get(), PermissionState::Granted);
            followed.report(PermissionState::Denied);
            assert_eq!(followed.get(), PermissionState::Denied);
        });
    }
}
