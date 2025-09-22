// src/main.rs
mod args;
mod scanner;
mod formatter;
mod types;

use args::{Cli, Commands, Args};
use formatter::{CliFormatter, MarkdownFormatter, OutputFormatter};
use scanner::collect_files;
use std::fs::File;
use std::io::{self, Write};
use atty::Stream;
use clap::Parser;
use clap::CommandFactory; // for Cli::command()

fn run(
    fmt: &dyn OutputFormatter,
    args: &Args,
    root: &str,
    out: &mut dyn std::io::Write,
) {
    let files = collect_files(args);

    fmt.print_preamble(root, out);
    fmt.print_index(&files, out);

    if args.contents {
        fmt.print_contents(&files, out);
    }
}

use std::collections::HashSet;
use std::fs;
use similar::{TextDiff, ChangeTag};

fn run_diff(from: Vec<String>, to: Vec<String>) {
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

    // Common files → line diffs
    for f in from_set.intersection(&to_set) {
        let from_content = fs::read_to_string(f).unwrap_or_default();
        let to_content = fs::read_to_string(f).unwrap_or_default();

        if from_content == to_content {
            continue; // unchanged
        }

        println!("\n📄 Diff for {f}:\n");

        let diff = TextDiff::from_lines(&from_content, &to_content);

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal  => " ",
            };

            // Use ANSI color
            let colored = match change.tag() {
                ChangeTag::Delete => format!("\x1b[91m{}{}\x1b[0m", sign, change),
                ChangeTag::Insert => format!("\x1b[92m{}{}\x1b[0m", sign, change),
                ChangeTag::Equal  => format!(" {}{}", sign, change),
            };
            print!("{}", colored);
        }

        println!();
    }
}


fn main() {
    let cli = Cli::parse();

    // If no args at all → show help
    if std::env::args().len() == 1 {
        Cli::command().print_help().unwrap();
        println!();
        return;
    }

    match cli.command {
        Some(Commands::Diff { from, to }) => {
            run_diff(from, to);
        }
        None => {
            let args = cli.args;
            let root = args.dir.clone();

            // Choose output destination
            let mut writer: Box<dyn Write> = if let Some(out_file) = &args.out {
                Box::new(File::create(out_file).expect("Failed to create output file"))
            } else {
                Box::new(io::stdout())
            };

            // Formatter detection
            let use_md = if let Some(out_file) = &args.out {
                out_file.ends_with(".md")
            } else {
                args.md
            };

            if use_md {
                let fmt = MarkdownFormatter { show_lines: !args.no_lines };
                run(&fmt, &args, &root, &mut *writer);
            } else {
                let fmt = CliFormatter {
                    colored: args.out.is_none() && atty::is(Stream::Stdout),
                    show_lines: !args.no_lines,
                };
                run(&fmt, &args, &root, &mut *writer);
            }
        }
    }
}
