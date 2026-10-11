//! Text ordered by a language's rules: the browser's `Intl.Collator` on the web, none elsewhere.

#[cfg(target_arch = "wasm32")]
mod web {
    use std::{cell::RefCell, cmp::Ordering, rc::Rc};

    use js_sys::{Array, Function, Reflect};
    use wasm_bindgen::{JsCast, JsValue};

    pub(crate) struct Collator(Function);

    impl Collator {
        pub(crate) fn compare(&self, a: &str, b: &str) -> Ordering {
            self.0
                .call2(&JsValue::NULL, &a.into(), &b.into())
                .ok()
                .and_then(|order| order.as_f64())
                .and_then(|order| order.partial_cmp(&0.0))
                .unwrap_or(Ordering::Equal)
        }
    }

    thread_local! {
        static LAST: RefCell<Option<(String, Rc<Collator>)>> = const { RefCell::new(None) };
    }

    /// An `Intl.Collator` for `lang`, the default locale's when the tag is not valid.
    fn build(lang: &str) -> Option<Collator> {
        let intl = Reflect::get(&js_sys::global(), &"Intl".into()).ok()?;
        let ctor: Function = Reflect::get(&intl, &"Collator".into())
            .ok()?
            .dyn_into()
            .ok()?;
        let made = Reflect::construct(&ctor, &Array::of1(&lang.into()))
            .or_else(|_| Reflect::construct(&ctor, &Array::new()))
            .ok()?;
        let compare: Function = Reflect::get(&made, &"compare".into())
            .ok()?
            .dyn_into()
            .ok()?;
        Some(Collator(compare))
    }

    /// The collator for the BCP 47 tag `lang`; the last one asked for is kept.
    pub(crate) fn collator(lang: &str) -> Option<Rc<Collator>> {
        LAST.with(|last| {
            let mut last = last.borrow_mut();
            if let Some((cached, collator)) = last.as_ref()
                && cached == lang
            {
                return Some(collator.clone());
            }
            let collator = Rc::new(build(lang)?);
            *last = Some((lang.to_string(), collator.clone()));
            Some(collator)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{cmp::Ordering, convert::Infallible, rc::Rc};

    pub(crate) struct Collator(Infallible);

    impl Collator {
        pub(crate) fn compare(&self, _: &str, _: &str) -> Ordering {
            match self.0 {}
        }
    }

    /// Never one: only the browser has a collator.
    pub(crate) fn collator(_: &str) -> Option<Rc<Collator>> {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::{Collator, collator};
#[cfg(target_arch = "wasm32")]
pub(crate) use web::{Collator, collator};
