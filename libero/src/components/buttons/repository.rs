use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            HtmlTag, Input, Part, States, Variant, base_props, parts_enum, parts_under_sx,
            use_button_group,
        },
        data_display::Pictogram,
        layout::use_box,
    },
    hooks::{SessionText, session_text, use_cache, use_formats, use_localization, use_theme},
    platform,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ACTION_ICON_SIZE, Color},
    tokens::NamedColorCss,
};

/// Where a repository lives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RepoHost {
    #[default]
    GitHub,
    GitLab,
}

impl RepoHost {
    /// The host's brand name, the start of the button's accessible name.
    pub fn name(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
        }
    }

    /// The repository's page, where the button leads.
    pub fn page_url(self, repo: &str) -> String {
        match self {
            Self::GitHub => format!("https://github.com/{repo}"),
            Self::GitLab => format!("https://gitlab.com/{repo}"),
        }
    }

    /// The public, unauthenticated endpoint that knows the star count.
    fn api_url(self, repo: &str) -> String {
        match self {
            Self::GitHub => format!("https://api.github.com/repos/{repo}"),
            // GitLab names a project by its path, URL-encoded.
            Self::GitLab => format!(
                "https://gitlab.com/api/v4/projects/{}",
                repo.replace('/', "%2F")
            ),
        }
    }

    /// The field of [`api_url`](Self::api_url)'s JSON that holds the count.
    fn stars_field(self) -> &'static str {
        match self {
            Self::GitHub => "stargazers_count",
            Self::GitLab => "star_count",
        }
    }
}

/// Session cache key: unauthenticated calls are rate limited (GitHub: 60 an hour).
fn stars_key(api: &str) -> String {
    format!("libero-repo-stars:{api}")
}

/// How long a fetched count counts as fresh; older, it shows while a new fetch runs.
const STARS_TTL_MS: u64 = 10 * 60 * 1000;

/// The stored `count@fetched_ms`, or a bare count (no fetch time: stale).
fn stored_stars(stored: &str) -> Option<(u64, Option<u64>)> {
    match stored.split_once('@') {
        Some((count, at)) => Some((count.parse().ok()?, at.parse().ok())),
        None => Some((stored.parse().ok()?, None)),
    }
}

fn parsed_count(stars: SessionText) -> Option<u64> {
    stored_stars(&stars.get()?).map(|(count, _)| count)
}

/// Whether the stored count was fetched within the TTL of `now`.
fn is_fresh(stored: &str, now: u64) -> bool {
    stored_stars(stored)
        .and_then(|(_, at)| at)
        .is_some_and(|at| now.saturating_sub(at) < STARS_TTL_MS)
}

fn parse_stars(body: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get(field)?
        .as_u64()
}

/// Half the button, from its size rather than its width, which a pill grows.
static GLYPH_SX: StaticSx = StaticSx::new(|| {
    let side = format!("calc({} * 0.5)", ACTION_ICON_SIZE.overridable());
    sx().display("inline-flex")
        .flex_shrink("0")
        .width(side.clone())
        .height(side)
});

/// With a count, the square grows into a pill. On a wrapper, leaving the caller's `sx` alone.
static STARS_SX: StaticSx = StaticSx::new(|| {
    sx().display("contents").when(
        "stars",
        sx().selector(
            "& > a",
            sx().width("auto")
                .padding_inline("sm")
                .gap("xs")
                .font_size("sm")
                .text_decoration("none"),
        ),
    )
});

/// Text needs 4.5:1, which an unfilled or muted tonal label can miss.
static COUNT_SX: StaticSx =
    StaticSx::new(|| sx().when("ink", sx().color(NamedColorCss::INK.value())));

parts_enum! {
    /// [`Repository`]'s inner parts, children of the link that takes `sx`.
    pub enum RepositoryPart {
        /// The host's logo.
        Icon = "icon" => "& > [data-slot='icon']",
        /// The star count, once known.
        Count = "count" => "& > [data-slot='count']",
    }
}

base_props! {
    parts(RepositoryPart);
    pub struct RepositoryProps {
        /// `owner/repo`, as in the repository's URL; GitLab takes nested groups.
        repo: String,
        /// Unset, GitHub.
        #[props(default)]
        host: RepoHost,
        #[props(default, into)]
        variant: Input<Variant>,
        /// Unset, the theme's `muted`; a gradient then takes the theme's gradient.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Replaces the name's subject (host and repo); the stars and the new-tab cue
        /// still follow. A raw `"aria-label"` attribute replaces the name whole.
        #[props(default, into)]
        aria_label: Option<String>,
    }
}

