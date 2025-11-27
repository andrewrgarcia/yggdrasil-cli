use std::fs;

/// Load patterns from a file, skipping empty lines and comments.
pub fn load_patterns_file(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .map(|contents| {
            contents
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_patterns_file_parses_correctly() {
        let tmpfile = "test_patterns.txt";
        std::fs::write(tmpfile, "foo\n#comment\nbar\n").unwrap();

        let patterns = load_patterns_file(tmpfile);
        assert_eq!(patterns, vec!["foo".to_string(), "bar".to_string()]);

        std::fs::remove_file(tmpfile).unwrap();
    }
}
