//! Exact unit filters (todo 1500). libtest's filter is a substring, so `table::` also ran
//! `sortable_table::` and `select::` ran `multi_select::`. A filter ending in `::` is a unit:
//! the runner lists the tests, keeps the names that start with it and passes them `--exact`.

/// libtest flags whose next argument is a value, not a filter.
const VALUED: &[&str] = &[
    "--skip",
    "--test-threads",
    "--format",
    "--color",
    "--logfile",
    "--shuffle-seed",
    "-Z",
];

/// The filters, the `--skip` substrings and every other argument, in order.
fn split(args: &[String]) -> (Vec<&str>, Vec<&str>, Vec<String>) {
    let (mut filters, mut skips, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if let Some(skip) = arg.strip_prefix("--skip=") {
            skips.push(skip);
        } else if arg == "--skip" {
            skips.extend(args.next().map(String::as_str));
        } else if VALUED.contains(&arg.as_str()) {
            rest.push(arg.clone());
            rest.extend(args.next().cloned());
        } else if arg.starts_with('-') {
            rest.push(arg.clone());
        } else {
            filters.push(arg.as_str());
        }
    }
    (filters, skips, rest)
}

/// Whether the arguments name a unit (`foo::`) that libtest alone would widen.
pub(crate) fn wanted(args: &[String]) -> bool {
    !args.iter().any(|arg| arg == "--exact" || arg == "--list")
        && split(args).0.iter().any(|filter| filter.ends_with("::"))
}

/// The test names in `--list` output (`name: test`).
pub(crate) fn listed(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .map(str::to_string)
        .collect()
}

/// The arguments that run exactly what `args` select: a `foo::` filter as a prefix, any
/// other as libtest's substring. Empty when nothing matches, since no filter runs all.
pub(crate) fn exact(listed: &[String], args: &[String]) -> Vec<String> {
    let (filters, skips, mut rest) = split(args);
    let names: Vec<String> = listed
        .iter()
        .filter(|name| {
            filters.iter().any(|filter| match filter.ends_with("::") {
                true => name.starts_with(filter),
                false => name.contains(filter),
            }) && !skips.iter().any(|skip| name.contains(skip))
        })
        .cloned()
        .collect();
    if names.is_empty() {
        return names;
    }
    rest.push("--exact".into());
    rest.extend(names);
    rest
}

/// Units whose tests open other modules' fixtures too, from the route literals in their sources
/// (`tests::extra_fixtures_list_the_routes_each_unit_opens` keeps it so).
const EXTRA_FIXTURES: &[(&str, &[&str])] = &[
    ("color_picker", &["color_field"]),
    (
        "combobox",
        &[
            "autocomplete",
            "cascader",
            "multi_select",
            "select",
            "tags_field",
        ],
    ),
    ("image_cropper", &["file_field"]),
    ("negative", &["button", "modal", "tabs"]),
    (
        "planted",
        &[
            "button",
            "carousel",
            "collapse",
            "drawer",
            "floating_window",
            "lightbox",
            "menu",
            "menubar",
            "multi_select",
            "radio_group",
            "select",
            "spotlight",
            "tabs",
            "tags_field",
            "tree",
        ],
    ),
    ("slider", &["range_slider"]),
    ("tree", &["chip"]),
    ("video", &["audio"]),
];

