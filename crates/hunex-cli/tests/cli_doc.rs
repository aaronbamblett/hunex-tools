//! Checks that docs/cli.md mentions every format, command and option in the help text.

use std::collections::BTreeSet;

const DOC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/cli.md");

fn helps() -> Vec<(&'static str, &'static str)> {
    use hunex_cli::{
        cbg, chs, cli, ctd, font, hfa, hw, image, lenzu, mrg, mzp, mzx, nopus, nx, switch_script,
        switch_text,
    };
    vec![
        ("", cli::HELP),
        ("cbg", cbg::HELP),
        ("chs", chs::HELP),
        ("ctd", ctd::HELP),
        ("font", font::HELP),
        ("hfa", hfa::HELP),
        ("hw", hw::HELP),
        ("image", image::HELP),
        ("lenzu", lenzu::HELP),
        ("mrg", mrg::HELP),
        ("mzp", mzp::HELP),
        ("mzx", mzx::HELP),
        ("nopus", nopus::HELP),
        ("nx", nx::HELP),
        ("switch-script", switch_script::HELP),
        ("switch-text", switch_text::HELP),
    ]
}

fn trim(token: &str) -> &str {
    token.trim_matches(|c: char| "[](),.;:'\"".contains(c))
}

fn is_option(token: &str) -> bool {
    let rest = if let Some(rest) = token.strip_prefix("--") {
        rest
    } else if let Some(rest) = token.strip_prefix('-') {
        rest
    } else {
        return false;
    };
    rest.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

#[test]
fn cli_doc_lists_every_format_command_and_option() {
    let doc = std::fs::read_to_string(DOC).expect("read docs/cli.md");
    let mut required: BTreeSet<String> = BTreeSet::new();
    for (format, text) in helps() {
        for line in text.lines() {
            let Some(rest) = line.trim_start().strip_prefix("hunex ") else {
                continue;
            };
            let words: Vec<&str> = rest.split_whitespace().collect();
            let Some(&first) = words.first() else {
                continue;
            };
            if format.is_empty() {
                if !first.starts_with('<') {
                    required.insert(format!("hunex {}", trim(first)));
                }
            } else if first == format {
                if let Some(&command) = words.get(1) {
                    if !command.starts_with('<') {
                        required.insert(format!("hunex {format} {}", trim(command)));
                    }
                }
            }
        }
        for token in text.split_whitespace() {
            let token = trim(token);
            if is_option(token) {
                required.insert(format!("`{token}"));
            }
        }
        if !format.is_empty() {
            required.insert(format!("hunex {format}"));
        }
    }
    required.insert("`--version".to_string());
    required.insert("`--help".to_string());

    let missing: Vec<&String> = required
        .iter()
        .filter(|item| !doc.contains(item.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/cli.md does not mention these items from the help text: {missing:?}"
    );
}
