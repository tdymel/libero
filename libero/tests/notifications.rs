//! `Notifications`' rendered contract: two live regions per stack that exist
//! before anything is announced into them, the limit and its queue, the
//! options, and the timers that close a notification on their own.

mod common;

use std::thread;
use std::time::{Duration, Instant};

use common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        NotificationData, NotificationLive, NotificationOptions, NotificationScope, Notifications,
        Placement, use_notifications, use_notifications_with,
    },
    theme::{AutoClose, NotificationDefaults, Theme},
};

/// Each `<li>` in `html`, open tag to close tag.
fn items(html: &str) -> Vec<&str> {
    html.match_indices("<li")
        .map(|(start, _)| {
            let rest = &html[start..];
            &rest[..rest.find("</li>").expect("an unterminated li")]
        })
        .collect()
}

/// The markup of the stack anchored at `placement`: the `div` whose
/// `data-state` names both of its edges, up to the next stack.
fn stack<'a>(html: &'a str, vertical: &str, horizontal: &str) -> &'a str {
    let marker = format!("vertical-{vertical} horizontal-{horizontal}");
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("no {marker} stack:\n{html}"));
    let rest = &html[start..];
    let end = rest[marker.len()..]
        .find("vertical-")
        .map_or(rest.len(), |end| end + marker.len());
    &rest[..end]
}

#[test]
fn every_stack_renders_both_live_regions_before_anything_is_shown() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Notifications {} }
        }
    }

    let html = body(&render(app));

    assert_eq!(html.matches(r#"aria-live="polite""#).count(), 9, "{html}");
    assert_eq!(
        html.matches(r#"aria-live="assertive""#).count(),
        9,
        "{html}"
    );
    assert!(items(&html).is_empty(), "{html}");
}

#[test]
fn a_shown_message_renders_as_an_alert_in_the_default_corner() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| notify.show("Saved."));
        rsx! {
            LiberoProvider { Notifications {} }
        }
    }

    // `use_notifications` sits above `LiberoProvider` here, so the store is
    // the root's either way.
    let html = body(&render(app));
    let bottom_end = stack(&html, "bottom", "end");
    let shown = items(bottom_end);

    assert_eq!(shown.len(), 1, "{html}");
    assert!(shown[0].contains("Saved."), "{}", shown[0]);
    // Not `alert`: the list is already the live region.
    assert!(shown[0].contains(r#"role="group""#), "{}", shown[0]);
    assert!(shown[0].contains(r#"aria-label="Close""#), "{}", shown[0]);
}

#[test]
fn a_title_without_a_message_names_nothing_it_does_not_have() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| {
            notify.show(NotificationData {
                title: Some("Uploaded".into()),
                ..Default::default()
            })
        });
        rsx! {
            LiberoProvider { Notifications {} }
        }
    }

    let html = body(&render(app));
    let shown = items(&html);

    assert_eq!(shown.len(), 1, "{html}");
    assert!(shown[0].contains("aria-labelledby"), "{}", shown[0]);
    assert!(!shown[0].contains("aria-describedby"), "{}", shown[0]);
}

#[test]
fn the_options_pick_the_stack_the_region_and_the_close_button() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| {
            notify.show_with(
                "Connection lost",
                NotificationOptions {
                    position: Some(Placement::TopStart),
                    live: NotificationLive::Assertive,
                    closable: false,
                    ..Default::default()
                },
            )
        });
        rsx! {
            LiberoProvider { Notifications {} }
        }
    }

    let html = body(&render(app));
    let top_start = stack(&html, "top", "start");

    let assertive = &top_start[top_start.find(r#"aria-live="assertive""#).unwrap()..];
    let assertive = &assertive[..assertive.find("</ol>").unwrap()];
    assert!(assertive.contains("Connection lost"), "{top_start}");
    assert!(!assertive.contains("<button"), "{assertive}");
    assert!(items(stack(&html, "bottom", "end")).is_empty(), "{html}");
}

#[test]
fn a_stack_shows_its_limit_and_queues_the_rest() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| {
            for n in 0..4 {
                notify.show(format!("Message {n}"));
            }
        });
        rsx! {
            LiberoProvider { Notifications { limit: 3 } }
        }
    }

    let html = body(&render(app));
    let shown = items(&html);

    assert_eq!(shown.len(), 3, "{html}");
    // In order: the oldest are the ones shown.
    assert!(shown[0].contains("Message 0"), "{}", shown[0]);
    assert!(!html.contains("Message 3"), "{html}");
}

#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: u8,
}

fn upload_template(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    rsx! {
        p { "{upload.file}: {upload.percent}%" }
    }
}

