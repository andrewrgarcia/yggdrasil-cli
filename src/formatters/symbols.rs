//! Declared-symbol extraction.
//!
//! Line-based rather than a raw regex sweep, because the sweep matched
//! declarations inside comments, docstrings, and `#[cfg(test)]` blocks —
//! which on this very repo meant `python.rs` reported seven test functions
//! and one real one.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

/// Historical cap. `extract_symbols` keeps it so codex headers stay stable.
pub const DEFAULT_MAX_SYMBOLS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    /// Brace-nested, `//` comments, `#[attr]` attributes.
    Rust,
    /// Indent-nested, `#` comments, triple-quoted docstrings.
    Python,
    /// Brace-nested, `//` comments, no attributes.
    Script,
}

fn family(lang: &str) -> Option<Family> {
    match lang {
        "rust" => Some(Family::Rust),
        "python" => Some(Family::Python),
        "javascript" | "typescript" | "tsx" | "jsx" => Some(Family::Script),
        _ => None,
    }
}

/// Whether this language has an extractor at all.
///
/// Callers need this to tell "nothing declared here" from "we don't parse
/// this" — a `.toml` with no symbols is not the same as an empty `.rs`.
pub fn supports_symbols(lang: &str) -> bool {
    family(lang).is_some()
}

/// Extract declared symbols, capped at [`DEFAULT_MAX_SYMBOLS`].
pub fn extract_symbols(lang: &str, content: &str) -> Vec<String> {
    extract_symbols_capped(lang, content, DEFAULT_MAX_SYMBOLS)
}

/// Extract declared symbols with a caller-chosen cap.
pub fn extract_symbols_capped(lang: &str, content: &str, max: usize) -> Vec<String> {
    let Some(fam) = family(lang) else {
        return Vec::new();
    };

    if max == 0 {
        return Vec::new();
    }

    let raw = match fam {
        Family::Python => scan_indented(content),
        other => scan_braced(content, other),
    };

    finish(raw, max)
}

// ── shared ────────────────────────────────────────────────────────────

/// Drop test scaffolding, dedupe, cap. Order is preserved.
///
// viceroy: extracted from the old inline `.take(10)` — the cap is now one
// step in a pipeline instead of an invisible tail on an iterator chain.
fn finish(names: Vec<String>, max: usize) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();

    for name in names {
        if is_test_name(&name) {
            continue;
        }
        if seen.insert(name.clone()) {
            out.push(name);
            if out.len() == max {
                break;
            }
        }
    }

    out
}

fn is_test_name(name: &str) -> bool {
    name == "tests" || name == "test" || name.starts_with("test_")
}

/// Strip leading modifier keywords so the declaration keyword lands first.
fn strip_modifiers<'a>(line: &'a str, words: &[&str]) -> &'a str {
    let mut s = line.trim_start();

    'outer: loop {
        for word in words {
            let Some(rest) = s.strip_prefix(word) else {
                continue;
            };

            // `pub(crate)`, `pub(super)`, `pub(in path)`
            if rest.starts_with('(') {
                if let Some(close) = rest.find(')') {
                    s = rest[close + 1..].trim_start();
                    continue 'outer;
                }
            }

            if rest.starts_with(char::is_whitespace) {
                s = rest.trim_start();

                // `extern "C" fn …`
                if *word == "extern" && s.starts_with('"') {
                    if let Some(close) = s[1..].find('"') {
                        s = s[close + 2..].trim_start();
                    }
                }

                continue 'outer;
            }
        }
        break;
    }

    s
}

/// Net brace movement on a line.
///
/// Braces inside string and char literals are miscounted; in practice they
/// are rare enough at declaration depth that the nesting filter still holds.
fn brace_delta(line: &str) -> i32 {
    let opens = line.matches('{').count() as i32;
    let closes = line.matches('}').count() as i32;
    opens - closes
}

// ── brace-nested languages ────────────────────────────────────────────

fn rust_decl() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:struct|enum|trait|union|type|fn)\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap()
    })
}

