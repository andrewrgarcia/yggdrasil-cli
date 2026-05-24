use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub lang: String,
}


pub fn infer_metadata(path: &str) -> FileMetadata {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let lang: String = match ext.as_str() {
        "rs" => "rust".into(),

        "py" => "python".into(),

        "js" => "javascript".into(),

        "ts" => "typescript".into(),

        "tsx" => "tsx".into(),

        "jsx" => "jsx".into(),

        "md" => "markdown".into(),

        "toml" => "toml".into(),

        "json" => "json".into(),

        "yaml" | "yml" => "yaml".into(),

        "html" => "html".into(),

        "css" => "css".into(),

        "tex" => "latex".into(),

        "" => "text".into(),

        _ => ext.clone(),
    };

    FileMetadata { lang }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_detection() {
        let m = infer_metadata("src/main.rs");

        assert_eq!(m.lang, "rust");
    }

}
