//! Checks that every relative link in the README, CONTRIBUTING, the CHANGELOG and the docs
//! folder points at a file or folder that exists. Links to other sites and to anchors on the
//! same page are not checked.

use std::fs;
use std::path::{Path, PathBuf};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn markdown_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            markdown_under(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

/// The targets of inline Markdown links: the text between `](` and the closing `)`, when it has
/// no whitespace.
fn link_targets(text: &str) -> Vec<&str> {
    let mut targets = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        let after = &rest[at + 2..];
        let Some(end) = after.find(')') else { break };
        let target = &after[..end];
        if !target.is_empty() && !target.chars().any(char::is_whitespace) {
            targets.push(target);
        }
        rest = &after[end..];
    }
    targets
}

#[test]
fn relative_links_in_the_documents_resolve() {
    let root = repository();
    let mut files = vec![
        root.join("README.md"),
        root.join("CONTRIBUTING.md"),
        root.join("CHANGELOG.md"),
    ];
    markdown_under(&root.join("docs"), &mut files);
    let mut broken = Vec::new();
    let mut checked = 0usize;
    for file in &files {
        let text = fs::read_to_string(file).unwrap();
        let folder = file.parent().unwrap();
        for target in link_targets(&text) {
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let path = target.split('#').next().unwrap();
            if path.is_empty() {
                continue;
            }
            checked += 1;
            if !folder.join(path).exists() {
                broken.push(format!(
                    "{}: broken link to {target}",
                    file.strip_prefix(&root).unwrap_or(file).display()
                ));
            }
        }
    }
    assert!(checked > 0, "no relative links were found");
    assert!(
        broken.is_empty(),
        "broken relative links:\n{}",
        broken.join("\n")
    );
}