fn script_decl() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:class|function\*?|interface|enum|type)\s+([A-Za-z_$][A-Za-z0-9_$]*)")
            .unwrap()
    })
}

/// `const foo = () => …` / `const foo = function …` — a declaration in all
/// but keyword. A bare `const MAX = 5` is data, not structure, so it is out.
fn script_const_fn() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*(?::[^=]*)?=.*(?:=>|\bfunction\b)")
            .unwrap()
    })
}

const RUST_MODIFIERS: &[&str] = &[
    "pub", "async", "unsafe", "const", "extern", "default", "static",
];

const SCRIPT_MODIFIERS: &[&str] = &["export", "default", "async", "declare", "abstract"];

fn scan_braced(content: &str, fam: Family) -> Vec<String> {
    let mut out = Vec::new();

    let mut depth: i32 = 0;
    let mut in_block_comment = false;
    let mut pending_cfg_test = false;
    let mut skip_next_decl = false;
    // Brace depth at which the current `#[cfg(test)] mod` was entered.
    let mut test_region: Option<i32> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        let depth_before = depth;
        depth += brace_delta(line);

        // ── comments ──────────────────────────────────────────────
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }

        // ── inside a test module ──────────────────────────────────
        if let Some(entry_depth) = test_region {
            if depth <= entry_depth {
                test_region = None;
            }
            continue;
        }

        // ── attributes ────────────────────────────────────────────
        if fam == Family::Rust && trimmed.starts_with('#') {
            if trimmed.starts_with("#[cfg(test)]") {
                pending_cfg_test = true;
            } else if trimmed.contains("test]") {
                // #[test], #[tokio::test], #[rstest] …
                skip_next_decl = true;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        // A `#[cfg(test)]`-guarded module opens a region we skip wholesale.
        if pending_cfg_test {
            let opens_mod = strip_modifiers(trimmed, RUST_MODIFIERS).starts_with("mod ");
            if opens_mod {
                test_region = Some(depth_before);
                pending_cfg_test = false;
                continue;
            }
            pending_cfg_test = false;
        }

        // Top level and one level in (impl bodies, module bodies). Deeper
        // means a closure or an inner helper — noise in an index.
        if depth_before > 1 {
            continue;
        }

        let modifiers = if fam == Family::Rust {
            RUST_MODIFIERS
        } else {
            SCRIPT_MODIFIERS
        };
        let stripped = strip_modifiers(trimmed, modifiers);

        let captured = if fam == Family::Rust {
            rust_decl().captures(stripped)
        } else {
            script_decl()
                .captures(stripped)
                .or_else(|| script_const_fn().captures(stripped))
        };

        if let Some(caps) = captured {
            if skip_next_decl {
                skip_next_decl = false;
                continue;
            }
            if let Some(name) = caps.get(1) {
                out.push(name.as_str().to_string());
            }
        }
    }

    out
}

// ── indent-nested languages ───────────────────────────────────────────

fn python_decl() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?:class|def)\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap())
}

/// One level of Python indentation, in spaces.
const PY_INDENT: usize = 4;

