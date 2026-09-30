use dioxus::prelude::*;

use super::VisuallyHidden;

/// A polite live region for one-off messages. Render it unconditionally:
/// a region inserted with its text is not announced.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Announcer {
    /// Counted, so a message equal to the last one still lands as a new node.
    /// The count outlives `clear`, so it only grows.
    said: Signal<(u64, Option<String>)>,
}

impl Announcer {
    pub(crate) fn say(mut self, message: String) {
        let count = self.said.peek().0 + 1;
        self.said.set((count, Some(message)));
    }

    /// Empties the region, so a stale message is not read on a later visit.
    pub(crate) fn clear(mut self) {
        if self.said.peek().1.is_some() {
            self.said.write().1 = None;
        }
    }

    /// The region reads the message in its own scope, so the owner does not re-render per message.
    pub(crate) fn render(self) -> Element {
        rsx! {
            AnnouncerRegion { said: self.said }
        }
    }
}

#[component]
fn AnnouncerRegion(said: Signal<(u64, Option<String>)>) -> Element {
    let (count, message) = said.read().clone();
    rsx! {
        VisuallyHidden { role: "status",
            for message in message {
                span { key: "{count}", "{message}" }
            }
        }
    }
}

pub(crate) fn use_announcer() -> Announcer {
    Announcer {
        said: use_signal(|| (0, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeat_after_clear_is_a_new_node() {
        let mut dom = VirtualDom::new(|| {
            let announcer = use_announcer();
            use_hook(|| {
                announcer.say("Saved".into());
                let first = announcer.said.peek().0;
                announcer.clear();
                announcer.say("Saved".into());
                assert_ne!(announcer.said.peek().0, first, "the key repeated");
            });
            rsx! {}
        });
        dom.rebuild_in_place();
    }

    #[test]
    fn a_message_does_not_re_render_the_owner() {
        use std::cell::Cell;
        thread_local! {
            static RENDERS: Cell<u32> = const { Cell::new(0) };
            static ANNOUNCER: Cell<Option<Announcer>> = const { Cell::new(None) };
        }
        #[component]
        fn Owner() -> Element {
            RENDERS.set(RENDERS.get() + 1);
            let announcer = use_announcer();
            ANNOUNCER.set(Some(announcer));
            announcer.render()
        }
        let mut dom = VirtualDom::new(|| rsx! { crate::LiberoProvider { Owner {} } });
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let renders = RENDERS.get();
        let announcer = ANNOUNCER.get().unwrap();
        dom.in_runtime(|| announcer.say("Saved".into()));
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert!(
            dioxus_ssr::render(&dom).contains("Saved"),
            "the region missed the message"
        );
        assert_eq!(RENDERS.get(), renders, "the owner re-rendered");
    }
}
