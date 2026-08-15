// src/snapshot/split.rs

use crate::types::FileEntry;

/// Partition files into token-bounded packets.
///
/// Uses the token estimate already computed during collection
/// (`FileEntry::token_est`) instead of re-reading every file from disk.
pub fn split_files_by_tokens(files: Vec<FileEntry>, target_tokens: usize) -> Vec<Vec<FileEntry>> {
    let mut packets: Vec<Vec<FileEntry>> = Vec::new();
    let mut current: Vec<FileEntry> = Vec::new();
    let mut current_tokens = 0usize;

    for file in files {
        // viceroy: was estimate_file_tokens(&file.path), a second full read of
        // every file. collect_files already derives this from the same formula.
        let est = file.token_est;

        // If adding this file would exceed the packet budget
        if !current.is_empty() && current_tokens + est > target_tokens {
            packets.push(current);
            current = Vec::new();
            current_tokens = 0;
        }

        current_tokens += est;
        current.push(file);
    }

    if !current.is_empty() {
        packets.push(current);
    }

    packets
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, tokens: usize) -> FileEntry {
        FileEntry {
            path: path.into(),
            line_count: 0,
            word_count: 0,
            token_est: tokens,
        }
    }

    #[test]
    fn packs_until_budget_then_breaks() {
        let files = vec![entry("a", 400), entry("b", 400), entry("c", 400)];
        let packets = split_files_by_tokens(files, 1000);

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].len(), 2);
        assert_eq!(packets[1].len(), 1);
    }

    #[test]
    fn oversized_file_gets_its_own_packet() {
        let files = vec![entry("small", 10), entry("huge", 5000), entry("tail", 10)];
        let packets = split_files_by_tokens(files, 100);

        assert_eq!(packets.len(), 3);
    }
}