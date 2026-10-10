//! G-STR · the string table (m0_contrat.md §4.1, §5.4). sr-app is bin-only (R11), so this test includes the table's
//! source by path: `crates/sr-app/src/strings.rs` is graded as written, not through a binary. Three checks —
//! the contract's `strings` block and `STRINGS` hold the same keys (each way), the texts are byte-identical, and
//! web/index.html carries each `web.*` text verbatim. Bytes, never normalised: × U+00D7, — U+2014, ≈ U+2248, · U+00B7
//! and … U+2026 (§4.1) are different code points from the look-alikes a typo or an editor would put in their place.

#[path = "../src/strings.rs"]
#[allow(dead_code)]
mod strings;

use std::collections::BTreeMap;
use std::path::PathBuf;

use strings::STRINGS;

fn repo_file(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("{} is not UTF-8: {e}", path.display()))
}

/// §4.1's block: the lines between the ```strings fence and the next fence, each `key = "text"`.
fn contract_block() -> Vec<(String, String)> {
    let contract = repo_file("milestones/m0/m0_contrat.md");
    let mut rows = Vec::new();
    let mut inside = false;
    for line in contract.lines() {
        if !inside {
            inside = line.trim_end() == "```strings";
            continue;
        }
        if line.starts_with("```") {
            return rows;
        }
        let (key, quoted) = line.split_once(" = ").unwrap_or_else(|| panic!("a row of §4.1 without ` = `: {line:?}"));
        let text = quoted
            .strip_prefix('"')
            .and_then(|q| q.strip_suffix('"'))
            .unwrap_or_else(|| panic!("a row of §4.1 whose text is not quoted: {line:?}"));
        rows.push((key.to_string(), text.to_string()));
    }
    panic!("m0_contrat.md has no closed ```strings block (§4.1)");
}

fn table() -> BTreeMap<&'static str, &'static str> {
    let mut map = BTreeMap::new();
    for (key, text) in STRINGS {
        assert!(map.insert(*key, *text).is_none(), "STRINGS holds the key `{key}` twice");
    }
    map
}

fn block() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (key, text) in contract_block() {
        assert!(map.insert(key.clone(), text).is_none(), "§4.1 holds the key `{key}` twice");
    }
    map
}

#[test]
fn same_keys_both_ways() {
    let (table, block) = (table(), block());
    // A block read as empty must not pass: the table is 100+ rows (§4.1).
    assert!(block.len() >= 100, "§4.1's block read as {} rows", block.len());
    let missing: Vec<&String> = block.keys().filter(|k| !table.contains_key(k.as_str())).collect();
    let extra: Vec<&&str> = table.keys().filter(|k| !block.contains_key(**k)).collect();
    assert!(missing.is_empty(), "keys in §4.1 and not in STRINGS: {missing:?}");
    assert!(extra.is_empty(), "keys in STRINGS and not in §4.1: {extra:?}");
}

#[test]
fn texts_byte_identical() {
    let (table, block) = (table(), block());
    let mut wrong = Vec::new();
    for (key, want) in &block {
        if let Some(got) = table.get(key.as_str()) {
            if got.as_bytes() != want.as_bytes() {
                wrong.push(format!("{key}: STRINGS {:?} ({:02x?}), §4.1 {:?} ({:02x?})", got, got.as_bytes(), want, want.as_bytes()));
            }
        }
    }
    assert!(wrong.is_empty(), "texts that differ from §4.1:\n{}", wrong.join("\n"));
    // `get` reads the same table (the desktop's title comes through it).
    assert_eq!(strings::get("app.title").as_bytes(), block["app.title"].as_bytes());
}

#[test]
fn web_texts_verbatim_in_index_html() {
    let page = repo_file("web/index.html");
    let block = block();
    let web: Vec<(&String, &String)> = block.iter().filter(|(k, _)| k.starts_with("web.")).collect();
    assert!(web.len() >= 4, "§4.1 holds {} web.* rows, expected at least 4", web.len());
    let lacking: Vec<&str> = web.iter().filter(|(_, text)| !page.contains(text.as_str())).map(|(k, _)| k.as_str()).collect();
    assert!(lacking.is_empty(), "web/index.html lacks these web.* texts verbatim: {lacking:?}");
}
