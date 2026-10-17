use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::json;

use super::{Slot, WebViewListener, eval_with, runs_scripts};
use crate::platform::Read;
use crate::platform::permission::{
    PermissionApi, PermissionKind, PermissionState, PermissionSubscription,
};

pub(crate) fn permission() -> Option<&'static dyn PermissionApi> {
    runs_scripts().then_some(&PERMISSIONS as &'static dyn PermissionApi)
}

struct WebViewPermissions;

static PERMISSIONS: WebViewPermissions = WebViewPermissions;

/// Binds `status`, `null` where the query rejects or the WebView has no Permissions API.
const PERMISSION_STATUS: &str = "let status = null;
    try { status = await navigator.permissions.query({ name: data }); } catch (error) {}";

impl PermissionApi for WebViewPermissions {
    fn query(&self, kind: PermissionKind) -> Read<PermissionState> {
        let read = eval_with(
            json!(kind.name()),
            &format!("{PERMISSION_STATUS}\nreturn status?.state ?? null;"),
        );
        Box::pin(async move {
            let state = read.join::<Option<String>>().await.ok().flatten();
            Ok(state.map_or(PermissionState::Unknown, |state| {
                PermissionState::from_name(&state)
            }))
        })
    }

    fn on_change(
        &self,
        kind: PermissionKind,
        callback: Box<dyn Fn(PermissionState)>,
    ) -> Box<dyn PermissionSubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!(kind.name()),
            &format!(
                "{PERMISSION_STATUS}
                const send = () => dioxus.send(status?.state ?? null);
                send();
                status?.addEventListener('change', send);
                {}
                status?.removeEventListener('change', send);",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(state) = script.recv::<Option<String>>().await {
                callback(state.map_or(PermissionState::Unknown, |state| {
                    PermissionState::from_name(&state)
                }));
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

impl PermissionSubscription for WebViewListener {}
