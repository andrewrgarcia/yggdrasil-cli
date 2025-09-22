use std::fs;
use similar::{TextDiff, ChangeTag};
use std::collections::{HashMap, HashSet};
use blake3;

#[derive(Debug, Clone)]
pub struct BlockMatch {
    pub from_file: String,
    pub from_range: (usize, usize),
    pub to_file: String,
    pub to_range: (usize, usize),
}

/// Index: hash -> all (file, line_no) occurrences
type LineIndex = HashMap<u64, Vec<(String, usize)>>;

/// Compute a 64-bit hash for a line
fn hash_line(line: &str) -> u64 {
    let h = blake3::hash(line.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&h.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

/// Build an index of all lines in `to` set
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

/// Try to extend a block starting from a matched line
fn extend_block(
    from_lines: &[&str],
    to_lines: &[&str],
    from_start: usize,
    to_start: usize,
) -> (usize, usize, usize, usize) {
    let mut f1 = from_start;
    let mut t1 = to_start;
    let mut f2 = from_start;
    let mut t2 = to_start;

    // extend forward
    while f2 < from_lines.len() && t2 < to_lines.len() && from_lines[f2] == to_lines[t2] {
        f2 += 1;
        t2 += 1;
    }

    // extend backward
    while f1 > 0 && t1 > 0 && from_lines[f1 - 1] == to_lines[t1 - 1] {
        f1 -= 1;
        t1 -= 1;
    }

    (f1, f2, t1, t2)
}

/// Find block matches between two file sets
pub fn find_block_matches(
    from_files: &[(String, String)],
    to_files: &[(String, String)],
) -> Vec<BlockMatch> {
    let index = build_index(to_files);
    let mut matches = Vec::new();
    let mut visited_from: HashSet<(String, usize)> = HashSet::new();
    let mut visited_to: HashSet<(String, usize)> = HashSet::new();

    for (from_file, content) in from_files {
        let from_lines: Vec<&str> = content.lines().collect();
        for (i, line) in from_lines.iter().enumerate() {
            let h = hash_line(line);

            // Skip already matched lines
            if visited_from.contains(&(from_file.clone(), i)) {
                continue;
            }

            if let Some(candidates) = index.get(&h) {
                for (to_file, j) in candidates {
                    if visited_to.contains(&(to_file.clone(), *j)) {
                        continue;
                    }

                    let to_content = to_files.iter().find(|(f, _)| f == to_file).unwrap().1.clone();
                    let to_lines: Vec<&str> = to_content.lines().collect();

                    let (f1, f2, t1, t2) = extend_block(&from_lines, &to_lines, i, *j);

                    // Only record blocks with ≥3 lines
                    if f2 - f1 >= 3 {
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


/// Run a codex diff across two sets of files
pub fn run_diff(from: Vec<String>, to: Vec<String>) {
    let from_set: HashSet<_> = from.iter().collect();
    let to_set: HashSet<_> = to.iter().collect();

    // Deleted files
    for f in from_set.difference(&to_set) {
        println!("- {}", f);
    }

    // New files
    for f in to_set.difference(&from_set) {
        println!("+ {}", f);
    }

    // Common files → inline diffs
    for f in from_set.intersection(&to_set) {
        let from_content = fs::read_to_string(f).unwrap_or_default();
        let to_content = fs::read_to_string(f).unwrap_or_default();

        if from_content != to_content {
            diff_file_contents(f, &from_content, &to_content);
        }
    }

    // --- NEW: cross-file block matches ---
    let from_files: Vec<(String, String)> = from
        .iter()
        .map(|f| (f.clone(), fs::read_to_string(f).unwrap_or_default()))
        .collect();

    let to_files: Vec<(String, String)> = to
        .iter()
        .map(|f| (f.clone(), fs::read_to_string(f).unwrap_or_default()))
        .collect();

    let block_matches = find_block_matches(&from_files, &to_files);

    if !block_matches.is_empty() {
        println!("\n📦 Cross-file block matches:");
        for m in block_matches {
            println!(
                "🔀 {}:{}–{} → {}:{}–{}",
                m.from_file,
                m.from_range.0,
                m.from_range.1,
                m.to_file,
                m.to_range.0,
                m.to_range.1
            );
        }
    }
}


/// Print a line-by-line diff for a single file
fn diff_file_contents(file: &str, from_content: &str, to_content: &str) {
    println!("\n📄 Diff for {file}:\n");

    let diff = TextDiff::from_lines(from_content, to_content);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal  => " ",
        };

        let colored = match change.tag() {
            ChangeTag::Delete => format!("\x1b[91m{}{}\x1b[0m", sign, change),
            ChangeTag::Insert => format!("\x1b[92m{}{}\x1b[0m", sign, change),
            ChangeTag::Equal  => format!(" {}{}", sign, change),
        };

        print!("{}", colored);
    }

    println!();
}
