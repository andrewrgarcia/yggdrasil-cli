// For later: any transformations, sorting, grouping.
// For now, simply passthrough.

use crate::types::FileEntry;

pub fn prepare_file_list(files: Vec<FileEntry>) -> Vec<FileEntry> {
    files
}