fn scan_indented(content: &str) -> Vec<String> {
    let mut out = Vec::new();

    let mut in_docstring = false;
    let mut delim = "";

    for line in content.lines() {
        let expanded = line.replace('\t', "    ");
        let trimmed = expanded.trim();
        let indent = expanded.len() - expanded.trim_start().len();

        // ── docstrings ────────────────────────────────────────────
        // An odd number of triple-quotes on a line flips the state, which
        // also catches `x = """` where the quotes are not line-leading.
        let doubles = trimmed.matches("\"\"\"").count();
        let singles = trimmed.matches("'''").count();

        if in_docstring {
            let closes = (delim == "\"\"\"" && doubles % 2 == 1)
                || (delim == "'''" && singles % 2 == 1);
            if closes {
                in_docstring = false;
            }
            continue;
        }

        if doubles % 2 == 1 {
            in_docstring = true;
            delim = "\"\"\"";
            continue;
        }
        if singles % 2 == 1 {
            in_docstring = true;
            delim = "'''";
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Top level, plus class bodies. Deeper is a nested helper.
        if indent > PY_INDENT {
            continue;
        }

        let stripped = strip_modifiers(trimmed, &["async"]);

        if let Some(caps) = python_decl().captures(stripped) {
            if let Some(name) = caps.get(1) {
                out.push(name.as_str().to_string());
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_extract() {
        let s = r#"
            struct Foo {}
            enum Status {}
            fn parse() {}
        "#;

        let result = extract_symbols("rust", s);

        assert!(result.contains(&"Foo".to_string()));
        assert!(result.contains(&"parse".to_string()));
    }

    #[test]
    fn rust_skips_test_modules() {
        let s = "\
pub fn real_work() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one() {}

    fn helper_inside_tests() {}
}
";
        let got = extract_symbols("rust", s);
        assert_eq!(got, vec!["real_work"]);
    }

    #[test]
    fn rust_skips_standalone_test_fns() {
        let s = "#[test]\nfn checks_a_thing() {}\npub fn shipped() {}\n";
        let got = extract_symbols("rust", s);
        assert_eq!(got, vec!["shipped"]);
    }

    #[test]
    fn rust_skips_comments() {
        let s = "\
// fn commented_out() {}
/// Doc: see `fn documented_reference()`
/*
fn inside_block_comment() {}
*/
pub fn actual() {}
";
        let got = extract_symbols("rust", s);
        assert_eq!(got, vec!["actual"]);
    }

    #[test]
    fn rust_keeps_impl_methods_but_not_inner_fns() {
        let s = "\
impl Extractor {
    fn from_path() {
        fn buried_helper() {}
    }
    fn extract_imports() {}
}
";
        let got = extract_symbols("rust", s);
        assert_eq!(got, vec!["from_path", "extract_imports"]);
    }

    #[test]
    fn rust_strips_visibility_and_qualifiers() {
        let s = "\
pub(crate) struct Inner {}
pub async fn fetch() {}
pub unsafe extern \"C\" fn ffi_entry() {}
";
        let got = extract_symbols("rust", s);
        assert_eq!(got, vec!["Inner", "fetch", "ffi_entry"]);
    }

    #[test]
    fn python_skips_docstrings_and_nesting() {
        let s = "\
\"\"\"
def not_a_real_function():
\"\"\"

class Loader:
    def load(self):
        def inner():
            pass

def top_level():
    pass
";
        let got = extract_symbols("python", s);
        assert_eq!(got, vec!["Loader", "load", "top_level"]);
    }

    #[test]
    fn python_drops_test_functions() {
        let s = "def test_basic_imports():\n    pass\ndef extract():\n    pass\n";
        let got = extract_symbols("python", s);
        assert_eq!(got, vec!["extract"]);
    }

    #[test]
    fn script_keeps_functional_consts_only() {
        let s = "\
export const MAX_RETRIES = 5;
export const handler = async () => {};
export default class Widget {}
function plain() {}
";
        let got = extract_symbols("typescript", s);
        assert_eq!(got, vec!["handler", "Widget", "plain"]);
    }

    #[test]
    fn duplicates_collapse() {
        let s = "fn same() {}\nfn same() {}\nfn other() {}\n";
        let got = extract_symbols("rust", s);
        assert_eq!(got, vec!["same", "other"]);
    }

    #[test]
    fn cap_is_respected_and_zero_is_empty() {
        let s = (0..30)
            .map(|i| format!("fn f{}() {{}}\n", i))
            .collect::<String>();

        assert_eq!(extract_symbols("rust", &s).len(), DEFAULT_MAX_SYMBOLS);
        assert_eq!(extract_symbols_capped("rust", &s, 3).len(), 3);
        assert!(extract_symbols_capped("rust", &s, 0).is_empty());
    }

    #[test]
    fn support_is_reported_honestly() {
        assert!(supports_symbols("rust"));
        assert!(supports_symbols("tsx"));
        assert!(!supports_symbols("toml"));
        assert!(!supports_symbols("markdown"));
        assert!(extract_symbols("toml", "fn nope() {}").is_empty());
    }
}