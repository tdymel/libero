//! Media queries natively: stylo parses each query once and evaluates it against the document's
//! own device, so it answers as the page's `@media` rules match (todo 3878).

use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use blitz_dom::BaseDocument;
use cssparser::Parser;
use style::{
    context::QuirksMode,
    media_queries::MediaList,
    parser::ParserContext,
    stylesheets::{CssRuleType, CustomMediaEvaluator, Origin, UrlExtraData},
};
use style_traits::ParsingMode;

use super::{anchor, color_scheme, on_viewport_resize, when_laid_out};
use crate::platform::{
    ColorSchemeSubscription, MediaQueryApi, MediaQuerySubscription, ScrollSubscription,
};

pub(in crate::platform::backend) fn media_query() -> Option<&'static dyn MediaQueryApi> {
    Some(&MEDIA_QUERY)
}

struct BlitzMediaQuery;

static MEDIA_QUERY: BlitzMediaQuery = BlitzMediaQuery;

fn parse(query: &str) -> MediaList {
    // A media query holds no URL: any base will do.
    let url = UrlExtraData::from(url::Url::parse("about:blank").expect("a valid URL"));
    let mut context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Media),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        Default::default(),
        None,
        None,
        Default::default(),
    );
    MediaList::parse(&mut context, &mut Parser::new(query))
}

fn matches(doc: &mut BaseDocument, list: &MediaList) -> bool {
    list.evaluate(
        doc.stylist_device(),
        QuirksMode::NoQuirks,
        &mut CustomMediaEvaluator::none(),
    )
}

struct Watcher {
    list: MediaList,
    last: Cell<Option<bool>>,
    callback: Box<dyn Fn(bool)>,
}

/// Asks once the document is laid out; a watch dropped meanwhile hears nothing.
fn check(watcher: Weak<Watcher>) {
    when_laid_out(Box::new(move || {
        let (Some(watcher), Some(anchor)) = (watcher.upgrade(), anchor()) else {
            return;
        };
        let now = matches(&mut anchor.doc_mut(), &watcher.list);
        if watcher.last.replace(Some(now)) != Some(now) {
            (watcher.callback)(now);
        }
    }));
}

impl MediaQueryApi for BlitzMediaQuery {
    /// The first answer once the document is laid out, then each change a window resize
    /// brings, and a scheme switch for a query that asks for the scheme.
    fn watch(&self, query: &str, callback: Box<dyn Fn(bool)>) -> Box<dyn MediaQuerySubscription> {
        let watcher = Rc::new(Watcher {
            list: parse(query),
            last: Cell::new(None),
            callback,
        });
        let weak = Rc::downgrade(&watcher);
        check(weak.clone());
        let resize = on_viewport_resize(Box::new({
            let weak = weak.clone();
            move || check(weak.clone())
        }));
        // The scheme is polled, so only a query that reads it starts the poll.
        let scheme = query
            .contains("prefers-color-scheme")
            .then(|| color_scheme())
            .flatten()
            .map(|api| api.on_change(Box::new(move |_| check(weak.clone()))));
        Box::new(BlitzMediaWatch {
            _watcher: watcher,
            _resize: resize,
            _scheme: scheme,
        })
    }
}

struct BlitzMediaWatch {
    _watcher: Rc<Watcher>,
    _resize: Option<Box<dyn ScrollSubscription>>,
    _scheme: Option<Box<dyn ColorSchemeSubscription>>,
}

impl MediaQuerySubscription for BlitzMediaWatch {}
