//! `use_user_media` on a server render: no capture API and no prompt; the
//! state is read in an effect, never during render (hydration).

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::hooks::{use_user_media, use_user_media_devices};

#[test]
fn a_server_render_reports_no_capture() {
    fn app() -> Element {
        let media = use_user_media(Default::default());
        let devices = use_user_media_devices();
        rsx! {
            video { ..media.attributes() }
            p { id: "supported", "{media.is_supported()}" }
            p { id: "camera", "{media.camera_permission():?}" }
            p { id: "live", "{media.is_live()}" }
            p { id: "error", "{media.error():?}" }
            p { id: "cameras", "{devices.cameras().len()}" }
        }
    }

    let html = body(&render(app));

    for expected in [
        r#"<p id="supported">false</p>"#,
        r#"<p id="camera">Unsupported</p>"#,
        r#"<p id="live">false</p>"#,
        r#"<p id="error">None</p>"#,
        r#"<p id="cameras">0</p>"#,
    ] {
        assert!(html.contains(expected), "{expected} missing in {html}");
    }
    assert!(
        html.contains("<video data-lsx-capture="),
        "untagged preview in {html}"
    );
}