#[test]
fn a_custom_template_draws_its_own_data_and_update_replaces_it() {
    fn app() -> Element {
        let uploads = use_notifications_with(upload_template);
        use_hook(|| {
            let id = uploads.show(Upload {
                file: "archive.zip",
                percent: 0,
            });
            uploads.update(
                id,
                Upload {
                    file: "archive.zip",
                    percent: 40,
                },
            );
        });
        rsx! {
            LiberoProvider { Notifications {} }
        }
    }

    let html = body(&render(app));
    let shown = items(&html);

    assert_eq!(shown.len(), 1, "{html}");
    assert!(shown[0].contains("archive.zip: 40%"), "{}", shown[0]);
}

/// Short enough that a test waits milliseconds, not seconds.
static FAST: Theme = Theme {
    notification: NotificationDefaults {
        auto_close: AutoClose::After(30),
        transition_duration: 30,
        ..Theme::DEFAULT.notification
    },
    ..Theme::DEFAULT
};

/// Polls `dom` until `done` holds for its rendered markup, or `limit` runs
/// out. `process_events` drains the task a timer delivers through, and
/// `render_immediate` applies what it wrote.
fn drive_until(dom: &mut VirtualDom, limit: Duration, done: impl Fn(&str) -> bool) -> String {
    let start = Instant::now();
    loop {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = body(&dioxus_ssr::render(dom));
        if done(&html) || start.elapsed() > limit {
            return html;
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn a_notification_leaves_after_its_time_and_is_then_removed() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| notify.show("Saved."));
        rsx! {
            LiberoProvider { theme: &FAST, Notifications {} }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();

    let html = drive_until(&mut dom, Duration::from_secs(2), |html| {
        html.contains(r#"data-state="leaving""#)
    });
    assert!(html.contains(r#"data-state="leaving""#), "{html}");

    let html = drive_until(&mut dom, Duration::from_secs(2), |html| {
        items(html).is_empty()
    });
    assert!(items(&html).is_empty(), "{html}");
}

#[test]
fn a_sticky_notification_stays_and_the_next_one_moves_up() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| {
            notify.show("Goes");
            notify.show_with(
                "Stays",
                NotificationOptions {
                    auto_close: Some(AutoClose::Never),
                    ..Default::default()
                },
            );
            notify.show("Queued");
        });
        rsx! {
            LiberoProvider { theme: &FAST, Notifications { limit: 2 } }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();

    // "Goes" closes and is removed; "Queued" takes its place and closes in
    // turn, having started its own timer only once shown.
    let html = drive_until(&mut dom, Duration::from_secs(3), |html| {
        items(html).len() == 1
    });
    assert!(!html.contains("Goes") && !html.contains("Queued"), "{html}");
    assert!(html.contains("Stays"), "{html}");
}

#[test]
fn hiding_a_queued_notification_removes_it_without_ever_showing_it() {
    fn app() -> Element {
        let notify = use_notifications();
        use_hook(|| {
            notify.show("First");
            let queued = notify.show("Second");
            notify.hide(queued);
        });
        rsx! {
            LiberoProvider { theme: &FAST, Notifications { limit: 1 } }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();

    // Merely marked as leaving, "Second" would take the freed slot once
    // "First" closes, and run an exit it never needed.
    let html = drive_until(&mut dom, Duration::from_secs(2), |html| {
        html.contains("Second") || items(html).is_empty()
    });
    assert!(items(&html).is_empty(), "{html}");
    assert!(!html.contains("Second"), "{html}");
}

#[component]
fn RaiseSaved() -> Element {
    let notify = use_notifications();
    use_hook(|| notify.show("Saved."));
    rsx! {}
}

#[component]
fn RaiseUpload() -> Element {
    let uploads = use_notifications_with(upload_template);
    use_hook(|| {
        uploads.show(Upload {
            file: "archive.zip",
            percent: 10,
        })
    });
    rsx! {}
}

/// The store is created by whichever scope asks first, in the root. Two
/// raisers in unrelated branches and a host in a third must all see one queue.
#[test]
fn unrelated_scopes_share_one_store() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                div { RaiseSaved {} }
                section { RaiseUpload {} }
                aside { Notifications {} }
            }
        }
    }

    let html = body(&render(app));
    let shown = items(&html);

    assert_eq!(shown.len(), 2, "{html}");
    assert!(shown[0].contains("Saved."), "{html}");
    assert!(shown[1].contains("archive.zip: 10%"), "{html}");
}

/// Nothing about the store needs `LiberoProvider`: without one, and without a
/// host, `show` just queues.
#[test]
fn showing_never_panics_without_a_provider() {
    fn app() -> Element {
        rsx! {
            RaiseSaved {}
            RaiseUpload {}
        }
    }

    render(app);
}
