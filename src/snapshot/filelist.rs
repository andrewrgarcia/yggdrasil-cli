use crate::types::FileEntry;
use natord::compare;

pub fn prepare_file_list(mut files: Vec<FileEntry>) -> Vec<FileEntry> {
    // Sort by:
    // 1. directory path
    // 2. natural order filename (section1 < section10 < section11)
    files.sort_by(|a, b| {
        // Split paths into (dir, file)
        let (adir, afile) = split_path(&a.path);
        let (bdir, bfile) = split_path(&b.path);

        // First compare directory path lexicographically
        match adir.cmp(&bdir) {
            std::cmp::Ordering::Equal => {
                // Then compare filenames using natural ordering (1 < 2 < 10)
                compare(&afile, &bfile)
            }
            other => other,
        }
    });

    files
}

fn split_path(path: &str) -> (String, String) {
    match path.rsplit_once('/') {
        Some((dir, file)) => (dir.to_string(), file.to_string()),
        None => ("".to_string(), path.to_string()),
    }
}