/// The fixture modules the filters need (todo 1618), for `E2E_FIXTURES`: each filter's unit
/// and its extras. None, so all build, when a filter is no `unit::...` or its unit has no
/// module of its own among `modules`.
pub(crate) fn fixtures(args: &[String], modules: &[String]) -> Option<Vec<String>> {
    if args.iter().any(|arg| arg == "--list") {
        return None;
    }
    let (filters, ..) = split(args);
    if filters.is_empty() {
        return None;
    }
    let mut wanted = Vec::new();
    for filter in filters {
        let (unit, _) = filter.split_once("::")?;
        if !modules.iter().any(|module| module == unit) {
            return None;
        }
        let extras = EXTRA_FIXTURES
            .iter()
            .find(|(name, _)| *name == unit)
            .map_or(&[][..], |(_, extras)| *extras);
        wanted.push(unit.to_string());
        wanted.extend(extras.iter().map(|extra| extra.to_string()));
    }
    wanted.sort();
    wanted.dedup();
    Some(wanted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    const LISTED: &[&str] = &[
        "table::a_wide_table_meets_the_baseline",
        "table::sticky::a_header_sticks",
        "sortable_table::it_sorts",
        "select::it_opens",
        "multi_select::it_opens",
    ];

    #[test]
    fn a_unit_keeps_only_names_that_start_with_it() {
        let args = exact(&strings(LISTED), &strings(&["table::", "--nocapture"]));
        assert_eq!(
            args,
            strings(&[
                "--nocapture",
                "--exact",
                "table::a_wide_table_meets_the_baseline",
                "table::sticky::a_header_sticks",
            ])
        );
    }

    #[test]
    fn other_filters_stay_substrings_and_skips_apply() {
        let args = strings(&[
            "select::",
            "sticky",
            "--skip",
            "multi",
            "--test-threads",
            "2",
        ]);
        assert!(wanted(&args));
        assert_eq!(
            exact(&strings(LISTED), &args),
            strings(&[
                "--test-threads",
                "2",
                "--exact",
                "table::sticky::a_header_sticks",
                "select::it_opens",
            ])
        );
    }

    #[test]
    fn no_match_and_explicit_modes_are_left_alone() {
        assert!(exact(&strings(LISTED), &strings(&["tabel::"])).is_empty());
        assert!(!wanted(&strings(&["table::", "--exact"])));
        assert!(!wanted(&strings(&["table::", "--list"])));
        assert!(!wanted(&strings(&["table", "--test-threads", "2"])));
    }

    #[test]
    fn a_unit_filter_builds_its_fixtures_and_extras_only() {
        let modules = strings(&["divider", "slider", "range_slider", "table"]);
        let fixtures = |args: &[&str]| fixtures(&strings(args), &modules);
        assert_eq!(
            fixtures(&["slider::", "divider::it_draws", "--nocapture"]),
            Some(strings(&["divider", "range_slider", "slider"]))
        );
        assert_eq!(fixtures(&["sticky"]), None);
        assert_eq!(fixtures(&["rtl_keys::"]), None);
        assert_eq!(fixtures(&["--nocapture"]), None);
        assert_eq!(fixtures(&["table::", "--list"]), None);
    }

    /// Modules `e2e/fixtures/src/lib.rs` builds always, and the files that are no module.
    const ALWAYS_BUILT: &[&str] = &["common", "docs_shell", "home", "lib", "main", "perf"];

    /// The route-shaped `"/..."` literals in `source`, a query cut off.
    fn route_literals(source: &str) -> Vec<String> {
        source
            .match_indices("\"/")
            .filter_map(|(at, _)| {
                let text = &source[at + 1..];
                let text = &text[..text.find('"')?];
                let route = text.split('?').next()?;
                route
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "/-_".contains(c))
                    .then(|| route.to_string())
            })
            .collect()
    }

    fn rust_files(path: &std::path::Path) -> Vec<String> {
        let read = |path: &std::path::Path| std::fs::read_to_string(path).unwrap();
        if path.is_dir() {
            std::fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
                .map(|path| read(&path))
                .collect()
        } else if path.with_extension("rs").is_file() {
            vec![read(&path.with_extension("rs"))]
        } else {
            Vec::new()
        }
    }

    /// Todo 1721: a unit's run builds every other fixture module whose route it names, and
    /// `EXTRA_FIXTURES` lists no module it does not.
    #[test]
    fn extra_fixtures_list_the_routes_each_unit_opens() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut owners = std::collections::HashMap::new();
        let mut modules = Vec::new();
        for entry in std::fs::read_dir(root.join("fixtures/src")).unwrap() {
            let path = entry.unwrap().path();
            let module = path.file_stem().unwrap().to_string_lossy().into_owned();
            if ALWAYS_BUILT.contains(&module.as_str()) {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            let Some(start) = source.find("pub const ROUTES") else {
                continue;
            };
            let block = &source[start..];
            let block = &block[..block.find("\n];").unwrap_or(block.len())];
            for route in route_literals(block) {
                owners.insert(route, module.clone());
            }
            modules.push(module);
        }
        modules.sort();

        let mut wrong = Vec::new();
        for module in &modules {
            let sources = rust_files(&root.join("tests/all").join(module));
            let mut opened: Vec<&str> = sources
                .iter()
                .flat_map(|source| route_literals(source))
                .filter_map(|route| owners.get(&route).map(String::as_str))
                .filter(|owner| *owner != module.as_str())
                .collect();
            opened.sort();
            opened.dedup();
            let listed = EXTRA_FIXTURES
                .iter()
                .find(|(unit, _)| *unit == module.as_str())
                .map_or(&[][..], |(_, extras)| *extras);
            if opened != listed {
                wrong.push(format!(
                    "(\"{module}\", &{opened:?}) but it lists {listed:?}"
                ));
            }
        }
        for (unit, _) in EXTRA_FIXTURES {
            if !modules.iter().any(|module| module.as_str() == *unit) {
                wrong.push(format!(
                    "{unit} has no fixture module, so its run builds all"
                ));
            }
        }
        assert!(
            wrong.is_empty(),
            "EXTRA_FIXTURES disagrees with the routes the units open:\n  {}",
            wrong.join("\n  ")
        );
    }

    #[test]
    fn route_literals_are_quoted_paths_only() {
        let source = r#"open("/a-b/c_1?x=1", "/b c", "/Up", r"\"/d"); "word"/2"#;
        assert_eq!(route_literals(source), strings(&["/a-b/c_1", "/d"]));
    }

    #[test]
    fn list_output_yields_test_names_only() {
        let stdout = "table::a: test\nbench::b: bench\n\n2 tests, 0 benchmarks\n";
        assert_eq!(listed(stdout), strings(&["table::a"]));
    }
}
