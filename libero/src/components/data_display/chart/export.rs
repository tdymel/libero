//! A chart's drawing as a file of its own: SVG as drawn, or PNG through the page's canvas.

use std::rc::Rc;

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
        false => font.replace('"', "'"),
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

/// A chart's drawing as an SVG or PNG file, and its zoom reset, from code. Made with
/// [`use_chart`], passed as a chart's `handle`.
#[derive(Clone, Copy, PartialEq)]
pub struct ChartHandle {
    source: CopyValue<Option<ExportSource>>,
    zoom: CopyValue<Option<Zoom>>,
}

impl ChartHandle {
    pub(super) fn attach(&self, source: ExportSource, zoom: Option<Zoom>) {
        let (mut slot, mut zoom_slot) = (self.source, self.zoom);
        if let Ok(mut slot) = slot.try_write() {
            *slot = Some(source);
        }
        if let Ok(mut slot) = zoom_slot.try_write() {
            *slot = zoom;
        }
    }

    /// On the chart's drop: its zoom's signal goes with it.
    pub(super) fn detach(&self) {
        let (mut source, mut zoom) = (self.source, self.zoom);
        if let Ok(mut slot) = source.try_write() {
            *slot = None;
        }
        if let Ok(mut slot) = zoom.try_write() {
            *slot = None;
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

    /// Back to the whole x axis, as the chart's reset button does.
    pub fn reset_zoom(&self) {
        if let Some(zoom) = self.zoom.try_peek().ok().and_then(|zoom| *zoom) {
            zoom.reset();
        }
    }
}

const NOTHING_DRAWN: &str = "the chart has no drawing";

/// A handle to a chart's drawing: pass it as the chart's `handle`, then save or read it.
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
///         handle: chart,
///     }
///     Button {
///         onclick: move |_| async move {
///             chart.save_png("visitors.png").await;
///         },
///         "Save as PNG"
///     }
/// }
/// # }
/// ```
pub fn use_chart() -> ChartHandle {
    use_hook(|| ChartHandle {
        source: CopyValue::new(None),
        zoom: CopyValue::new(None),
    })
}

/// A file name from the chart's title: `visitors.png`, else `chart.png`.
pub(super) fn file_name(title: Option<&str>, extension: &str) -> String {
    let stem: String = title
        .unwrap_or_default()
        .chars()
        .map(|ch| match ch.is_alphanumeric() {
            true => ch.to_lowercase().next().unwrap_or(ch),
            false => '-',
        })
        .collect();
    let stem = stem
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let stem = if stem.is_empty() { "chart" } else { &stem };
    format!("{stem}.{extension}")
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
                        export_button: true,
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains(">Save as SVG<") && html.contains(">Save as PNG<"),
            "{html}"
        );
        assert!(html.contains("data-slot=\"toolbar\""), "{html}");
        let handle = HANDLE.get().unwrap();
        let svg = dom.in_runtime(|| handle.svg()).expect("drawn");
        assert!(svg.starts_with("<svg width=\"640\""), "{svg}");
        assert!(svg.contains("<title>Visitors</title>"), "{svg}");
        assert!(!svg.contains("inherit") && !svg.contains("var("), "{svg}");
    }

    #[test]
    fn a_title_names_the_file() {
        assert_eq!(
            file_name(Some("Weekly visitors, 2026"), "svg"),
            "weekly-visitors-2026.svg"
        );
        assert_eq!(file_name(None, "png"), "chart.png");
        assert_eq!(file_name(Some("!!"), "png"), "chart.png");
    }
}
