// src/snapshot/split.rs

use crate::types::FileEntry;
use std::fs;

/// Estimate tokens for a file (cheap, deterministic)
pub fn estimate_file_tokens(path: &str) -> usize {
    let Ok(text) = fs::read_to_string(path) else {
        return 0;
    };

    let words = text.split_whitespace().count();
    ((words as f32) * 1.33).round() as usize
}

/// Partition files into token-bounded packets
pub fn split_files_by_tokens(
    files: Vec<FileEntry>,
    target_tokens: usize,
) -> Vec<Vec<FileEntry>> {

    let mut packets: Vec<Vec<FileEntry>> = Vec::new();
    let mut current: Vec<FileEntry> = Vec::new();
    let mut current_tokens = 0usize;

    for file in files {
        let est = estimate_file_tokens(&file.path);

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
