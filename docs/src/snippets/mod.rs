//! Every piece of Rust the docs pages print (`const` snippets and rendered `Demo` code), written
//! to the gitignored `libero/tests/page_snippets.md`: run this before libero's doc-tests.
//!
//! Each blank-line piece is placed by its first line: items on top, `let` in the body, rsx in
//! `rsx!`. Directives in comment lines right above the `const` or the page's `Demo {`:
//!
//! - `// snippet: ignore - <why>` - not compiled; required for non-Rust `CodeBlock`s.
//! - `// snippet: after A, B` - the page's snippets `A` and `B` go first.
//! - `// snippet: item <item>` - one more item, e.g. a stand-in icon component.
//! - `// snippet: let <statement>` - one more statement in the body.
//! - `// snippet: in <rsx>` - the rsx the snippet goes into, at `..`.

mod compile;
mod drift;
mod generate;
mod md_mirror;
mod parse;
mod props;
mod route_pages;

use compile::*;
use drift::*;
use md_mirror::*;
use parse::*;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::thread;

use dioxus::history::{History, MemoryHistory, provide_history_context};
use dioxus::prelude::*;

use crate::Route;
use crate::components::{DemoCode, DemoValues, PropGroup};

const PRELUDE: &str = "use dioxus::prelude::*;
use libero::components::*;
use libero::hooks::*;
use libero::localization::*;
use libero::platform::*;
use libero::sx::*;
use libero::theme::*;
use libero::{IconProvider, IconSet, IconSlot, LiberoProvider, use_theme};
// Both globs name a `Title`; the component is the one a snippet means.
use libero::components::Title;
use pictogram_icons_lucide as lucide;
use std::time::Duration;
";

thread_local! {
    static DEMOS: RefCell<Vec<DemoCode>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    static PAGES: RefCell<Vec<RecordedPage>> = const { RefCell::new(Vec::new()) };
}

/// A mounted `DocPage`'s title, markdown mirror and property groups.
pub struct RecordedPage {
    pub title: String,
    pub markdown: Option<String>,
    pub properties: Vec<PropGroup>,
}

/// Called by every `DocPage` as it mounts.
pub fn record_page(page: RecordedPage) {
    PAGES.with(|pages| pages.borrow_mut().push(page));
}

/// Called by every `Demo` as it mounts.
pub fn record(code: &DemoCode) {
    DEMOS.with(|demos| demos.borrow_mut().push(code.clone()));
}

struct Snippet {
    name: String,
    body: String,
    markers: Vec<String>,
}

#[derive(PartialEq)]
enum Shape {
    Items,
    Statements,
    /// A value like `sx().padding("md")`, compiled as `let _ = ..;`.
    Expression,
    /// A component's prop, like `label: |s: Section| ..`: needs `in`.
    Prop,
    Rsx,
    /// Not Rust as far as the first line tells: TOML, shell, prose.
    Unknown,
}

/// The site's own `App` at `route`, so the shell gets every context it reads:
/// a copy without the `Rtl` provider made the shell panic and hid every `Demo` (todo 827).
#[component]
fn Page(route: Route) -> Element {
    use_hook(|| {
        provide_history_context(
            Rc::new(MemoryHistory::with_initial_path(route.to_string())) as Rc<dyn History>
        )
    });
    rsx! {
        crate::App {}
    }
}

/// One static route, rendered once.
struct Rendered {
    route: Route,
    /// Every `Demo`, in render order.
    demos: Vec<DemoCode>,
    /// Every `DocPage`, in render order.
    pages: Vec<RecordedPage>,
}

fn render(route: &Route) -> Rendered {
    DEMOS.with(|demos| demos.borrow_mut().clear());
    PAGES.with(|pages| pages.borrow_mut().clear());
    let mut dom = VirtualDom::new_with_props(
        Page,
        PageProps {
            route: route.clone(),
        },
    );
    dom.rebuild_in_place();
    Rendered {
        route: route.clone(),
        demos: DEMOS.with(|demos| std::mem::take(&mut *demos.borrow_mut())),
        pages: PAGES.with(|pages| std::mem::take(&mut *pages.borrow_mut())),
    }
}

/// Every static route rendered once for all the tests here, split over threads: the renders
/// were most of their time, four times over.
fn rendered() -> &'static [Rendered] {
    static RENDERED: OnceLock<Vec<Rendered>> = OnceLock::new();
    RENDERED.get_or_init(|| {
        let routes = Route::static_routes();
        let threads = thread::available_parallelism().map_or(4, |n| n.get().min(8));
        thread::scope(|scope| {
            let handles: Vec<_> = routes
                .chunks(routes.len().div_ceil(threads).max(1))
                .map(|chunk| scope.spawn(|| chunk.iter().map(render).collect::<Vec<_>>()))
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().unwrap())
                .collect()
        })
    })
}

/// A method's name without its parameters: `show(args)` is `show`.
fn bare(name: &str) -> &str {
    name.split('(').next().unwrap_or(name)
}
