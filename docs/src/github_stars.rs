//! The header's GitHub link with the repo's star count. Self-contained: the
//! e2e fixture `docs_shell` includes this file.

use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, States},
    hooks::{use_formats, use_localization},
    localization::{AnchorLabels, Localization},
    sx::sx,
};

/// Unauthenticated calls are limited to 60 an hour, so a count lives for the session.
const STARS_STORAGE_KEY: &str = "libero-docs-stars";

/// An icon link to a GitHub repo. A known star count above 0 joins the icon,
/// and the box grows into a pill; before that, and on an error, the icon alone.
#[component]
pub fn GitHubLink(to: &'static str, children: Element) -> Element {
    let localization = use_localization();
    let separator = use_formats().decimal_separator;
    let stars = use_github_stars(to);
    let count = stars().filter(|&count| count > 0);

    rsx! {
        // The icon alone shows it is external, so the new-tab cue rides the name.
        ActionIcon {
            aria_label: github_label(localization, separator, count),
            to,
            target: "_blank",
            variant: "outlined",
            color: "muted",
            size: "lg",
            sx: sx().when(
                "stars",
                sx().width("auto")
                    .padding_inline("sm")
                    .gap("xs")
                    .font_size("sm")
                    .text_decoration("none"),
            ),
            states: States::new().with("stars", count.is_some()),
            {children}
            if let Some(count) = count {
                span { aria_hidden: "true", {compact_count(count, separator)} }
            }
        }
    }
}

/// The repo's star count. `None` before the answer, on an error, and off the web.
fn use_github_stars(repo_url: &'static str) -> ReadSignal<Option<u64>> {
    let mut stars = use_signal(|| None);
    use_hook(move || {
        if !cfg!(target_arch = "wasm32") {
            return;
        }
        let api = repo_url.replace("https://github.com/", "https://api.github.com/repos/");
        spawn(async move {
            let js = format!(
                "const cached = sessionStorage.getItem('{STARS_STORAGE_KEY}');
                if (cached !== null) return Number(cached);
                try {{
                    const response = await fetch('{api}');
                    if (!response.ok) return null;
                    const count = (await response.json()).stargazers_count;
                    if (!Number.isInteger(count)) return null;
                    sessionStorage.setItem('{STARS_STORAGE_KEY}', String(count));
                    return count;
                }} catch (error) {{
                    return null;
                }}"
            );
            if let Ok(Some(count)) = document::eval(&js).join::<Option<u64>>().await {
                stars.set(Some(count));
            }
        });
    });
    stars.into()
}

/// `999`, `1.2k`, `12k`, `1.2M`, with the locale's decimal separator. Rounds
/// down, so a count never shows more than it is.
fn compact_count(count: u64, separator: &str) -> String {
    let scaled = |unit: u64, suffix: &str| {
        let tenths = count * 10 / unit;
        if tenths >= 100 || tenths.is_multiple_of(10) {
            format!("{}{suffix}", tenths / 10)
        } else {
            format!("{}{separator}{}{suffix}", tenths / 10, tenths % 10)
        }
    };
    match count {
        0..1_000 => count.to_string(),
        1_000..1_000_000 => scaled(1_000, "k"),
        _ => scaled(1_000_000, "M"),
    }
}

/// "GitHub, 1.2k stars (opens in a new tab)". The docs keep no strings of their
/// own: German when the localization's words are, English otherwise.
fn github_label(localization: &Localization, separator: &str, stars: Option<u64>) -> String {
    let new_tab = localization.anchor.new_tab;
    let Some(count) = stars else {
        return format!("GitHub {new_tab}");
    };
    // One small field, not the whole struct: `Localization::GERMAN` is a `const`,
    // so its address says nothing.
    let noun = match (localization.anchor == AnchorLabels::GERMAN, count) {
        (true, 1) => "Stern",
        (true, _) => "Sterne",
        (false, 1) => "star",
        (false, _) => "stars",
    };
    format!(
        "GitHub, {} {noun} {new_tab}",
        compact_count(count, separator)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_count_steps_through_units() {
        let cases = [
            (0, "0"),
            (999, "999"),
            (1_000, "1k"),
            (1_234, "1.2k"),
            (9_999, "9.9k"),
            (12_345, "12k"),
            (999_999, "999k"),
            (1_250_000, "1.2M"),
        ];
        for (count, expected) in cases {
            assert_eq!(compact_count(count, "."), expected, "{count}");
        }
        assert_eq!(compact_count(1_234, ","), "1,2k");
    }

    #[test]
    fn label_names_the_count_in_both_languages() {
        assert_eq!(
            github_label(&Localization::ENGLISH, ".", Some(1_234)),
            "GitHub, 1.2k stars (opens in a new tab)"
        );
        assert_eq!(
            github_label(&Localization::GERMAN, ",", Some(1)),
            "GitHub, 1 Stern (öffnet in einem neuen Tab)"
        );
        assert_eq!(
            github_label(&Localization::ENGLISH, ".", None),
            "GitHub (opens in a new tab)"
        );
    }
}
