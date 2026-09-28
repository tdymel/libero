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
    hooks::{use_cache, use_formats, use_localization, use_theme},
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

fn cached_stars(api: &str) -> Option<u64> {
    platform::session_get(&stars_key(api))?.parse().ok()
}

fn parse_stars(body: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get(field)?
        .as_u64()
}

/// `999`, `1.2k`, `12k`, `1.2M`. Rounds down, so it never shows more than it is.
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
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Replaces the built name (host, repo, stars, new-tab cue) whole.
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
    let host = props.host;
    let api = host.api_url(&props.repo);

    // Bumped when a count lands, so the render reads the cache again.
    let mut landed = use_signal(|| 0u32);
    use_cache(api.clone(), |api| {
        if cached_stars(api).is_some() {
            return;
        }
        let fetched = platform::fetch_text(api);
        let api = api.clone();
        spawn(async move {
            let Some(count) = fetched
                .await
                .and_then(|body| parse_stars(&body, host.stars_field()))
            else {
                return;
            };
            platform::session_set(&stars_key(&api), &count.to_string());
            landed += 1;
        });
    });
    landed.read();
    let count = cached_stars(&api).filter(|&count| count > 0);

    let new_tab = localization.anchor.new_tab;
    // The repo in the name, else two buttons on a page read alike.
    let subject = format!("{} {}", host.name(), props.repo);
    let aria_label = match (props.aria_label.clone(), count) {
        (Some(label), _) => label,
        (None, Some(count)) => format!(
            "{subject}, {} {new_tab}",
            (localization.repository.stars)(count, &compact_count(count, separator))
        ),
        (None, None) => format!("{subject} {new_tab}"),
    };
    let group = use_button_group();
    let variant = props
        .variant
        .copied_or(group.variant.unwrap_or(theme.repository.variant));
    let color = props
        .color
        .into_option()
        .or(group.color)
        .unwrap_or_else(|| theme.repository.color.into());
    // An unfilled label may miss 4.5:1 (`warning` is 3.27:1).
    let ink = match variant {
        Variant::Filled => false,
        Variant::Tonal => color == ThemeAwareValue::from(Color::Muted),
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
        let shown = compact_count(count, separator);
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
                color,
                size: props.size.clone(),
                radius: props.radius.clone(),
                class: props.class.clone(),
                sx: parts_under_sx(&props.parts, props.sx.clone()),
                states: props.states.clone(),
                attributes: props.attributes.clone(),
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
}
