use super::*;

/// The trigger, below the host so its handle feeds it. Its own component: `Demo` calls
/// `render` in its own scope, where the hooks would be invisible.
#[component]
#[allow(clippy::too_many_arguments)]
pub(super) fn Examples(
    color: String,
    variant: String,
    title: bool,
    closable: bool,
    live: String,
    template: String,
    position: String,
    auto_close: String,
    contained: bool,
) -> Element {
    let notify = use_notifications();
    let cards = use_notifications_with(card_notification);
    let next_sender = use_hook(|| Rc::new(Cell::new(0)));
    let mut last = use_signal(String::new);
    let onaction = use_callback(move |action: String| last.set(action));
    let uploads = use_notifications_with(upload_notification);
    // Each upload's ticker. Dropped with the preview, which is what stops them.
    let tickers = use_hook(|| {
        Rc::new(std::cell::RefCell::new(Vec::<
            std::boxed::Box<dyn TimerSubscription>,
        >::new()))
    });

    let color: Input<_> = Input::from(color.as_str());
    let variant = Input::<Variant>::from(variant.as_str()).copied_or(Variant::Tonal);
    let live = match live.as_str() {
        "assertive" => NotificationLive::Assertive,
        _ => NotificationLive::Polite,
    };
    // A contained host answers both, so only the app-wide case names them per
    // notification - the one this site renders near the root has the defaults.
    let host_answers = contained;
    let placement = Placement::from(position.as_str());
    let auto_close = auto_close_of(&auto_close);
    let options = move |sticky: bool| NotificationOptions {
        placement: (!host_answers).then_some(placement),
        auto_close: match sticky {
            true => Some(AutoClose::Never),
            false => (!host_answers).then_some(auto_close),
        },
        closable,
        live,
    };

    let show_upload = move || {
        let id = uploads.show_with(
            Upload {
                file: "archive.zip",
                percent: 0.0,
            },
            options(true),
        );
        let Some(api) = timer() else {
            return;
        };
        let percent = Cell::new(0.0);
        // The callback runs outside every scope, so it only writes - `update`
        // is a signal write and nothing else.
        let ticker = api.every(
            Duration::from_millis(300),
            std::boxed::Box::new(move || {
                if percent.get() < 100.0 {
                    percent.set(percent.get() + 10.0);
                    uploads.update(
                        id,
                        Upload {
                            file: "archive.zip",
                            percent: percent.get(),
                        },
                    );
                }
            }),
        );
        tickers.borrow_mut().push(ticker);
    };

    let upload = template == "upload";
    let show = move |_| {
        if upload {
            show_upload();
            return;
        }
        if template == "card" {
            let (sender, initials, text) = SENDERS[next_sender.get() % SENDERS.len()];
            next_sender.set(next_sender.get() + 1);
            cards.show_with(
                Message {
                    sender,
                    initials,
                    title: title.then(|| "Saved".into()),
                    text,
                    onaction,
                },
                // Its Reply and Mute are actions, so it waits for the reader (WCAG 2.2.1).
                options(true),
            );
            return;
        }
        notify.show_with(
            NotificationData {
                title: title.then(|| "Saved".into()),
                message: "Your changes are safe.".into(),
                color: color.clone(),
                variant: Input::Value(variant),
                ..Default::default()
            },
            options(false),
        );
    };

    rsx! {
        // At the top: the stacks default to the bottom edge, so on a phone the
        // buttons leave them room.
        Flex {
            direction: "column",
            align: "center",
            justify: "start",
            gap: "sm",
            sx: sx().height(match contained {
                true => "420px",
                false => "auto",
            }),
            Flex { direction: "row", justify: "center", gap: "sm", wrap: "wrap",
                Button { variant: "outlined", onclick: show, "Show one" }
                Button { variant: "standard", onclick: move |_| {
                        notify.clear();
                        cards.clear();
                    },
                    "Clear all" }
            }
            // Always mounted, so a screen reader hears what Reply or Mute did.
            Text { size: "sm", role: "status", "{last}" }
            Text { size: "sm",
                if contained {
                    "This host is contained, so it draws its stacks in its own box. The "
                    "preview needs that, an app does not."
                } else {
                    "Without contained, this goes to the one app-wide host near the root. "
                    "Look at the edge of the window."
                }
            }
        }
    }
}
