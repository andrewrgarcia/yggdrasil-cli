mod args;
mod scanner;
mod formatter;
mod types;

use args::Args;
use formatter::{CliFormatter, MarkdownFormatter, OutputFormatter};
use scanner::collect_files;
use std::fs::File;
use std::io::{self, Write};
use atty::Stream;
use clap::Parser;
use clap::CommandFactory; // for Args::command()

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
    let args = Args::parse();

    // If no extra args are provided, show help
    if std::env::args().len() == 1 {
        Args::command().print_help().unwrap();
        println!();
        return;
    }

    let root = args.dir.clone();

    // Choose output destination
    let mut writer: Box<dyn Write> = if let Some(out_file) = &args.out {
        Box::new(File::create(out_file).expect("Failed to create output file"))
    } else {
        Box::new(io::stdout())
    };

    // Formatter detection:
    // 1. If --out ends with .md → Markdown
    // 2. Else if --md flag given → Markdown
    // 3. Else → CLI
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
