use std::collections::{HashMap, HashSet};

use crate::types::BlockMatch;
use super::block_hash::hash_line;
use super::structural::extend_structural_block;

/// Index: hash -> all occurrences (file, line_no)
type LineIndex = HashMap<u64, Vec<(String, usize)>>;

/// Build inverted index for all lines in `to` set.
fn build_index(files: &[(String, String)]) -> LineIndex {
    let mut index = LineIndex::new();
    for (file, content) in files {
        for (i, line) in content.lines().enumerate() {
            let h = hash_line(line);
            index.entry(h).or_default().push((file.clone(), i));
        }
    }
    index
}

/// Find block matches for a specific min block size.
fn find_block_matches_single(
    from_files: &[(String, String)],
    to_files: &[(String, String)],
    min_block_size: usize,
) -> Vec<BlockMatch> {
    let index = build_index(to_files);
    let mut matches = Vec::new();
    let mut visited_from = HashSet::new();
    let mut visited_to = HashSet::new();

    for (from_file, content) in from_files {
        let from_lines: Vec<&str> = content.lines().collect();

        for (i, line) in from_lines.iter().enumerate() {
            if visited_from.contains(&(from_file.clone(), i)) {
                continue;
            }

            let h = hash_line(line);
            if let Some(candidates) = index.get(&h) {
                for (to_file, j) in candidates {
                    if visited_to.contains(&(to_file.clone(), *j)) {
                        continue;
                    }

                    let (_, to_content) =
                        to_files.iter().find(|(f, _)| f == to_file).unwrap();
                    let to_lines: Vec<&str> = to_content.lines().collect();

                    let (f1, f2, t1, t2) =
                        extend_structural_block(&from_lines, &to_lines, i, *j);

                    if f2 - f1 >= min_block_size {
                        for k in f1..f2 {
                            visited_from.insert((from_file.clone(), k));
                        }
                        for k in t1..t2 {
                            visited_to.insert((to_file.clone(), k));
                        }

                        matches.push(BlockMatch {
                            from_file: from_file.clone(),
                            from_range: (f1, f2),
                            to_file: to_file.clone(),
                            to_range: (t1, t2),
                        });
                    }
                }
            }
        }
    }

    matches
}

/// Multi-pass block matching: 5 lines → 3 → 1
pub fn find_block_matches_multi(
    from_files: &[(String, String)],
    to_files: &[(String, String)],
) -> Vec<BlockMatch> {
    let mut all = Vec::new();
    for &min_size in &[5, 3, 1] {
        let pass = find_block_matches_single(from_files, to_files, min_size);
        all.extend(pass);
    }
    all
}

