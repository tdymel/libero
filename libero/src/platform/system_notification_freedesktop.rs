//! Blitz on Linux: the freedesktop notification server over D-Bus (todo 1348).

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::poll_fn;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{OnceLock, mpsc as std_mpsc};
use std::time::Duration;

use dbus::arg::{PropMap, RefArg, Variant};
use dbus::blocking::LocalConnection;
use dbus::message::MatchRule;
use dioxus::core::Task;
use dioxus::prelude::spawn;
use futures_channel::{mpsc, oneshot};
use futures_core::Stream;

use super::{
    Answer, NotificationEvent, Shown, ShownNotification, SystemNotification, SystemNotificationApi,
    SystemNotificationError,
};
use crate::platform::PermissionState;

const DESTINATION: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
const INTERFACE: &str = "org.freedesktop.Notifications";
/// A server that is not running gets D-Bus activated within this.
const TIMEOUT: Duration = Duration::from_secs(2);
/// How long a command waits for the bus thread at most.
const TICK: Duration = Duration::from_millis(50);

enum Command {
    Probe(oneshot::Sender<bool>),
    Show {
        notification: Content,
        events: mpsc::UnboundedSender<NotificationEvent>,
        reply: oneshot::Sender<Option<u32>>,
    },
    Close(u32),
}

/// What `Notify` sends; `SystemNotification` holds a `Callback`, which is not `Send`.
struct Content {
    title: String,
    body: String,
    icon: String,
    tag: Option<String>,
    silent: bool,
}

/// The server's ids by tag, and where each id's events go.
#[derive(Default)]
struct Routes {
    tags: HashMap<String, u32>,
    events: HashMap<u32, mpsc::UnboundedSender<NotificationEvent>>,
}

impl Routes {
    /// The id a same-tag notification had, so the server replaces it in place.
    fn replaces(&self, tag: Option<&str>) -> u32 {
        tag.and_then(|tag| self.tags.get(tag).copied()).unwrap_or(0)
    }

    fn shown(
        &mut self,
        id: u32,
        tag: Option<String>,
        events: mpsc::UnboundedSender<NotificationEvent>,
    ) {
        if let Some(tag) = tag {
            self.tags.insert(tag, id);
        }
        self.events.insert(id, events);
    }

    fn send(&mut self, id: u32, event: NotificationEvent) {
        let delivered = self
            .events
            .get(&id)
            .is_some_and(|events| events.unbounded_send(event).is_ok());
        if !delivered || event == NotificationEvent::Close {
            self.events.remove(&id);
            self.tags.retain(|_, shown| *shown != id);
        }
    }
}

/// The bus thread: `Notify` and the signals must share one connection, as
/// GNOME sends `ActionInvoked` to the sender only.
fn commands() -> &'static std_mpsc::Sender<Command> {
    static COMMANDS: OnceLock<std_mpsc::Sender<Command>> = OnceLock::new();
    COMMANDS.get_or_init(|| {
        let (sender, receiver) = std_mpsc::channel();
        std::thread::spawn(move || run(receiver));
        sender
    })
}

fn run(commands: std_mpsc::Receiver<Command>) {
    let connection = LocalConnection::new_session().ok();
    let routes = Rc::new(RefCell::new(Routes::default()));
    if let Some(connection) = &connection {
        let clicks = routes.clone();
        let invoked = MatchRule::new_signal(INTERFACE, "ActionInvoked");
        let _ = connection.add_match(invoked, move |(id, action): (u32, String), _, _| {
            if action == "default" {
                clicks.borrow_mut().send(id, NotificationEvent::Click);
            }
            true
        });
        let closes = routes.clone();
        let closed = MatchRule::new_signal(INTERFACE, "NotificationClosed");
        let _ = connection.add_match(closed, move |(id, _reason): (u32, u32), _, _| {
            closes.borrow_mut().send(id, NotificationEvent::Close);
            true
        });
    }
    loop {
        match commands.recv_timeout(TICK) {
            Ok(command) => handle(connection.as_ref(), &routes, command),
            Err(std_mpsc::RecvTimeoutError::Timeout) => {}
            Err(std_mpsc::RecvTimeoutError::Disconnected) => return,
        }
        if let Some(connection) = &connection {
            let _ = connection.process(Duration::ZERO);
        }
    }
}

