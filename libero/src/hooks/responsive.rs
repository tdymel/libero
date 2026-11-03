use dioxus::prelude::*;

use crate::{context::Viewport, tokens::Responsive};

/// A per-breakpoint value as Rust reads it: keyboard axes, pointer maths, anything
/// CSS cannot carry. A plain value is just that value: no viewport watch, no
/// re-render.
///
/// A responsive one follows the window. The server, and a client hydrating its HTML,
/// render `base` and switch after mount; a client-only web app reads the breakpoint in
/// its first render; natively it arrives after the first layout. So put what shows on
/// screen in CSS (range copies) and keep this for behaviour.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{hooks::use_responsive, theme::{Responsive, responsive}};
/// # fn app() -> Element {
/// let columns: Responsive<u8> = responsive(1).md(3);
/// let columns = use_responsive(&columns);
///
/// rsx! { "{columns} per row" }
/// # }
/// ```
pub fn use_responsive<T: Clone + PartialEq + 'static>(value: &Responsive<T>) -> ReadSignal<T> {
    let viewport = use_hook(try_consume_context::<Viewport>);
    let value = value.clone();
    use_memo(use_reactive!(|value| {
        let breakpoint = viewport
            .filter(|_| !value.is_plain())
            .and_then(|viewport| viewport.breakpoint());
        match breakpoint {
            Some(Some(size)) => value.at(size),
            _ => value.base_ref().clone(),
        }
    }))
    .into()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;
    use crate::{context::use_viewport_provider, theme::responsive, tokens::Size};

    thread_local! {
        static SEEN: RefCell<Vec<(u8, u8)>> = const { RefCell::new(Vec::new()) };
        static VIEWPORT: Cell<Option<Viewport>> = const { Cell::new(None) };
    }

    fn app() -> Element {
        let viewport = use_viewport_provider(true);
        VIEWPORT.set(Some(viewport));
        rsx! { Reader {} }
    }

    #[component]
    fn Reader() -> Element {
        let plain = use_responsive(&Responsive::from(2u8));
        let columns = use_responsive(&responsive(1u8).sm(2).lg(4));
        SEEN.with_borrow_mut(|seen| seen.push((plain(), columns())));
        rsx! {}
    }

    fn settle(dom: &mut VirtualDom) {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }

    #[test]
    fn the_base_until_a_breakpoint_is_known_then_its_value() {
        SEEN.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        // Nothing measures in a unit test: the server's answer.
        assert_eq!(SEEN.with_borrow(|seen| *seen.last().unwrap()), (2, 1));

        let viewport = VIEWPORT.get().unwrap();
        dom.in_runtime(|| viewport.force(Some(Size::Md)));
        settle(&mut dom);
        assert_eq!(SEEN.with_borrow(|seen| *seen.last().unwrap()), (2, 2));

        dom.in_runtime(|| viewport.force(Some(Size::Xxl)));
        settle(&mut dom);
        assert_eq!(SEEN.with_borrow(|seen| *seen.last().unwrap()), (2, 4));

        dom.in_runtime(|| viewport.force(None));
        settle(&mut dom);
        assert_eq!(SEEN.with_borrow(|seen| *seen.last().unwrap()), (2, 1));
    }

    #[component]
    fn PlainOnly() -> Element {
        let plain = use_responsive(&Responsive::from(7u8));
        SEEN.with_borrow_mut(|seen| seen.push((plain(), 0)));
        rsx! {}
    }

    fn plain_app() -> Element {
        let viewport = use_viewport_provider(true);
        VIEWPORT.set(Some(viewport));
        rsx! { PlainOnly {} }
    }

    #[test]
    fn a_plain_value_starts_no_watch_and_never_re_renders() {
        SEEN.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(plain_app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let viewport = VIEWPORT.get().unwrap();
        assert!(!viewport.wanted());

        let renders = SEEN.with_borrow(Vec::len);
        dom.in_runtime(|| viewport.force(Some(Size::Lg)));
        settle(&mut dom);
        assert_eq!(SEEN.with_borrow(Vec::len), renders);
    }
}
