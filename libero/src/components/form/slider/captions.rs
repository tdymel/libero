//! Mark captions, thinned where neighbours would touch, as an axis drops tick labels (todo 5508).

use dioxus::prelude::*;

use super::core::SliderPart;
use crate::{components::common::Part, hooks::use_element, platform::ElementApi};

/// The least room between two shown captions, in px.
const CAPTION_GAP: f64 = 4.0;

/// Where a caption starts and ends along the track, in client px.
type Span = (f64, f64);

#[derive(Props, Clone, PartialEq)]
pub(super) struct MarkCaptionsProps {
    /// Each caption's position style and text, in mark order.
    captions: Vec<(String, String)>,
    class: Option<String>,
    vertical: bool,
    aria_hidden: bool,
    /// Bumped when the track resizes.
    layout: ReadSignal<u64>,
}

#[component]
pub(super) fn MarkCaptions(props: MarkCaptionsProps) -> Element {
    // Owned here and written by each caption, so no handle outlives its scope.
    let spans = use_signal(Vec::<Option<Span>>::new);
    let resized = use_signal(|| 0_u64);
    let count = props.captions.len();
    // In render, so every caption's new span re-thins; all shown until each is measured.
    let measured: Option<Vec<Span>> = spans.read().iter().take(count).copied().collect();
    let hide = measured
        .filter(|measured| measured.len() == count)
        .map(|measured| thinned(&measured, CAPTION_GAP))
        .unwrap_or_default();
    rsx! {
        for (index, (at, label)) in props.captions.iter().enumerate() {
            MarkCaption {
                key: "{index}",
                index,
                at: at.clone(),
                label: label.clone(),
                class: props.class.clone(),
                vertical: props.vertical,
                aria_hidden: props.aria_hidden,
                hidden: hide.get(index).copied().unwrap_or(false),
                spans,
                layout: props.layout,
                resized,
            }
        }
    }
}

/// One caption. A mount, its own resize (its text, the text size) or the track's
/// bumps `resized` or `layout`, and every caption re-measures itself.
#[component]
fn MarkCaption(
    index: usize,
    at: String,
    label: String,
    class: Option<String>,
    vertical: bool,
    aria_hidden: bool,
    hidden: bool,
    spans: Signal<Vec<Option<Span>>>,
    layout: ReadSignal<u64>,
    resized: Signal<u64>,
) -> Element {
    let element = use_element();
    use_effect(use_reactive!(|index, vertical| {
        let _ = (element.mount_token(), layout(), resized());
        spawn(async move {
            let (Ok((x, y)), Ok(size)) =
                (element.client_offset().await, element.dimensions().await)
            else {
                return;
            };
            let span = match vertical {
                true => (y, y + size.height),
                false => (x, x + size.width),
            };
            spans.with_mut(|all| {
                if all.len() <= index {
                    all.resize(index + 1, None);
                }
                all[index] = Some(span);
            });
        });
    }));
    // Hidden, not removed: it keeps its size for the next measure. Always declared, as a
    // raw `style` that drops one keeps its last value ([[codebase/css-vars]]).
    let visibility = if hidden { "hidden" } else { "inherit" };
    let style = format!("{at};visibility:{visibility}");
    rsx! {
        span {
            class,
            "data-slot": SliderPart::MarkLabel.slot(),
            // An upright track's captions sit where a thumb's bubble lands.
            "data-tooltip-clear": vertical.then_some("true"),
            style,
            "aria-hidden": (aria_hidden || hidden).then_some("true"),
            onmounted: element.mount(),
            onresize: move |_| resized += 1,
            {label}
        }
    }
}

/// Which captions to hide so no two shown ones come closer than `gap`; the outermost two stay.
fn thinned(spans: &[Span], gap: f64) -> Vec<bool> {
    let mut hidden = vec![false; spans.len()];
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by(|a, b| spans[*a].0.total_cmp(&spans[*b].0));
    let [first, middle @ .., last] = order.as_slice() else {
        return hidden;
    };
    let (mut end, limit) = (spans[*first].1, spans[*last].0);
    for &index in middle {
        let (start, stop) = spans[index];
        if start < end + gap || stop + gap > limit {
            hidden[index] = true;
        } else {
            end = stop;
        }
    }
    hidden
}

#[cfg(test)]
mod tests {
    use super::thinned;

    #[test]
    fn apart_captions_all_stay() {
        let spans = [(0.0, 30.0), (100.0, 160.0), (200.0, 240.0), (280.0, 310.0)];
        assert_eq!(thinned(&spans, 4.0), [false; 4]);
    }

    /// The audit's 200 % text: "Medium" 84-185 runs into "High" 164-222 (todo 5508).
    #[test]
    fn a_caption_touching_its_neighbour_hides_and_the_ends_stay() {
        let spans = [(0.0, 50.0), (84.0, 185.0), (164.0, 222.0), (240.0, 286.0)];
        assert_eq!(thinned(&spans, 4.0), [false, false, true, false]);
    }

    #[test]
    fn a_caption_too_close_to_the_last_hides() {
        let spans = [(0.0, 50.0), (100.0, 238.0), (240.0, 286.0)];
        assert_eq!(thinned(&spans, 4.0), [false, true, false]);
    }

    /// RTL lays the marks out right to left: the order is by position, not by mark.
    #[test]
    fn order_follows_position() {
        let spans = [(240.0, 286.0), (164.0, 222.0), (84.0, 185.0), (0.0, 50.0)];
        assert_eq!(thinned(&spans, 4.0), [false, true, false, false]);
    }

    #[test]
    fn one_or_two_captions_never_hide() {
        assert_eq!(thinned(&[(0.0, 50.0)], 4.0), [false]);
        assert_eq!(thinned(&[(0.0, 50.0), (10.0, 60.0)], 4.0), [false, false]);
    }
}
