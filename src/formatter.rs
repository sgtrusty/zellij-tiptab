use std::path::Path;

fn trunc(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

pub fn fmt_bin(bin: &str) -> String {
    trunc(bin, 12).to_string()
}

pub fn fmt_folder(cwd: &Path) -> String {
    let name = cwd
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_else(|| cwd.to_str().unwrap_or(""));
    trunc(name, 12).to_string()
}

pub fn fmt_git(cwd: &Path, root: &Path) -> String {
    let root_basename = root
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    let root_basename = trunc(root_basename, 12);

    let relative = match cwd.strip_prefix(root) {
        Ok(r) if !r.as_os_str().is_empty() => r,
        _ => return root_basename.to_string(),
    };

    let last = relative
        .components()
        .last()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    let last = trunc(last, 12);

    format!("{root_basename}/../{last}")
}

pub fn fmt_label(position: u32, label: &str) -> String {
    let name = format!("{}: {}", position + 1, label);
    match name.char_indices().nth(60) {
        Some((i, _)) => name[..i].to_string(),
        None => name,
    }
}
