//! A chart's drawing as a file of its own: SVG as drawn, or PNG through the page's canvas.

use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

use super::{engine::Drawn, zoom::Zoom};
use crate::{
    platform::{SaveOutcome, save_file, svg_png},
    theme::{HexColor, Theme},
};

/// A PNG's pixels per SVG unit: sharp on a high-density screen and in print.
const PNG_SCALE: f32 = 2.0;

/// What a chart hands its handle and its export buttons: a fresh drawing in the normal
/// palette, forced colours or not, and what makes it standalone.
#[derive(Clone)]
pub(super) struct ExportSource {
    pub(super) draw: Rc<dyn Fn() -> Option<Drawn>>,
    pub(super) title: Option<String>,
    pub(super) background: HexColor,
    pub(super) font: &'static str,
}

impl ExportSource {
    pub(super) fn of(
        theme: &Theme,
        title: Option<String>,
        draw: impl Fn() -> Option<Drawn> + 'static,
    ) -> Self {
        Self {
            draw: Rc::new(draw),
            title,
            background: theme.surface,
            font: theme.text.font_family,
        }
    }

    /// The SVG and its size in px.
    pub(super) fn svg(&self) -> Option<(String, f32, f32)> {
        let drawn = (self.draw)()?;
        let (width, height) = (drawn.geometry.width, drawn.geometry.height);
        let svg = standalone(
            &drawn.svg,
            width,
            height,
            self.title.as_deref(),
            self.background,
            self.font,
        );
        Some((svg, width, height))
    }

    pub(super) async fn png(&self) -> Option<Vec<u8>> {
        let (svg, width, height) = self.svg()?;
        svg_png(svg, width, height, PNG_SCALE).await
    }
}

