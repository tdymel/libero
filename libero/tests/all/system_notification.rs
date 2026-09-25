//! `use_system_notification` and `use_push_subscription` on a server render: no
//! API and no prompt; the state is read in an effect, never during render (hydration).

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::hooks::{PushOptions, use_push_subscription, use_system_notification};

#[test]
fn a_server_render_reports_no_system_notifications() {
    fn app() -> Element {
        let notifier = use_system_notification();
        let push = use_push_subscription(PushOptions::default());
        rsx! {
            p { id: "supported", "{notifier.is_supported()}" }
            p { id: "permission", "{notifier.permission():?}" }
            p { id: "error", "{notifier.error():?}" }
            p { id: "pending", "{notifier.is_pending()}" }
            p { id: "push-supported", "{push.is_supported()}" }
            p { id: "push-permission", "{push.permission():?}" }
            p { id: "subscription", "{push.subscription():?}" }
        }
    }

    let html = body(&render(app));

    for expected in [
        r#"<p id="supported">false</p>"#,
        r#"<p id="permission">Unsupported</p>"#,
        r#"<p id="error">None</p>"#,
        r#"<p id="pending">false</p>"#,
        r#"<p id="push-supported">false</p>"#,
        r#"<p id="push-permission">Unsupported</p>"#,
        r#"<p id="subscription">None</p>"#,
    ] {
        assert!(html.contains(expected), "{expected} missing in {html}");
    }
}
