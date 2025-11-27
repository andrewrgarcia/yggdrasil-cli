use glob::Pattern;
use std::path::Path;

pub fn matches_filters(path: &str, filters: &[String], default: bool) -> bool {
    if filters.is_empty() {
        return default;
    }

    let norm_path = path.strip_prefix("./").unwrap_or(path);
    let base = Path::new(norm_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    filters.iter().any(|f| {
        let norm_filter = f.strip_prefix("./").unwrap_or(f);
        norm_path == norm_filter
            || base == norm_filter
            || norm_path.starts_with(norm_filter)
            || Pattern::new(norm_filter)
                .map(|p| p.matches(norm_path) || p.matches(base))
                .unwrap_or(false)
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_filters_exact_and_glob() {
        let filters = vec!["src".to_string(), "*.rs".to_string()];

        assert!(matches_filters("src/main.rs", &filters, false));
        assert!(matches_filters("foo.rs", &filters, false));
        assert!(!matches_filters("docs/readme.md", &filters, false));
    }

    #[test]
    fn test_matches_filters_empty_returns_default() {
        let filters: Vec<String> = vec![];
        assert!(matches_filters("anything", &filters, true));
        assert!(!matches_filters("anything", &filters, false));
    }
}

