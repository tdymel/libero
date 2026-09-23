use dioxus::prelude::*;
use pictogram_icons_lobe as lobe;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input, Variant, base_props},
        data_display::{Icon, Pictogram},
        form::Chip,
        overlay::{Menu, MenuEntry, MenuItem, use_menu},
    },
    context::IconSlot,
    hooks::{use_localization, use_theme},
    localization::fill,
    sx::{ThemeAwareValue, sx},
    theme::Size,
};

/// An assistant a [`Tldr`] menu can hand a page to.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::SummaryProvider;
/// # fn app() -> Element {
/// let mut providers = SummaryProvider::defaults();
/// providers.retain(|provider| provider.name != "Google AI");
/// providers.push(SummaryProvider::new("Mistral", "https://chat.mistral.ai/chat?q="));
/// # rsx! {}
/// # }
/// ```
#[derive(Clone, PartialEq)]
pub struct SummaryProvider {
    /// The item's text and accessible name.
    pub name: String,
    /// A "new chat with this prompt" address; the encoded prompt is appended.
    pub url: String,
    /// Drawn before the name, in the text colour.
    pub mark: Option<Element>,
}

impl SummaryProvider {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            mark: None,
        }
    }

    pub fn mark(mut self, mark: Element) -> Self {
        self.mark = Some(mark);
        self
    }

    pub fn chatgpt() -> Self {
        Self::new("ChatGPT", "https://chat.openai.com/?q=")
            .mark(rsx! { Pictogram { icon: lobe::openai::mono } })
    }

    pub fn google_ai() -> Self {
        Self::new(
            "Google AI",
            "https://www.google.com/search?udm=50&aep=11&q=",
        )
        .mark(rsx! { Pictogram { icon: lobe::google::mono } })
    }

    pub fn claude() -> Self {
        Self::new("Claude", "https://claude.ai/new?q=")
            .mark(rsx! { Pictogram { icon: lobe::claude::mono } })
    }

    pub fn perplexity() -> Self {
        Self::new("Perplexity", "https://www.perplexity.ai/search/new?q=")
            .mark(rsx! { Pictogram { icon: lobe::perplexity::mono } })
    }

    /// ChatGPT, Google AI, Claude and Perplexity, in that order.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::chatgpt(),
            Self::google_ai(),
            Self::claude(),
            Self::perplexity(),
        ]
    }
}

/// `provider` asked to summarize `page`: `prompt`'s `{url}` filled, percent-encoded and appended.
///
/// ```
/// use libero::components::{SummaryProvider, summary_url};
///
/// let url = summary_url(&SummaryProvider::claude(), "Read {url}", "https://a.b/c");
/// assert_eq!(url, "https://claude.ai/new?q=Read%20https%3A%2F%2Fa.b%2Fc");
/// ```
pub fn summary_url(provider: &SummaryProvider, prompt: &str, page: &str) -> String {
    let asked = fill(prompt, &[("url", &page)]);
    format!("{}{}", provider.url, encode(&asked))
}

/// Percent-encodes everything but RFC 3986's unreserved characters.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

base_props! {
    pub struct TldrProps {
        /// The page's absolute address, e.g. its markdown mirror; assistants read it.
        #[props(into)]
        url: String,
        /// The menu's links, in order.
        #[props(default = SummaryProvider::defaults())]
        providers: Vec<SummaryProvider>,
        /// What the assistant is asked; `{url}` is filled with `url` (`{{url}}` in an `rsx!` literal).
        /// Unset, the localization's.
        #[props(default, into)]
        prompt: Option<String>,
        /// The trigger's text. Unset, the localization's `TLDR`.
        #[props(default, into)]
        label: Option<String>,
        /// A sparkles-only trigger, named by `aria_label`.
        #[props(default)]
        icon_only: bool,
        /// Names the icon-only trigger. Unset, the localization's.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Unset, `theme.tldr.variant`.
        #[props(default, into)]
        variant: Input<Variant>,
        /// The trigger's size step. Unset, `sm`.
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius, independent of `size`. Unset, the trigger's own: `xl` on the chip, `sm` icon-only.
        #[props(default, into)]
        radius: Input<Size>,
        /// Unset, `theme.tldr.color`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
    }
}

