use std::path::Path;

/// Expand a set of paths (files or directories) into a flat list of file paths.
pub fn expand_paths(paths: &[String]) -> Vec<String> {
    let mut files = Vec::new();

    for p in paths {
        let path = Path::new(p);

        if path.is_file() {
            files.push(p.clone());
        } else if path.is_dir() {
            let walker = walkdir::WalkDir::new(path).into_iter();
            for entry in walker.filter_map(Result::ok) {
                if entry.file_type().is_file() {
                    if let Some(s) = entry.path().to_str() {
                        files.push(s.to_string());
                    }
                }
            }
        } else {
            eprintln!("⚠️ Path not found: {}", p);
        }
    }

    files
}

