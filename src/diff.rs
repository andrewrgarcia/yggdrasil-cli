use std::fs;
use std::collections::{HashMap, HashSet, BTreeMap};
use std::path::Path;
use similar::{TextDiff, ChangeTag};
use blake3;
use atty::Stream;
use crate::types::{BlockMatch, BlockWithVote, GroupedMatches};
use crate::formatter_diff::{DiffFormatter, DiffCliFormatter, DiffMarkdownFormatter};

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

/// Detects structural boundaries (likely start/end of a block)
fn is_boundary(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty()
        || trimmed.starts_with("class ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("@")
}

/// Extend a block from a matched line, respecting structural boundaries
fn extend_structural_block(
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
        if is_boundary(from_lines[f2]) || is_boundary(to_lines[t2]) {
            break;
        }
        f2 += 1;
        t2 += 1;
    }

    // extend backward
    while f1 > 0 && t1 > 0 && from_lines[f1 - 1] == to_lines[t1 - 1] {
        if is_boundary(from_lines[f1 - 1]) || is_boundary(to_lines[t1 - 1]) {
            break;
        }
        f1 -= 1;
        t1 -= 1;
    }

    (f1, f2, t1, t2)
}

/// Find block matches between file sets
pub fn find_block_matches(
    from_files: &[(String, String)],
    to_files: &[(String, String)],
    min_block_size: usize,
) -> Vec<BlockMatch> {
    let index = build_index(to_files);
    let mut matches = Vec::new();
    let mut visited_from: HashSet<(String, usize)> = HashSet::new();
    let mut visited_to: HashSet<(String, usize)> = HashSet::new();

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

                    let to_content = to_files
                        .iter()
                        .find(|(f, _)| f == to_file)
                        .unwrap()
                        .1
                        .clone();
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

/// Multi-pass matching with different block size thresholds
pub fn find_block_matches_multi(
    from_files: &[(String, String)],
    to_files: &[(String, String)],
) -> Vec<BlockMatch> {
    let mut all_matches = Vec::new();
    let mut visited_from: HashSet<(String, usize)> = HashSet::new();
    let mut visited_to: HashSet<(String, usize)> = HashSet::new();

    for &min_block_size in &[5, 3, 1] {
        let matches = find_block_matches(from_files, to_files, min_block_size)
            .into_iter()
            .filter(|m| m.from_range.1 - m.from_range.0 >= min_block_size)
            .collect::<Vec<_>>();

        for m in &matches {
            for i in m.from_range.0..m.from_range.1 {
                visited_from.insert((m.from_file.clone(), i));
            }
            for j in m.to_range.0..m.to_range.1 {
                visited_to.insert((m.to_file.clone(), j));
            }
        }
        all_matches.extend(matches);
    }

    all_matches
}

/// Expand a list of files/dirs into flat file paths (recursively)
fn expand_paths(paths: &[String]) -> Vec<String> {
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

fn hash_block(lines: &[&str]) -> u64 {
    let joined = lines.join("\n");
    let h = blake3::hash(joined.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&h.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

/// Group matches and tag duplicates as additions
fn group_and_filter_matches(
    matches: Vec<BlockMatch>,
    from_files: &[(String, String)],
) -> Vec<GroupedMatches> {
    let mut grouped: BTreeMap<(String, String), Vec<BlockWithVote>> = BTreeMap::new();

    // Map file → line vector
    let mut file_lines: HashMap<String, Vec<String>> = HashMap::new();
    for (fname, content) in from_files {
        file_lines.insert(fname.clone(), content.lines().map(|s| s.to_string()).collect());
    }

    // Track hashes → whether already seen
    let mut seen_blocks: HashMap<u64, bool> = HashMap::new();

    for m in matches {
        let mut is_addition = false;

        if let Some(lines) = file_lines.get(&m.from_file) {
            let block: Vec<&str> = lines[m.from_range.0..m.from_range.1]
                .iter()
                .map(|s| s.as_str())
                .collect();
            let bhash = hash_block(&block);

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
        .map(|((from, to), blocks)| GroupedMatches {
            from_file: from,
            to_file: to,
            blocks,
        })
        .collect()
}

/// Run a codex diff across two sets of files
pub fn run_diff(from: Vec<String>, to: Vec<String>, align_tags: bool) {
    let from_files = expand_paths(&from);
    let to_files   = expand_paths(&to);

    let from_set: HashSet<_> = from_files.iter().collect();
    let to_set: HashSet<_> = to_files.iter().collect();

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
    let from_files_content: Vec<(String, String)> = from_files
        .iter()
        .map(|f| (f.clone(), fs::read_to_string(f).unwrap_or_default()))
        .collect();

    let to_files_content: Vec<(String, String)> = to_files
        .iter()
        .map(|f| (f.clone(), fs::read_to_string(f).unwrap_or_default()))
        .collect();

    let block_matches = find_block_matches_multi(&from_files_content, &to_files_content);

    if !block_matches.is_empty() {
        let grouped = group_and_filter_matches(block_matches, &from_files_content);

        if !grouped.is_empty() {
            // pick formatter
            let use_md = to.iter().any(|f| f.ends_with(".md"));
            let mut writer: Box<dyn std::io::Write> = Box::new(std::io::stdout());

            if use_md {
                let fmt = DiffMarkdownFormatter;
                fmt.print_preamble(&mut *writer);
                fmt.print_index(&grouped, &mut *writer);
                fmt.print_contents(&grouped, &mut *writer);
            } else {
                let fmt = DiffCliFormatter {
                    colored: atty::is(Stream::Stdout),
                    align_tags: align_tags,
                };
                fmt.print_preamble(&mut *writer);
                fmt.print_index(&grouped, &mut *writer);
                fmt.print_contents(&grouped, &mut *writer);
            }
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
            ChangeTag::Equal => " ",
        };

        let colored = match change.tag() {
            ChangeTag::Delete => format!("\x1b[91m{}{}\x1b[0m", sign, change),
            ChangeTag::Insert => format!("\x1b[92m{}{}\x1b[0m", sign, change),
            ChangeTag::Equal => format!(" {}{}", sign, change),
        };

        print!("{}", colored);
    }

    println!();
}
