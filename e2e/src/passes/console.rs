//! Console and page errors, recorded from fixture open to the end of the test.
//! Both portal bugs in `codebase/use-popover` were loud here and silent in the DOM.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::js_protocol::runtime::{EventConsoleApiCalled, EventExceptionThrown};
use futures::StreamExt;
use std::sync::{Arc, Mutex};

/// Everything the page said while the recorder was alive.
#[derive(Clone, Default)]
pub struct Recorder {
    messages: Arc<Mutex<Vec<String>>>,
}

impl Recorder {
    /// Subscribe to the page's console and uncaught exceptions.
    pub async fn attach(page: &Page) -> Result<Self> {
        let recorder = Recorder::default();

        let mut errors = page.event_listener::<EventExceptionThrown>().await?;
        let sink = recorder.messages.clone();
        tokio::spawn(async move {
            while let Some(event) = errors.next().await {
                let detail = &event.exception_details;
                let text = detail
                    .exception
                    .as_ref()
                    .and_then(|e| e.description.clone())
                    .unwrap_or_else(|| detail.text.clone());
                sink.lock().unwrap().push(format!("pageerror: {text}"));
            }
        });

        let mut logs = page.event_listener::<EventConsoleApiCalled>().await?;
        let sink = recorder.messages.clone();
        tokio::spawn(async move {
            while let Some(event) = logs.next().await {
                let level = format!("{:?}", event.r#type).to_lowercase();
                let args: Vec<&serde_json::Value> =
                    event.args.iter().filter_map(|a| a.value.as_ref()).collect();
                let Some((level, text)) = classify(&level, &args) else {
                    continue;
                };
                sink.lock().unwrap().push(format!("{level}: {text}"));
            }
        });

        Ok(recorder)
    }

    /// Look without draining, e.g. to wait for a message to arrive.
    pub fn peek(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }

    pub fn drain(&self) -> Vec<String> {
        std::mem::take(&mut *self.messages.lock().unwrap())
    }

    /// Fail if the page said anything at warning level or above, except what
    /// `BENIGN` allows. An allowed message is still printed, with its reason.
    pub fn assert_clean(&self, during: &str) -> Result<()> {
        let mut messages = self.drain();
        messages.retain(|message| match benign(message, BENIGN) {
            Some(allowed) => {
                eprintln!(
                    "console: allowed during {during} ({}): {message}",
                    allowed.reason
                );
                false
            }
            None => true,
        });
        if !messages.is_empty() {
            bail!(
                "the page reported errors or warnings during {during}:\n  {}",
                messages.join("\n  ")
            );
        }
        Ok(())
    }
}

/// A console message known to be harmless, with the reason a reviewer can audit.
pub struct Benign {
    /// Matched as a substring of the recorded message.
    pub pattern: &'static str,
    /// Why this is not a defect, and the todo that tracks it if there is one.
    pub reason: &'static str,
}

/// Empty on purpose: no run on 2026-09-19 logged a warning or an error
/// (Olaf102). Add an entry only with a reason a reviewer can check.
pub const BENIGN: &[Benign] = &[];

/// The recorded `dioxus_signals` warnings: a value read outside its owner's
/// scope, which may be dropped under the reader.
pub fn signal_warnings(messages: &[String]) -> Vec<&str> {
    messages
        .iter()
        .filter(|message| message.starts_with("tracing.warn") && message.contains("signals/src/"))
        .map(String::as_str)
        .collect()
}

/// The first entry of `list` that allows `message`.
pub fn benign<'a>(message: &str, list: &'a [Benign]) -> Option<&'a Benign> {
    list.iter().find(|entry| message.contains(entry.pattern))
}

/// Which messages count: `error`, `assert`, `warning` (review 7 E5), and dioxus tracing's
/// `%cWARN%c`/`%cERROR%c` markers, which arrive through `console.log` (todo 283).
fn classify(level: &str, args: &[&serde_json::Value]) -> Option<(String, String)> {
    let joined = || {
        args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    };
    match level {
        "error" | "assert" | "warning" => Some((format!("console.{level}"), joined())),
        _ => {
            let first = args.first()?.as_str()?;
            let traced = ["ERROR", "WARN"]
                .into_iter()
                .find(|marker| first.starts_with(&format!("%c{marker}%c")))?;
            Some((
                format!("tracing.{}", traced.to_lowercase()),
                first.replace("%c", ""),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_tracing_warning_through_console_log_counts() {
        let first =
            json!("%cWARN%c signals/src/warnings.rs:26%c A Copy Value created in ScopeId(16)");
        let css = json!("color: orange");
        let (level, text) = classify("log", &[&first, &css]).unwrap();
        assert_eq!(level, "tracing.warn");
        assert_eq!(
            text,
            "WARN signals/src/warnings.rs:26 A Copy Value created in ScopeId(16)"
        );
    }

    #[test]
    fn tracing_info_and_plain_logs_do_not_count() {
        assert!(classify("log", &[&json!("%cINFO%c x%c y")]).is_none());
        assert!(classify("debug", &[&json!("%cDEBUG%c x%c y")]).is_none());
        assert!(classify("log", &[&json!("WARN is just a word here")]).is_none());
        assert!(classify("log", &[]).is_none());
    }

    #[test]
    fn a_signal_warning_is_told_apart() {
        let messages = [
            "tracing.warn: WARN signals/src/warnings.rs:26 A Copy Value created".to_owned(),
            "tracing.warn: WARN libero/src/x.rs:1 something else".to_owned(),
            "console.warning: signals/src/ in a plain warning".to_owned(),
        ];
        assert_eq!(signal_warnings(&messages), [messages[0].as_str()]);
    }

    #[test]
    fn a_console_warning_counts() {
        let (level, _) = classify("warning", &[&json!("careful")]).unwrap();
        assert_eq!(level, "console.warning");
    }

    #[test]
    fn an_allowed_message_names_its_reason() {
        let list = [Benign {
            pattern: "known noise",
            reason: "todo 0: example",
        }];
        assert_eq!(
            benign("tracing.warn: some known noise here", &list).map(|b| b.reason),
            Some("todo 0: example")
        );
        assert!(benign("tracing.warn: something else", &list).is_none());
    }
}
