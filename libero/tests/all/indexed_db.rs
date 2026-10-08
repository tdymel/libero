//! `use_indexed_db` on a server render: no store to keep values in, so the default
//! shows, the load is done and `error()` says `Unavailable`.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::hooks::use_indexed_db;

#[test]
fn a_server_render_shows_the_default_and_no_store() {
    fn app() -> Element {
        let count = use_indexed_db("count", || 3_u32);
        rsx! {
            p { id: "count", "{count.get()}" }
            p { id: "loaded", "{count.is_loaded()}" }
            p { id: "stored", "{count.is_stored()}" }
            p { id: "error", "{count.error():?}" }
        }
    }

    let html = body(&render(app));

    for expected in [
        r#"<p id="count">3</p>"#,
        r#"<p id="loaded">true</p>"#,
        r#"<p id="stored">false</p>"#,
        r#"<p id="error">Some(Unavailable)</p>"#,
    ] {
        assert!(html.contains(expected), "{expected} missing in {html}");
    }
}
