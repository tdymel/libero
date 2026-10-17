use serde_json::json;

use super::{eval_with, runs_scripts};
use crate::platform::waveform::{PEAKS_SCRIPT, Peaks, WaveformApi};

pub(crate) fn waveform() -> Option<&'static dyn WaveformApi> {
    runs_scripts().then_some(&WAVEFORM as &'static dyn WaveformApi)
}

struct WebViewWaveform;

static WAVEFORM: WebViewWaveform = WebViewWaveform;

impl WaveformApi for WebViewWaveform {
    fn peaks(&self, src: &str, bars: usize) -> Peaks {
        let script = eval_with(
            json!([src, bars]),
            &format!(
                "{PEAKS_SCRIPT}
                const [src, bars] = data;
                try {{
                    return await peaks(src, bars);
                }} catch (error) {{
                    return null;
                }}"
            ),
        );
        Box::pin(async move { script.join::<Option<Vec<f64>>>().await.ok().flatten() })
    }
}