/// A menu of links that ask an assistant to summarize a page. Plain links: no script, no key.
/// `class`, `sx` and the attributes land on the trigger.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Tldr;
/// # fn app() -> Element {
/// rsx! {
///     Tldr { url: "https://libero-ui.dev/md/menu.md" }
///     Tldr { url: "https://libero-ui.dev/md/menu.md", icon_only: true }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/tldr>
#[component]
pub fn Tldr(props: TldrProps) -> Element {
    let theme = use_theme();
    let words = use_localization().tldr;
    let menu = use_menu();
    let prompt = props.prompt.as_deref().unwrap_or(words.prompt);
    let links = props
        .providers
        .iter()
        .map(|provider| {
            let item = MenuItem::new(provider.name.clone())
                .href(summary_url(provider, prompt, &props.url));
            match provider.mark.clone() {
                Some(mark) => item.leading(rsx! {
                    Icon { variant: "standard", size: "sm", color: "inherit", {mark} }
                }),
                None => item,
            }
            .into()
        })
        .collect();
    let items = vec![MenuEntry::Group {
        label: words.group.into(),
        items: links,
    }];

    let variant = props.variant.copied_or(theme.tldr.variant);
    let size = props.size.copied_or(Size::Md);
    let color = props
        .color
        .clone()
        .into_option()
        .unwrap_or_else(|| theme.tldr.color.into());
    let mut attributes = menu.a11y_attributes();
    attributes.extend(props.attributes.clone());

    rsx! {
        Menu { state: menu, items,
            if props.icon_only {
                ActionIcon {
                    size,
                    radius: props.radius.as_ref().copied(),
                    variant,
                    color,
                    aria_label: props.aria_label.clone().unwrap_or_else(|| words.icon_only.into()),
                    class: props.class.clone(),
                    sx: props.sx.clone(),
                    states: props.states.clone(),
                    attributes,
                    Glyph { slot: IconSlot::Sparkles, icon: lucide::sparkles::outlined }
                }
            } else {
                Chip {
                    size,
                    radius: props.radius.clone(),
                    variant,
                    color,
                    icon: rsx! {
                        Icon { variant: "standard", size: "sm", color: "inherit", Glyph { slot: IconSlot::Sparkles, icon: lucide::sparkles::outlined } }
                    },
                    // A block chip: in the menu's block wrapper an inline one sits on a line box.
                    sx: sx().display("flex").and(props.sx.clone().into_option().unwrap_or_default()),
                    class: props.class.clone(),
                    states: props.states.clone(),
                    // A no-op `onclick` makes it a `<button>`; the click bubbles to the menu.
                    onclick: |_| {},
                    attributes,
                    {props.label.clone().unwrap_or_else(|| words.label.into())}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROMPT: &str = "Summarize {url} and cite it.";

    #[test]
    fn the_prompt_names_the_page_and_is_encoded_into_q() {
        let providers = SummaryProvider::defaults();
        let names: Vec<_> = providers.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["ChatGPT", "Google AI", "Claude", "Perplexity"]);
        let claude = summary_url(&providers[2], PROMPT, "https://libero-ui.dev/overlay/menu");
        assert_eq!(
            claude,
            "https://claude.ai/new?q=Summarize%20https%3A%2F%2Flibero-ui.dev%2Foverlay%2Fmenu%20and%20cite%20it."
        );
        assert!(
            summary_url(&providers[1], PROMPT, "x")
                .starts_with("https://www.google.com/search?udm=50&aep=11&q=")
        );
    }

    #[test]
    fn the_default_prompt_names_the_page() {
        let english = crate::localization::TldrLabels::ENGLISH.prompt;
        let url = summary_url(
            &SummaryProvider::chatgpt(),
            english,
            "https://a.b/md/textarea.md",
        );
        assert!(url.starts_with("https://chat.openai.com/?q=Summarize%20and%20analyze"));
        assert!(url.contains("https%3A%2F%2Fa.b%2Fmd%2Ftextarea.md."));
        assert!(!url.contains(' '));
    }

    #[test]
    fn the_theme_defaults_keep_the_outlined_neutral_trigger() {
        let defaults = crate::theme::Theme::DEFAULT.tldr;
        assert_eq!(defaults.variant, Variant::Outlined);
        assert_eq!(defaults.color, crate::theme::Color::Neutral);
    }

    #[test]
    fn encoding_keeps_unreserved_and_escapes_the_rest() {
        assert_eq!(encode("a-b_c.d~E9"), "a-b_c.d~E9");
        assert_eq!(encode("a b&c=é"), "a%20b%26c%3D%C3%A9");
    }
}
