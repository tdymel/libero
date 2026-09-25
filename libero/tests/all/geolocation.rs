//! `use_geolocation` on a server render: no Geolocation API and no prompt;
//! the state is read in an effect, never during render (hydration).

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::hooks::{GeolocationOptions, use_geolocation};

#[test]
fn a_server_render_reports_no_geolocation() {
    fn app() -> Element {
        let location = use_geolocation(GeolocationOptions::default());
        rsx! {
            p { id: "supported", "{location.is_supported()}" }
            p { id: "permission", "{location.permission():?}" }
            p { id: "position", "{location.position():?}" }
            p { id: "error", "{location.error():?}" }
            p { id: "pending", "{location.is_pending()}" }
        }
    }

    let html = body(&render(app));

    for expected in [
        r#"<p id="supported">false</p>"#,
        r#"<p id="permission">Unsupported</p>"#,
        r#"<p id="position">None</p>"#,
        r#"<p id="error">None</p>"#,
        r#"<p id="pending">false</p>"#,
    ] {
        assert!(html.contains(expected), "{expected} missing in {html}");
    }
}
