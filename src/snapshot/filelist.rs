use crate::types::FileEntry;

/// Prepare file list for output:
/// - deterministic
/// - lexicographic (alphanumeric) path order
/// - user controls semantics via naming
pub fn prepare_file_list(mut files: Vec<FileEntry>) -> Vec<FileEntry> {
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}
