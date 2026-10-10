/// A `Carousel`'s strings. `{n}` is a one-based slide number, `{m}` the count.
/// A caller also passes `aria_label`/`slide_label` per instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarouselLabels {
    pub label: &'static str,
    pub previous: &'static str,
    pub next: &'static str,
    /// Names a dot: `{n}`, the first slide it shows.
    pub indicator: &'static str,
    /// Names the dot strip's group.
    pub indicators: &'static str,
    /// A slide group's accessible name: `{n}` and `{m}`.
    pub slide: &'static str,
    /// What the live region reads when the slide settles: `{n}` and `{m}`.
    pub status: &'static str,
    /// The live region above one slide per view: the slides showing,
    /// `{from}` to `{to}`, of `{n}`.
    pub status_range: &'static str,
    /// [`status_range`](Self::status_range) for a looping window across the seam, which
    /// reads past the last slide on to the first: `{from}`, `{to}` and `{n}`.
    pub status_wrap: &'static str,
    /// The autoplay button's name in both states: `aria-pressed` carries the state.
    pub pause: &'static str,
    /// The root's `aria-roledescription`.
    pub roledescription: &'static str,
    /// Each slide's `aria-roledescription`.
    pub slide_roledescription: &'static str,
}

impl CarouselLabels {
    pub const ENGLISH: Self = Self {
        label: "Carousel",
        previous: "Previous slide",
        next: "Next slide",
        indicator: "Go to slide {n}",
        indicators: "Choose slide",
        slide: "{n} of {m}",
        status: "Slide {n} of {m}",
        status_range: "Slides {from}–{to} of {n}",
        status_wrap: "Slides {from} to {to} of {n}, wrapping around",
        pause: "Pause slideshow",
        roledescription: "carousel",
        slide_roledescription: "slide",
    };

    pub const GERMAN: Self = Self {
        label: "Karussell",
        previous: "Vorherige Folie",
        next: "Nächste Folie",
        indicator: "Zu Folie {n}",
        indicators: "Folie wählen",
        slide: "{n} von {m}",
        status: "Folie {n} von {m}",
        status_range: "Folien {from}–{to} von {n}",
        status_wrap: "Folien {from} bis {to} von {n}, über das Ende hinaus",
        pause: "Diashow anhalten",
        roledescription: "Karussell",
        slide_roledescription: "Folie",
    };
}
