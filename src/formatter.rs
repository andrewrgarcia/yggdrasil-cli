use colored::*;
use std::fs;
use std::io::Write;

pub trait OutputFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write);
    fn print_index(&self, files: &Vec<String>, out: &mut dyn Write);
    fn print_contents(&self, files: &Vec<String>, out: &mut dyn Write);
}

/// Markdown-style formatter
pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write) {
        writeln!(out, "# ✨ Directory Codex: {}\n", root).unwrap();
        writeln!(out, "*Made with [Yggdrasil](https://crates.io/crates/yggdrasil)*  \n").unwrap();
        writeln!(out, "*This document contains two sections:*  ").unwrap();
        writeln!(out, "- **Files** → index of all paths.  ").unwrap();
        writeln!(out, "- **File Contents** → full text for each file under `### <path>`.  \n").unwrap();
        writeln!(out, "## 📄 Files").unwrap();
    }

    fn print_index(&self, files: &Vec<String>, out: &mut dyn Write) {
        for file in files {
            let anchor = file.replace("/", "-").replace(".", "-");
            writeln!(out, "- [{}](#{})", file, anchor).unwrap();
        }
        writeln!(out, "\n---\n\n## 📑 File Contents\n").unwrap();
    }

    fn print_contents(&self, files: &Vec<String>, out: &mut dyn Write) {
        for file in files {
            // Create a stable anchor id
            let anchor = file.replace("/", "-").replace(".", "-");

            // Explicit HTML anchor (so links always work)
            writeln!(out, "<a id=\"{}\"></a>", anchor).unwrap();

            // File heading
            writeln!(out, "### <{}>", file).unwrap();

            // Code block with file content
            writeln!(out, "```").unwrap();
            if let Ok(content) = fs::read_to_string(file) {
                write!(out, "{}", content).unwrap();
            } else {
                writeln!(out, "❌ Error reading file").unwrap();
            }
            writeln!(out, "```\n").unwrap();
        }
    }
}


pub struct CliFormatter {
    pub colored: bool,
}

impl OutputFormatter for CliFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write) {
        if self.colored {
            let title = "✨ Directory Codex:".bright_magenta().bold();
            let path = root.truecolor(0, 255, 255).bold();
            writeln!(out, "{} {}", title, path).unwrap();

            let link = format!(
                "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
                "https://crates.io/crates/yggdrasil-cli",
                "*Made with Yggdrasil*".truecolor(255, 255, 0).bold()
            );
            writeln!(out, "{}", link).unwrap();
        } else {
            writeln!(out, "✨ Directory Codex: {}", root).unwrap();
            writeln!(out, "*Made with Yggdrasil* (https://crates.io/crates/yggdrasil-cli)").unwrap();
        }

        writeln!(
            out,
            "\nSchema: index first, then file contents.\n\
            - Files are listed under '📄 Files'.\n\
            - Contents are shown with markers <<< FILE START: <path> >>> … <<< FILE END: <path> >>>\n"
        )
        .unwrap();
    }

    fn print_index(&self, files: &Vec<String>, out: &mut dyn Write) {
        if self.colored {
            writeln!(out, "{}", "📄 Files".bright_magenta().bold()).unwrap();
            for file in files {
                let icon = "📄".truecolor(255, 255, 0);
                let text = file.truecolor(0, 255, 255);
                writeln!(out, "{} {}", icon, text).unwrap();
            }
            writeln!(
                out,
                "\n{}",
                "===============================================".truecolor(255, 255, 0)
            )
            .unwrap();
            writeln!(out, "{}", "📑 File Contents".bright_magenta().bold()).unwrap();
        } else {
            writeln!(out, "📄 Files").unwrap();
            for file in files {
                writeln!(out, "📄 {}", file).unwrap();
            }
            writeln!(out, "\n===============================================").unwrap();
            writeln!(out, "📑 File Contents").unwrap();
        }
    }

    fn print_contents(&self, files: &Vec<String>, out: &mut dyn Write) {
        for file in files {
            if self.colored {
                writeln!(
                    out,
                    "{} <{}> {}",
                    "<<< FILE START:".bright_magenta().bold(),
                    file,
                    ">>>".bright_magenta().bold()
                )
                .unwrap();
            } else {
                writeln!(out, "<<< FILE START: {} >>>", file).unwrap();
            }

            if let Ok(content) = fs::read_to_string(file) {
                write!(out, "{}", content).unwrap();
            } else {
                writeln!(out, "❌ Error reading file").unwrap();
            }

            if self.colored {
                writeln!(
                    out,
                    "{} <{}> {}",
                    "<<< FILE END:".bright_magenta().bold(),
                    file,
                    ">>>".bright_magenta().bold()
                )
                .unwrap();
            } else {
                writeln!(out, "<<< FILE END: {} >>>", file).unwrap();
            }

            writeln!(out).unwrap();
        }
    }
}
