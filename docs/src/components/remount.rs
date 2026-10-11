use dioxus::prelude::*;

/// Mounts `children` afresh whenever `on` changes. A single root's `key` is not compared by the
/// pinned dioxus, so a demo that seeds state from `default_*` props needs this loop instead.
#[component]
pub fn Remount(on: String, children: Element) -> Element {
    rsx! {
        for key in [on] {
            Fragment { key: "{key}", {children.clone()} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Props)]
    struct SeededProps {
        mounts: Signal<u32>,
    }

    #[component]
    fn Seeded(props: SeededProps) -> Element {
        let mut mounts = props.mounts;
        use_hook(move || mounts += 1);
        rsx! { "seeded" }
    }

    #[test]
    fn a_changed_key_mounts_the_children_again() {
        thread_local! {
            static MOUNTS: std::cell::Cell<Option<Signal<u32>>> = const { std::cell::Cell::new(None) };
            static ON: std::cell::Cell<Option<Signal<String>>> = const { std::cell::Cell::new(None) };
        }

        fn app() -> Element {
            let mounts = use_signal(|| 0);
            let on = use_signal(|| "a".to_string());
            MOUNTS.set(Some(mounts));
            ON.set(Some(on));
            rsx! {
                Remount { on: on(), Seeded { mounts } }
            }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let (mounts, mut on) = (MOUNTS.get().unwrap(), ON.get().unwrap());
        assert_eq!(*mounts.peek(), 1);
        dom.in_runtime(|| on.set("a".to_string()));
        dom.render_immediate_to_vec();
        assert_eq!(*mounts.peek(), 1, "the same key mounted again");
        dom.in_runtime(|| on.set("b".to_string()));
        dom.render_immediate_to_vec();
        assert_eq!(*mounts.peek(), 2, "a new key kept the old mount");
    }
}
