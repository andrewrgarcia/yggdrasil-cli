use std::fs;
use std::collections::HashSet;

use atty::Stream;

use crate::formatter_diff::{DiffFormatter};
use crate::formatters::diff::{DiffCliFormatter, DiffMarkdownFormatter};

use super::expand::expand_paths;
use super::matcher::find_block_matches_multi;
use super::crossfile::group_and_filter_matches;
use super::inline::diff_file_contents;

/// Main diff orchestrator.
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

    // Same-file diffs
    for f in from_set.intersection(&to_set) {
        let from_content = fs::read_to_string(f).unwrap_or_default();
        let to_content = fs::read_to_string(f).unwrap_or_default();

        if from_content != to_content {
            diff_file_contents(f, &from_content, &to_content);
        }
    }

    // Cross-file semantic block diffs
    let from_pairs: Vec<_> = from_files
        .iter()
        .map(|f| (f.clone(), fs::read_to_string(f).unwrap_or_default()))
        .collect();

    let to_pairs: Vec<_> = to_files
        .iter()
        .map(|f| (f.clone(), fs::read_to_string(f).unwrap_or_default()))
        .collect();

    let block_matches = find_block_matches_multi(&from_pairs, &to_pairs);

    if block_matches.is_empty() {
        return;
    }

    let grouped = group_and_filter_matches(block_matches, &from_pairs);

    if grouped.is_empty() {
        return;
    }

    // Choose formatter
    let use_md = to.iter().any(|f| f.ends_with(".md"));
    let mut out: Box<dyn std::io::Write> = Box::new(std::io::stdout());

    if use_md {
        let fmt = DiffMarkdownFormatter;
        fmt.print_preamble(&mut *out);
        fmt.print_index(&grouped, &mut *out);
        fmt.print_contents(&grouped, &mut *out);
    } else {
        let fmt = DiffCliFormatter {
            colored: atty::is(Stream::Stdout),
            align_tags,
        };
        fmt.print_preamble(&mut *out);
        fmt.print_index(&grouped, &mut *out);
        fmt.print_contents(&grouped, &mut *out);
    }
}

