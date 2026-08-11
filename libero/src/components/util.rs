pub(crate) fn classes(base: Option<String>, extension: String) -> Option<String> {
    let mut classes = Vec::new();

    if let Some(class) = base {
        if !class.is_empty() {
            classes.push(class);
        }
    }

    classes.push(extension);

    Some(classes.join(" "))
}
