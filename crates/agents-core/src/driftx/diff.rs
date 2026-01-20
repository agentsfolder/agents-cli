use similar::TextDiff;

pub fn unified_diff_for(old: &str, new: &str, old_name: &str, new_name: &str) -> String {
    let old_n = normalize_newlines(old);
    let new_n = normalize_newlines(new);

    let diff = TextDiff::from_lines(&old_n, &new_n);

    diff.unified_diff()
        .context_radius(3)
        .header(old_name, new_name)
        .to_string()
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}