fn handle(connection: Option<&LocalConnection>, routes: &RefCell<Routes>, command: Command) {
    let proxy = connection.map(|connection| connection.with_proxy(DESTINATION, PATH, TIMEOUT));
    match command {
        Command::Probe(reply) => {
            let answered = proxy.is_some_and(|proxy| {
                let info: Result<(String, String, String, String), _> =
                    proxy.method_call(INTERFACE, "GetServerInformation", ());
                info.is_ok()
            });
            let _ = reply.send(answered);
        }
        Command::Show {
            notification,
            events,
            reply,
        } => {
            let Some(proxy) = proxy else {
                let _ = reply.send(None);
                return;
            };
            let replaces = routes.borrow().replaces(notification.tag.as_deref());
            let mut hints = PropMap::new();
            if notification.silent {
                hints.insert(
                    "suppress-sound".into(),
                    Variant(Box::new(true) as Box<dyn RefArg>),
                );
            }
            let shown: Result<(u32,), _> = proxy.method_call(
                INTERFACE,
                "Notify",
                (
                    app_name(),
                    replaces,
                    notification.icon,
                    notification.title,
                    notification.body,
                    vec!["default", ""],
                    hints,
                    -1i32,
                ),
            );
            let id = shown.ok().map(|(id,)| id);
            if let Some(id) = id {
                routes.borrow_mut().shown(id, notification.tag, events);
            }
            let _ = reply.send(id);
        }
        Command::Close(id) => {
            if let Some(proxy) = proxy {
                let _: Result<(), _> = proxy.method_call(INTERFACE, "CloseNotification", (id,));
            }
        }
    }
}

fn app_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| Some(path.file_stem()?.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

pub(super) struct FreedesktopNotification;

pub(super) static SYSTEM_NOTIFICATION: FreedesktopNotification = FreedesktopNotification;

/// The desktop asks no permission: a running server means `Granted`.
impl SystemNotificationApi for FreedesktopNotification {
    fn probe(&self) -> Answer<Option<PermissionState>> {
        let (reply, answer) = oneshot::channel();
        let _ = commands().send(Command::Probe(reply));
        Box::pin(async move {
            answer
                .await
                .unwrap_or(false)
                .then_some(PermissionState::Granted)
        })
    }

    fn request(&self) -> Answer<PermissionState> {
        let probe = self.probe();
        Box::pin(async move { probe.await.unwrap_or(PermissionState::Unsupported) })
    }

    /// The window is not raised on a click: `platform` holds no Blitz window.
    fn show(
        &self,
        notification: &SystemNotification,
        events: Box<dyn Fn(NotificationEvent)>,
    ) -> Shown {
        let (reply, answer) = oneshot::channel();
        let (sender, mut receiver) = mpsc::unbounded();
        let _ = commands().send(Command::Show {
            notification: Content {
                title: notification.title.clone(),
                body: notification.body.clone().unwrap_or_default(),
                icon: notification.icon.clone().unwrap_or_default(),
                tag: notification.tag.clone(),
                silent: notification.silent,
            },
            events: sender,
            reply,
        });
        Box::pin(async move {
            let id = answer
                .await
                .ok()
                .flatten()
                .ok_or(SystemNotificationError::Failed)?;
            let task = spawn(async move {
                while let Some(event) = poll_fn(|cx| Pin::new(&mut receiver).poll_next(cx)).await {
                    events(event);
                }
            });
            Ok(Box::new(FreedesktopShown { id, task }) as Box<dyn ShownNotification>)
        })
    }
}

/// Dropping it ends the task; the bus thread forgets the id on its next event.
struct FreedesktopShown {
    id: u32,
    task: Task,
}

impl ShownNotification for FreedesktopShown {
    fn close(&self) {
        let _ = commands().send(Command::Close(self.id));
    }
}

impl Drop for FreedesktopShown {
    fn drop(&mut self) {
        self.task.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tag_replaces_its_notification_until_it_closes() {
        let mut routes = Routes::default();
        let (sender, mut receiver) = mpsc::unbounded();
        assert_eq!(routes.replaces(Some("build")), 0);
        routes.shown(7, Some("build".into()), sender);
        assert_eq!(routes.replaces(Some("build")), 7);
        assert_eq!(routes.replaces(None), 0);
        routes.send(7, NotificationEvent::Click);
        assert_eq!(receiver.try_recv().ok(), Some(NotificationEvent::Click));
        routes.send(7, NotificationEvent::Close);
        assert_eq!(receiver.try_recv().ok(), Some(NotificationEvent::Close));
        assert_eq!(routes.replaces(Some("build")), 0);
        assert!(routes.events.is_empty());
    }

    #[test]
    fn a_dropped_listener_is_forgotten() {
        let mut routes = Routes::default();
        let (sender, receiver) = mpsc::unbounded();
        routes.shown(3, Some("sync".into()), sender);
        drop(receiver);
        routes.send(3, NotificationEvent::Click);
        assert!(routes.events.is_empty());
        assert_eq!(routes.replaces(Some("sync")), 0);
    }
}
