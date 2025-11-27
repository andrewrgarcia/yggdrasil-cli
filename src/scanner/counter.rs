/// Count lines in a file, returning 0 if unreadable.
pub fn count_lines(path: &str) -> usize {
    std::fs::read_to_string(path)
        .map(|contents| contents.lines().count())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_lines_basic() {
        let tmpfile = "test_count_lines.txt";
        std::fs::write(tmpfile, "a\nb\nc\n").unwrap();

        assert_eq!(count_lines(tmpfile), 3);

        std::fs::remove_file(tmpfile).unwrap();
    }
}
