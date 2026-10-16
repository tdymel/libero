//! Text ordered by a language's rules: the browser's `Intl.Collator` on the web, ICU4X's with
//! the `collation` feature elsewhere, none otherwise.

use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cmp::Ordering;

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

    /// An `Intl.Collator` for `lang`, the default locale's when the tag is not valid.
    pub(crate) fn build(lang: &str) -> Option<Collator> {
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
}

#[cfg(all(not(target_arch = "wasm32"), feature = "collation"))]
mod icu {
    use std::cmp::Ordering;

    use icu_locale_core::Locale;

    pub(crate) struct Collator(icu_collator::CollatorBorrowed<'static>);

    impl Collator {
        pub(crate) fn compare(&self, a: &str, b: &str) -> Ordering {
            self.0.compare(a, b)
        }
    }

    /// An ICU4X collator for the BCP 47 tag `lang`, the root locale's when the tag is not valid.
    pub(crate) fn build(lang: &str) -> Option<Collator> {
        let locale = lang.parse::<Locale>().unwrap_or(Locale::UNKNOWN);
        icu_collator::Collator::try_new(locale.into(), Default::default())
            .ok()
            .map(Collator)
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "collation")))]
mod none {
    use std::{cmp::Ordering, convert::Infallible};

    pub(crate) struct Collator(Infallible);

    impl Collator {
        pub(crate) fn compare(&self, _: &str, _: &str) -> Ordering {
            match self.0 {}
        }
    }

    /// Never one: only the browser, or ICU4X with `collation`, has a collator.
    pub(crate) fn build(_: &str) -> Option<Collator> {
        None
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "collation"))]
pub(crate) use icu::Collator;
#[cfg(all(not(target_arch = "wasm32"), feature = "collation"))]
use icu::build;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "collation")))]
pub(crate) use none::Collator;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "collation")))]
use none::build;
#[cfg(target_arch = "wasm32")]
pub(crate) use web::Collator;
#[cfg(target_arch = "wasm32")]
use web::build;

thread_local! {
    static LAST: RefCell<Option<(String, Rc<Collator>)>> = const { RefCell::new(None) };
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

#[cfg(all(test, not(target_arch = "wasm32"), feature = "collation"))]
mod tests {
    use super::*;

    #[test]
    fn a_language_orders_its_letters() {
        let sorted = |lang: &str| {
            let collator = collator(lang).unwrap();
            let mut words = ["äpple", "zoe", "anna"];
            words.sort_by(|a, b| collator.compare(a, b));
            words
        };
        assert_eq!(sorted("sv"), ["anna", "zoe", "äpple"]);
        assert_eq!(sorted("de"), ["anna", "äpple", "zoe"]);
    }

    #[test]
    fn an_invalid_tag_falls_back_to_the_root_locale() {
        assert!(collator("not a tag").unwrap().compare("a", "b").is_lt());
    }
}