/// The page's drawing as a file: sized, namespaced, on its background, titled, in a
/// named font. Its colours are hex already.
pub(super) fn standalone(
    svg: &str,
    width: f32,
    height: f32,
    title: Option<&str>,
    background: HexColor,
    font: &str,
) -> String {
    let Some(open) = svg.find("<svg") else {
        return svg.to_string();
    };
    let close = svg[open..]
        .find('>')
        .map_or(svg.len(), |close| open + close);
    let root = svg[open..close].trim_end_matches('/').replace(
        " width=\"100%\" height=\"100%\" aria-hidden=\"true\" focusable=\"false\"",
        "",
    );
    let xmlns = match root.contains("xmlns=") {
        true => "",
        false => " xmlns=\"http://www.w3.org/2000/svg\"",
    };
    let title = title
        .map(|title| format!("<title>{}</title>", escape(title)))
        .unwrap_or_default();
    // Quotes would end the attribute; a `var()` means nothing outside the page.
    let font = match font.contains("var(") {
        true => "sans-serif".to_string(),
        false => escape(&font.replace('"', "'")),
    };
    let body = svg[close + 1..].replace(
        "font-family=\"inherit\"",
        &format!("font-family=\"{font}\""),
    );
    format!(
        "<svg width=\"{width}\" height=\"{height}\"{xmlns}{}>{title}<rect width=\"100%\" height=\"100%\" fill=\"{background}\"/>{body}",
        &root[4..]
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A chart's drawing as an SVG or PNG file, and its zoom, from code. Made with
/// [`use_chart`], passed as a chart's `handle`.
#[derive(Clone, Copy, PartialEq)]
pub struct ChartHandle {
    source: CopyValue<Option<ExportSource>>,
    /// The chart that attached last, so a stale drop leaves a newer one's source.
    owner: CopyValue<Option<u64>>,
    zoom: Signal<Option<Zoom>>,
    /// The chart's zoom window, owned here so the page reads it without a warning.
    window: Signal<Option<(usize, usize)>>,
}

impl ChartHandle {
    pub(super) fn window(&self) -> Signal<Option<(usize, usize)>> {
        self.window
    }

    /// A fresh id for a chart that attaches.
    pub(super) fn new_owner() -> u64 {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    pub(super) fn attach(&self, owner: u64, source: ExportSource, zoom: Option<Zoom>) {
        let mut slot = self.source;
        if let Ok(mut slot) = slot.try_write() {
            *slot = Some(source);
        }
        let mut current = self.owner;
        if current.try_peek().is_ok_and(|held| *held != Some(owner))
            && let Ok(mut held) = current.try_write()
        {
            *held = Some(owner);
        }
        set_zoom(self.zoom, zoom);
    }

    /// On the chart's drop: its drawing and its zoom go with it, unless a newer chart
    /// attached since.
    pub(super) fn detach(&self, owner: u64) {
        let mut current = self.owner;
        if current
            .try_peek()
            .is_ok_and(|held| held.is_some_and(|held| held != owner))
        {
            return;
        }
        if let Ok(mut held) = current.try_write() {
            *held = None;
        }
        let mut source = self.source;
        if let Ok(mut slot) = source.try_write() {
            *slot = None;
        }
        set_zoom(self.zoom, None);
        let mut window = self.window;
        if window.try_peek().is_ok_and(|held| held.is_some())
            && let Ok(mut held) = window.try_write()
        {
            *held = None;
        }
    }

    fn source(&self) -> Option<ExportSource> {
        self.source.try_peek().ok()?.clone()
    }

    /// The chart as drawn now, a standalone SVG file's text; `None` before it draws.
    pub fn svg(&self) -> Option<String> {
        Some(self.source()?.svg()?.0)
    }

    /// The chart as PNG bytes at twice its size, drawn by the page. `None` before it
    /// draws, and where no page draws it: a server render, Blitz.
    pub async fn png(&self) -> Option<Vec<u8>> {
        self.source()?.png().await
    }

    /// Saves [`svg`](Self::svg) as `name` through [`save_file`].
    pub async fn save_svg(&self, name: &str) -> SaveOutcome {
        match self.svg() {
            Some(svg) => save_file(name, "image/svg+xml", svg.into_bytes()).await,
            None => SaveOutcome::Failed(NOTHING_DRAWN.to_string()),
        }
    }

    /// Saves [`png`](Self::png) as `name` through [`save_file`].
    pub async fn save_png(&self, name: &str) -> SaveOutcome {
        match self.png().await {
            Some(png) => save_file(name, "image/png", png).await,
            None => SaveOutcome::Failed(NOTHING_DRAWN.to_string()),
        }
    }

    /// Back to the whole x axis. Does nothing on a chart without `zoom`.
    pub fn reset_zoom(&self) {
        if let Some(zoom) = self.zoom.try_peek().ok().and_then(|zoom| *zoom) {
            zoom.reset();
        }
    }

    /// Whether the chart shows less than its whole x axis. Reactive: read in a component
    /// or a memo, it re-runs on every zoom, pan and reset; `false` without `zoom`.
    pub fn is_zoomed(&self) -> bool {
        let zoom = self.zoom.try_read().ok().and_then(|zoom| *zoom);
        zoom.is_some_and(|zoom| zoom.is_zoomed())
    }
}

/// Written only on a change: a reader of [`ChartHandle::is_zoomed`] re-runs on it.
fn set_zoom(mut slot: Signal<Option<Zoom>>, zoom: Option<Zoom>) {
    if slot.try_peek().is_ok_and(|held| *held != zoom)
        && let Ok(mut held) = slot.try_write()
    {
        *held = zoom;
    }
}

const NOTHING_DRAWN: &str = "the chart has no drawing";

/// A handle to a chart's drawing: pass it as the chart's `handle`, then save or read it, and
/// reset its zoom. The chart draws no buttons; render your own.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, LineChart, Series, use_chart};
/// # fn app() -> Element {
/// let chart = use_chart();
/// rsx! {
///     LineChart {
///         title: "Visitors",
///         x: ["Mon", "Tue", "Wed"],
///         series: vec![Series::new("Visits", [3, 5, 4])],
///         zoom: true,
///         handle: chart,
///     }
///     Button {
///         onclick: move |_| async move {
///             chart.save_png("visitors.png").await;
///         },
///         "Save as PNG"
///     }
///     Button {
///         disabled: !chart.is_zoomed(),
///         onclick: move |_| chart.reset_zoom(),
///         "Reset zoom"
///     }
/// }
/// # }
/// ```
pub fn use_chart() -> ChartHandle {
    use_hook(|| ChartHandle {
        source: CopyValue::new(None),
        owner: CopyValue::new(None),
        zoom: Signal::new(None),
        window: Signal::new(None),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_stands_alone() {
        let page = "<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\" focusable=\"false\" viewBox=\"0 0 200 100\"><text font-family=\"inherit\" fill=\"#111111\">Mon</text></svg>";
        let file = standalone(
            page,
            200.0,
            100.0,
            Some("Sales & <costs>"),
            HexColor::new(0xFFFFFF),
            "\"Inter\", var(--x)",
        );
        assert!(
            file.starts_with("<svg width=\"200\" height=\"100\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 200 100\"><title>Sales &amp; &lt;costs&gt;</title><rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>"),
            "{file}"
        );
        assert!(file.contains("font-family=\"sans-serif\""), "{file}");
        assert!(
            !file.contains("aria-hidden") && !file.contains("var("),
            "{file}"
        );
        let named = standalone(
            page,
            2.0,
            1.0,
            None,
            HexColor::new(0),
            "\"Inter\", sans-serif",
        );
        assert!(
            named.contains("font-family=\"'Inter', sans-serif\""),
            "{named}"
        );
        let markup = standalone(
            page,
            2.0,
            1.0,
            None,
            HexColor::new(0),
            "Fira <Sans> & \"Co\"",
        );
        assert!(
            markup.contains("font-family=\"Fira &lt;Sans&gt; &amp; 'Co'\""),
            "{markup}"
        );
    }

    #[test]
    fn the_handle_reads_the_drawing_as_a_file() {
        use crate::{
            LiberoProvider,
            components::data_display::{LineChart, Series},
        };
        thread_local! {
            static HANDLE: std::cell::Cell<Option<ChartHandle>> = const { std::cell::Cell::new(None) };
        }
        let mut dom = VirtualDom::new(|| {
            let chart = use_chart();
            HANDLE.set(Some(chart));
            rsx! {
                LiberoProvider {
                    LineChart {
                        title: "Visitors",
                        x: ["Mon", "Tue"],
                        series: vec![Series::new("Visits", [1.0, 2.0])],
                        handle: chart,
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("<button"), "{html}");
        let handle = HANDLE.get().unwrap();
        let svg = dom.in_runtime(|| handle.svg()).expect("drawn");
        assert!(svg.starts_with("<svg width=\"640\""), "{svg}");
        assert!(svg.contains("<title>Visitors</title>"), "{svg}");
        assert!(!svg.contains("inherit") && !svg.contains("var("), "{svg}");
    }

    #[test]
    fn the_handle_reads_and_resets_the_zoom() {
        use crate::{
            LiberoProvider,
            components::data_display::{LineChart, Series},
        };
        thread_local! {
            static HANDLE: std::cell::Cell<Option<ChartHandle>> = const { std::cell::Cell::new(None) };
        }
        let mut dom = VirtualDom::new(|| {
            let chart = use_chart();
            HANDLE.set(Some(chart));
            rsx! {
                LiberoProvider {
                    LineChart {
                        x: ["Mon", "Tue", "Wed", "Thu"],
                        series: vec![Series::new("Visits", [1.0, 2.0, 3.0, 4.0])],
                        zoom: true,
                        handle: chart,
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let handle = HANDLE.get().unwrap();
        dom.in_runtime(|| {
            let zoom = (*handle.zoom.peek()).expect("a zoomable chart");
            assert!(!handle.is_zoomed());
            zoom.set((1, 3));
            assert!(handle.is_zoomed());
            handle.reset_zoom();
            assert!(!handle.is_zoomed());
        });
    }
}