/// A link to a repository that shows its star count beside the host's icon.
/// Native builds need dioxus-native's `net` feature for the count.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Repository, RepoHost};
/// # fn app() -> Element {
/// rsx! {
///     Repository { repo: "tdymel/libero" }
///     Repository { repo: "gitlab-org/gitlab", host: RepoHost::GitLab }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/repository>
#[component]
pub fn Repository(props: RepositoryProps) -> Element {
    let theme = use_theme();
    let localization = use_localization();
    let separator = use_formats().decimal_separator;
    let compact = localization.repository.compact;
    let host = props.host;
    let api = host.api_url(&props.repo);

    let stars = use_cache(api.clone(), |api| {
        let stars = session_text(&stars_key(api));
        if stars
            .get()
            .is_some_and(|stored| is_fresh(&stored, platform::unix_millis()))
        {
            return stars;
        }
        let fetched = platform::fetch_text(api);
        spawn(async move {
            let Some(count) = fetched
                .await
                .and_then(|body| parse_stars(&body, host.stars_field()))
            else {
                return;
            };
            stars.set(format!("{count}@{}", platform::unix_millis()));
        });
        stars
    });
    let count = parsed_count(stars).filter(|&count| count > 0);

    let new_tab = localization.anchor.new_tab;
    // The repo in the name, else two buttons on a page read alike.
    let subject = props
        .aria_label
        .clone()
        .unwrap_or_else(|| format!("{} {}", host.name(), props.repo));
    let aria_label = match count {
        Some(count) => format!(
            "{subject}, {} {new_tab}",
            (localization.repository.stars)(count, &compact(count, separator))
        ),
        None => format!("{subject} {new_tab}"),
    };
    // A raw `aria-label` attribute is the whole-name override; `ActionIcon`'s own would win.
    let mut attributes = props.attributes.clone();
    let aria_label = match attributes.iter().position(|a| a.name == "aria-label") {
        Some(at) => match attributes.remove(at).value {
            AttributeValue::Text(raw) => raw,
            _ => aria_label,
        },
        None => aria_label,
    };
    let group = use_button_group();
    let variant = props
        .variant
        .copied_or(group.variant.unwrap_or(theme.repository.variant));
    // An unset gradient takes the theme's: no label reads on `muted` into its second stop
    // at 4.5:1 (todo 1650).
    let color = props
        .color
        .into_option()
        .or(group.color)
        .or_else(|| (variant != Variant::Gradient).then(|| theme.repository.color.into()));
    // An unfilled label may miss 4.5:1 (`warning` is 3.27:1); a gradient's label is contrast-picked.
    let ink = match variant {
        Variant::Filled | Variant::Gradient => false,
        Variant::Tonal => color == Some(ThemeAwareValue::from(Color::Muted)),
        _ => true,
    };
    let glyph = use_box()
        .framework_sx(&GLYPH_SX)
        .prepare()
        .attr("data-slot", RepositoryPart::Icon.slot())
        .render(
            HtmlTag::Span,
            Vec::new(),
            rsx! {
                Pictogram {
                    icon: match host {
                        RepoHost::GitHub => pictogram_icons_simple::github::regular,
                        RepoHost::GitLab => pictogram_icons_simple::gitlab::regular,
                    },
                }
            },
        );
    let count_states: Input<States> = States::new().with("ink", ink).into();
    let count_box = use_box()
        .framework_sx(&COUNT_SX)
        .states(&count_states)
        .prepare()
        .attr("data-slot", RepositoryPart::Count.slot());
    // Not heard: the link's `aria-label` replaces its content in the name.
    let count_text = count.map(|count| {
        let shown = compact(count, separator);
        count_box.render(HtmlTag::Span, Vec::new(), rsx! { "{shown}" })
    });
    let wrapper_states: Input<States> = States::new().with("stars", count.is_some()).into();
    let wrapper = use_box()
        .framework_sx(&STARS_SX)
        .states(&wrapper_states)
        .prepare();

    wrapper.render(
        HtmlTag::Span,
        Vec::new(),
        rsx! {
            ActionIcon {
                aria_label,
                to: host.page_url(&props.repo),
                target: "_blank",
                variant: Input::Value(variant),
                color: color.map_or(Input::None, Input::Value),
                size: props.size.clone(),
                radius: props.radius.clone(),
                class: props.class.clone(),
                sx: parts_under_sx(&props.parts, props.sx.clone()),
                states: props.states.clone(),
                attributes,
                {glyph}
                {count_text}
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<RepositoryPart>(),
            [
                ("icon", "& > [data-slot='icon']"),
                ("count", "& > [data-slot='count']"),
            ]
        );
    }

    #[test]
    fn each_host_names_its_endpoint_and_field() {
        assert_eq!(
            RepoHost::GitHub.api_url("a/b"),
            "https://api.github.com/repos/a/b"
        );
        assert_eq!(
            RepoHost::GitLab.api_url("group/sub/b"),
            "https://gitlab.com/api/v4/projects/group%2Fsub%2Fb"
        );
        assert_eq!(
            parse_stars(
                r#"{"stargazers_count": 12}"#,
                RepoHost::GitHub.stars_field()
            ),
            Some(12)
        );
        assert_eq!(
            parse_stars(r#"{"star_count": 3}"#, RepoHost::GitLab.stars_field()),
            Some(3)
        );
        assert_eq!(
            parse_stars(r#"{"message": "Not Found"}"#, "star_count"),
            None
        );
        assert_eq!(parse_stars("<html>", "star_count"), None);
    }

    /// A count fetched within the TTL is reused; an older or untimed one is fetched again.
    #[test]
    fn a_stored_count_expires_after_the_ttl() {
        assert_eq!(stored_stars("12@1000"), Some((12, Some(1000))));
        assert_eq!(stored_stars("12"), Some((12, None)));
        assert_eq!(stored_stars("x@1000"), None);
        assert!(is_fresh("1@1000", 1000 + STARS_TTL_MS - 1));
        assert!(!is_fresh("1@1000", 1000 + STARS_TTL_MS));
        assert!(!is_fresh("0@1000", 1000 + STARS_TTL_MS + 1));
        assert!(!is_fresh("1234", 1000));
    }
}
