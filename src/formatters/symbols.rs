use regex::Regex;

pub fn extract_symbols(lang: &str, content: &str) -> Vec<String> {
    let pattern = match lang {
        "rust" =>
            r"(?:struct|enum|trait|fn)\s+([A-Za-z_][A-Za-z0-9_]*)",

        "python" =>
            r"(?:class|def|async\s+def)\s+([A-Za-z_][A-Za-z0-9_]*)",

        "javascript" | "typescript" =>
            r"(?:class|function|const)\s+([A-Za-z_][A-Za-z0-9_]*)",

        _ => return vec![],
    };

    let re = Regex::new(pattern).unwrap();

    re.captures_iter(content)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .take(10)
        .collect()
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

        let result =
            extract_symbols("rust", s);

        assert!(
            result.contains(
                &"Foo".to_string()
            )
        );

        assert!(
            result.contains(
                &"parse".to_string()
            )
        );
    }
}