//! The languages `highlight.rs` can color. An entry exists only when its
//! `code-lang-*` feature compiled the grammar in - there is no "recognized
//! but not compiled" state. Plain text is the one entry every build has: its
//! grammar is empty, so naming it is how a caller says "no highlighting"
//! without the unrecognized-language warning.

use std::sync::LazyLock;

use super::highlight::Grammar;

pub(crate) struct LanguageEntry {
    pub(crate) label: &'static str,
    pub(crate) aliases: &'static [&'static str],
    pub(crate) grammar: fn() -> Grammar,
}

// Each push is feature-gated, so this can't be one `vec![...]`.
#[allow(clippy::vec_init_then_push)]
pub(crate) static LANGUAGE_CATALOG: LazyLock<Vec<LanguageEntry>> = LazyLock::new(|| {
    // Every push below is feature-gated, so with no language compiled in
    // nothing else is ever written.
    // Plain text stays first: `Language::name` finds it there to localize it.
    #[allow(unused_mut)]
    let mut catalog = vec![LanguageEntry {
        label: "Plain text",
        aliases: &["text", "plain", "plaintext", "txt"],
        grammar: || &[],
    }];

    #[cfg(feature = "code-lang-javascript")]
    catalog.push(LanguageEntry {
        label: "JavaScript",
        aliases: &["javascript", "js"],
        grammar: super::languages::javascript::grammar,
    });
    #[cfg(feature = "code-lang-typescript")]
    catalog.push(LanguageEntry {
        label: "TypeScript",
        aliases: &["typescript", "ts"],
        grammar: super::languages::typescript::grammar,
    });
    #[cfg(feature = "code-lang-python")]
    catalog.push(LanguageEntry {
        label: "Python",
        aliases: &["python", "py"],
        grammar: super::languages::python::grammar,
    });
    #[cfg(feature = "code-lang-java")]
    catalog.push(LanguageEntry {
        label: "Java",
        aliases: &["java"],
        grammar: super::languages::java::grammar,
    });
    #[cfg(feature = "code-lang-c")]
    catalog.push(LanguageEntry {
        label: "C",
        aliases: &["c"],
        grammar: super::languages::c::grammar,
    });
    #[cfg(feature = "code-lang-cpp")]
    catalog.push(LanguageEntry {
        label: "C++",
        aliases: &["cpp", "c++"],
        grammar: super::languages::cpp::grammar,
    });
    #[cfg(feature = "code-lang-csharp")]
    catalog.push(LanguageEntry {
        label: "C#",
        aliases: &["csharp", "cs", "c#"],
        grammar: super::languages::csharp::grammar,
    });
    #[cfg(feature = "code-lang-php")]
    catalog.push(LanguageEntry {
        label: "PHP",
        aliases: &["php"],
        grammar: super::languages::php::grammar,
    });
    #[cfg(feature = "code-lang-ruby")]
    catalog.push(LanguageEntry {
        label: "Ruby",
        aliases: &["ruby", "rb"],
        grammar: super::languages::ruby::grammar,
    });
    #[cfg(feature = "code-lang-go")]
    catalog.push(LanguageEntry {
        label: "Go",
        aliases: &["go", "golang"],
        grammar: super::languages::go::grammar,
    });
    #[cfg(feature = "code-lang-rust")]
    catalog.push(LanguageEntry {
        label: "Rust",
        aliases: &["rust", "rs"],
        grammar: super::languages::rust::grammar,
    });
    #[cfg(feature = "code-lang-swift")]
    catalog.push(LanguageEntry {
        label: "Swift",
        aliases: &["swift"],
        grammar: super::languages::swift::grammar,
    });
    #[cfg(feature = "code-lang-kotlin")]
    catalog.push(LanguageEntry {
        label: "Kotlin",
        aliases: &["kotlin", "kt"],
        grammar: super::languages::kotlin::grammar,
    });
    #[cfg(feature = "code-lang-scala")]
    catalog.push(LanguageEntry {
        label: "Scala",
        aliases: &["scala"],
        grammar: super::languages::scala::grammar,
    });
    #[cfg(feature = "code-lang-sql")]
    catalog.push(LanguageEntry {
        label: "SQL",
        aliases: &["sql"],
        grammar: super::languages::sql::grammar,
    });
    #[cfg(feature = "code-lang-json")]
    catalog.push(LanguageEntry {
        label: "JSON",
        aliases: &["json"],
        grammar: super::languages::json::grammar,
    });
    #[cfg(feature = "code-lang-yaml")]
    catalog.push(LanguageEntry {
        label: "YAML",
        aliases: &["yaml", "yml"],
        grammar: super::languages::yaml::grammar,
    });
    #[cfg(feature = "code-lang-toml")]
    catalog.push(LanguageEntry {
        label: "TOML",
        aliases: &["toml"],
        grammar: super::languages::toml::grammar,
    });
    #[cfg(feature = "code-lang-markdown")]
    catalog.push(LanguageEntry {
        label: "Markdown",
        aliases: &["markdown", "md"],
        grammar: super::languages::markdown::grammar,
    });
    #[cfg(feature = "code-lang-bash")]
    catalog.push(LanguageEntry {
        label: "Bash",
        aliases: &["bash", "sh", "shell"],
        grammar: super::languages::bash::grammar,
    });
    #[cfg(feature = "code-lang-html")]
    catalog.push(LanguageEntry {
        label: "HTML",
        aliases: &["html", "htm", "xml"],
        grammar: super::languages::html::grammar,
    });
    #[cfg(feature = "code-lang-css")]
    catalog.push(LanguageEntry {
        label: "CSS",
        aliases: &["css"],
        grammar: super::languages::css::grammar,
    });
    #[cfg(feature = "code-lang-dart")]
    catalog.push(LanguageEntry {
        label: "Dart",
        aliases: &["dart"],
        grammar: super::languages::dart::grammar,
    });
    #[cfg(feature = "code-lang-r")]
    catalog.push(LanguageEntry {
        label: "R",
        aliases: &["r"],
        grammar: super::languages::r::grammar,
    });
    #[cfg(feature = "code-lang-perl")]
    catalog.push(LanguageEntry {
        label: "Perl",
        aliases: &["perl", "pl"],
        grammar: super::languages::perl::grammar,
    });
    #[cfg(feature = "code-lang-lua")]
    catalog.push(LanguageEntry {
        label: "Lua",
        aliases: &["lua"],
        grammar: super::languages::lua::grammar,
    });
    #[cfg(feature = "code-lang-haskell")]
    catalog.push(LanguageEntry {
        label: "Haskell",
        aliases: &["haskell", "hs"],
        grammar: super::languages::haskell::grammar,
    });
    #[cfg(feature = "code-lang-objective-c")]
    catalog.push(LanguageEntry {
        label: "Objective-C",
        aliases: &["objective-c", "objc", "objectivec"],
        grammar: super::languages::objective_c::grammar,
    });
    #[cfg(feature = "code-lang-powershell")]
    catalog.push(LanguageEntry {
        label: "PowerShell",
        aliases: &["powershell", "ps1", "pwsh"],
        grammar: super::languages::powershell::grammar,
    });
    #[cfg(feature = "code-lang-graphql")]
    catalog.push(LanguageEntry {
        label: "GraphQL",
        aliases: &["graphql", "gql"],
        grammar: super::languages::graphql::grammar,
    });

    catalog
});
