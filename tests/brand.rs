//! The palette lives in art/DESIGN.md; every color literal on a site
//! surface must be declared there. Nothing is "remembered" to be kept
//! in sync; drift is a red test (same bargain as book.rs).

use std::{collections::BTreeSet, fs, path::PathBuf};

/// Every #rgb / #rrggbb literal in `s`, normalized to lowercase rrggbb.
/// Css id selectors like `#editor` never lex as 3 or 6 hex digits
/// followed by a non-hex boundary, so a plain scan is enough.
fn hexes(s: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b != b'#' {
            continue;
        }
        let run: String = s[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        match run.len() {
            6 => {
                out.insert(run.to_lowercase());
            }
            3 => {
                out.insert(run.chars().flat_map(|c| [c, c]).collect::<String>().to_lowercase());
            }
            _ => {}
        }
    }
    out
}

#[test]
fn site_colors_come_from_design_md() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let allowed = hexes(&fs::read_to_string(root.join("art/DESIGN.md")).unwrap());
    assert!(!allowed.is_empty(), "no palette found in art/DESIGN.md");

    let surfaces = [
        "site/index.html",
        "playground/site/index.html",
        "playground/site/style.css",
        "docs/book/theme/nevla.css",
    ];
    let mut drift = vec![];
    for path in surfaces {
        let used = hexes(&fs::read_to_string(root.join(path)).unwrap());
        for hex in used.difference(&allowed) {
            drift.push(format!("{path}: #{hex} is not in art/DESIGN.md"));
        }
    }
    assert!(drift.is_empty(), "palette drift:\n{}", drift.join("\n"));
}
