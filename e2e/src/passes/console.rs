//! Console and page errors.
//!
//! The cheapest pass in the suite and, per line, the most valuable. Both
//! portal bugs recorded in `codebase/use-popover` were loud in the console and
//! silent in the DOM: a test asserting only on markup passed while the
//! component was broken.
//!
//! Collection starts when the fixture opens and is drained at the end, so an
//! error raised during an interaction is caught, not just one raised at mount.

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
                // Only the levels that mean something is wrong. dioxus logs
                // freely at `log` and `debug` in a debug build, and failing on
                // those would make the pass noise rather than signal.
                let level = format!("{:?}", event.r#type).to_lowercase();
                if level != "error" && level != "assert" {
                    continue;
                }
                let text = event
                    .args
                    .iter()
                    .filter_map(|a| a.value.as_ref().map(|v| v.to_string()))
                    .collect::<Vec<_>>()
                    .join(" ");
                sink.lock()
                    .unwrap()
                    .push(format!("console.{level}: {text}"));
            }
        });

        Ok(recorder)
    }

    /// Look without draining, for a caller that wants to wait for a message to
    /// arrive before reading it.
    pub fn peek(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }

    pub fn drain(&self) -> Vec<String> {
        std::mem::take(&mut *self.messages.lock().unwrap())
    }

    /// Fail if the page said anything at error level.
    pub fn assert_clean(&self, during: &str) -> Result<()> {
        let messages = self.drain();
        if !messages.is_empty() {
            bail!(
                "the page reported errors during {during}:\n  {}",
                messages.join("\n  ")
            );
        }
        Ok(())
    }
}
