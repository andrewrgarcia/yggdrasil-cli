use crate::cli::Args;
use crate::formatters::{CliFormatter, MarkdownFormatter};

use crate::formatters::traits::OutputFormatter;
use atty::Stream;

pub fn select_formatter<'a>(
    args: &Args,
) -> Box<dyn OutputFormatter + 'a> {

    let use_md = if let Some(out_file) = &args.out {
        out_file.ends_with(".md")
    } else {
        args.md
    };

    if use_md {
        Box::new(MarkdownFormatter { show_lines: !args.no_lines })
    } else {
        Box::new(CliFormatter {
            colored: args.out.is_none() && atty::is(Stream::Stdout),
            show_lines: !args.no_lines,
        })
    }
}

