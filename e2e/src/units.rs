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
    fn list_output_yields_test_names_only() {
        let stdout = "table::a: test\nbench::b: bench\n\n2 tests, 0 benchmarks\n";
        assert_eq!(listed(stdout), strings(&["table::a"]));
    }
}
