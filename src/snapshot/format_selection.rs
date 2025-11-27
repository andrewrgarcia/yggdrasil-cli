use crate::args::Args;
use crate::formatters::output::{CliFormatter, MarkdownFormatter};
use crate::formatters::traits::OutputFormatter;
use atty::Stream;

/// Decide whether to use markdown or CLI formatter.
pub fn select_formatter<'a>(
    args: &Args,
    writer: &'a Box<dyn std::io::Write>,
) -> Box<dyn OutputFormatter + 'a> {

    // if --out ends with .md → forced markdown
    let use_md = if let Some(out_file) = &args.out {
        out_file.ends_with(".md")
    } else {
        args.md
    };

    if use_md {
        Box::new(MarkdownFormatter {
            show_lines: !args.no_lines,
        })
    } else {
        Box::new(CliFormatter {
            colored: args.out.is_none() && atty::is(Stream::Stdout),
            show_lines: !args.no_lines,
        })
    }
}
