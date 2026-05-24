/// Normalize a user-supplied path pattern into the canonical internal form:
/// forward slashes, no leading `./` or `.\`.
///
/// This makes VS Code's "Copy Relative Path" output (which uses `\` on Windows)
/// match paths produced by WalkDir, and removes the need for users to manually
/// prepend `.\` on Windows.
pub fn normalize_pattern(raw: &str) -> String {
    let s = raw.trim().replace('\\', "/");
    let s = s.strip_prefix("./").unwrap_or(&s);
    s.to_string()
}

/// Normalize a filesystem path (as emitted by WalkDir) into the same canonical form.
/// Separate function in case we want to diverge behavior later (e.g. preserve case).
pub fn normalize_path(raw: &str) -> String {
    let s = raw.replace('\\', "/");
    let s = s.strip_prefix("./").unwrap_or(&s);
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_dot_slash_unix() {
        assert_eq!(normalize_pattern("./src/main.rs"), "src/main.rs");
    }

    #[test]
    fn strips_dot_backslash_windows() {
        assert_eq!(normalize_pattern(".\\src\\main.rs"), "src/main.rs");
    }

    #[test]
    fn converts_backslashes() {
        assert_eq!(
            normalize_pattern("src\\scanner\\mod.rs"),
            "src/scanner/mod.rs"
        );
    }

    #[test]
    fn passes_through_clean_paths() {
        assert_eq!(normalize_pattern("src/main.rs"), "src/main.rs");
        assert_eq!(normalize_pattern("README.md"), "README.md");
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(normalize_pattern("  src/main.rs  "), "src/main.rs");
    }

    #[test]
    fn handles_directories() {
        assert_eq!(normalize_pattern("src\\scanner"), "src/scanner");
        assert_eq!(normalize_pattern(".\\src\\scanner"), "src/scanner");
    }
}
