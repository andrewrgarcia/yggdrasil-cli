// src/main.rs
mod args;
mod scanner;
mod formatter;
mod types;
mod diff;
mod formatter_diff;

use args::{Cli, Commands, Args};
use formatter::{CliFormatter, MarkdownFormatter, OutputFormatter};
use scanner::collect_files;
use std::fs::File;
use std::io::{self, Write};
use atty::Stream;
use clap::Parser;
use clap::CommandFactory; // for Cli::command()
use diff::run_diff;


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


fn main() {
    let cli = Cli::parse();

    // If no args at all → show help
    if std::env::args().len() == 1 {
        Cli::command().print_help().unwrap();
        println!();
        return;
    }

    match cli.command {
        Some(Commands::Diff { from, to, align_tags }) => {
            run_diff(from, to, align_tags);
        }
        None => {
            // Make mutable so we can adjust via --whited
            let mut args = cli.args;

            // --- Handle --whited shortcut ---
            if let Some(whited_opt) = &args.whited {
                args.white = Some(None);
                args.contents = true;

                // if user provided a name (e.g. OK.md), use it directly
                let out_name = match whited_opt {
                    Some(name) => name.to_string(),
                    None => "SHOW.md".to_string(),
                };

                args.out = Some(out_name);
            }


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
