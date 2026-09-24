/// A `Tldr` menu's words. Provider names are brand names and stay literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TldrLabels {
    /// The trigger's visible text.
    pub label: &'static str,
    /// Names the icon-only trigger.
    pub icon_only: &'static str,
    /// Names the group of providers in the menu.
    pub group: &'static str,
    /// What the assistant is asked; `{url}` is the page's address.
    pub prompt: &'static str,
}

impl TldrLabels {
    pub const ENGLISH: Self = Self {
        label: "TLDR",
        icon_only: "Summarize with AI",
        group: "Summarize with",
        prompt: "Summarize and analyze the key insights from {url}. If you cannot access this URL \
                 please fall back to your general knowledge.",
    };

    pub const GERMAN: Self = Self {
        label: "TLDR",
        icon_only: "Mit KI zusammenfassen",
        group: "Zusammenfassen mit",
        prompt: "Fasse die wichtigsten Erkenntnisse aus {url} zusammen und analysiere sie. Wenn \
                 du diese URL nicht öffnen kannst, greife auf dein allgemeines Wissen zurück.",
    };
}
