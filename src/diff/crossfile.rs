use std::collections::HashMap;

use crate::types::{BlockMatch, BlockWithVote, GroupedMatches};
use super::block_hash::hash_block;

/// Group block matches by (from_file, to_file) and detect duplicates.
pub fn group_and_filter_matches(
    matches: Vec<BlockMatch>,
    from_files: &[(String, String)],
) -> Vec<GroupedMatches> {
    let mut grouped: std::collections::BTreeMap<
        (String, String),
        Vec<BlockWithVote>
    > = std::collections::BTreeMap::new();

    // Map file → lines
    let mut file_lines = HashMap::new();
    for (fname, content) in from_files {
        file_lines.insert(fname.clone(), content.lines().map(|s| s.to_string()).collect::<Vec<_>>());
    }

    let mut seen_blocks = HashMap::<u64, bool>::new();

    for m in matches {
        let mut is_addition = false;

        if let Some(lines) = file_lines.get(&m.from_file) {
            let slice: Vec<&str> = lines[m.from_range.0..m.from_range.1]
                .iter()
                .map(|s| s.as_str())
                .collect();

            let bhash = hash_block(&slice);
            if seen_blocks.contains_key(&bhash) {
                is_addition = true;
            } else {
                seen_blocks.insert(bhash, true);
            }
        }

        grouped
            .entry((m.from_file.clone(), m.to_file.clone()))
            .or_default()
            .push(BlockWithVote { block: m, is_addition });
    }

    grouped
        .into_iter()
        .map(|((from, to), blocks)| GroupedMatches { from_file: from, to_file: to, blocks })
        .collect()
}

