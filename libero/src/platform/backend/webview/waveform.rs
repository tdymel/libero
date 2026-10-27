use serde_json::json;

use super::{Slot, eval_with, runs_scripts};
use crate::platform::waveform::{PEAKS_SCRIPT, Peaks, WaveformApi};

pub(crate) fn waveform() -> Option<&'static dyn WaveformApi> {
    runs_scripts().then_some(&WAVEFORM as &'static dyn WaveformApi)
}

struct WebViewWaveform;

static WAVEFORM: WebViewWaveform = WebViewWaveform;

impl WaveformApi for WebViewWaveform {
    /// Parked, not `join`ed: a large file can decode past `CHANNEL_HOLD_MS`.
    fn peaks(&self, src: &str, bars: usize) -> Peaks {
        let slot = Slot::new();
        let script = eval_with(
            json!([src, bars]),
            &format!(
                "{PEAKS_SCRIPT}
                const [src, bars] = data;
                try {{
                    dioxus.send(await peaks(src, bars));
                }} catch (error) {{
                    dioxus.send(null);
                }}
                {}",
                slot.park("")
            ),
        );
        Box::pin(async move {
            let mut script = script;
            let answer = script.recv::<Option<Vec<f64>>>().await.ok().flatten();
            drop(slot);
            answer
        })
    }
}
