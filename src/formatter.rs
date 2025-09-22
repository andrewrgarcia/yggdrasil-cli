use colored::*;
use std::fs;
use std::io::Write;
use crate::types::FileEntry;

pub trait OutputFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write);
    fn print_index(&self, files: &Vec<FileEntry>, out: &mut dyn Write);
    fn print_contents(&self, files: &Vec<FileEntry>, out: &mut dyn Write);
}

/// Markdown-style formatter
pub struct MarkdownFormatter {
    pub show_lines: bool,
}

/// CLI-style formatter
pub struct CliFormatter {
    pub colored: bool,
    pub show_lines: bool,
}

impl OutputFormatter for MarkdownFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write) {
        writeln!(out, "# ✨ Project Snapshot: {}\n", root).unwrap();
        writeln!(out, "*Made with [Yggdrasil](https://crates.io/crates/yggdrasil)*  \n").unwrap();
        writeln!(out, "*This document contains two sections:*").unwrap();
        writeln!(out, "- **Files** → index of all paths.").unwrap();
        writeln!(out, "- **File Contents** → full text for each file under `### <path>`.\n").unwrap();
        writeln!(out, "## 📄 Files").unwrap();
    }

    fn print_index(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        if self.show_lines {
            let mut total_lines = 0;

            for entry in files {
                let anchor = entry.path.replace("/", "-").replace(".", "-");
                total_lines += entry.line_count;
                writeln!(out, "- [{} ({} lines)](#{})", entry.path, entry.line_count, anchor).unwrap();
            }

            writeln!(out, "\n====").unwrap();
            writeln!(out, "- [📦 Total LOC ({} lines)](#)\n", total_lines).unwrap();
        } else {
            for entry in files {
                let anchor = entry.path.replace("/", "-").replace(".", "-");
                writeln!(out, "- [{}](#{})", entry.path, anchor).unwrap();
            }
        }

        writeln!(out, "\n---\n\n## 📑 File Contents\n").unwrap();
    }



    fn print_contents(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        for entry in files {
            let anchor = entry.path.replace("/", "-").replace(".", "-");

            writeln!(out, "<a id=\"{anchor}\"></a>").unwrap();
            writeln!(out, "### <{}>", entry.path).unwrap();

            writeln!(out, "```").unwrap();
            if let Ok(content) = fs::read_to_string(&entry.path) {
                if content.ends_with('\n') {
                    write!(out, "{}", content).unwrap();
                } else {
                    writeln!(out, "{}", content).unwrap();
                }
            } else {
                writeln!(out, "❌ Error reading file").unwrap();
            }
            writeln!(out, "```\n").unwrap();
        }
    }
}

impl OutputFormatter for CliFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write) {
        if self.colored {
            let title = "✨ Project Snapshot:".bright_magenta().bold();
            let path = root.truecolor(0, 255, 255).bold();
            writeln!(out, "{} {}", title, path).unwrap();

            let link = format!(
                "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
                "https://crates.io/crates/yggdrasil-cli",
                "*Made with Yggdrasil*".truecolor(255, 255, 0).bold()
            );
            writeln!(out, "{}", link).unwrap();
        } else {
            writeln!(out, "✨ Project Snapshot: {}", root).unwrap();
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

    fn print_index(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        let max_len = files.iter().map(|f| f.path.len()).max().unwrap_or(0);

        if self.colored {
            writeln!(out, "{}", "📄 Files".bright_magenta().bold()).unwrap();
            for entry in files {
                let icon = "📄".truecolor(255, 255, 0);
                let text = entry.path.truecolor(0, 255, 255);

                if self.show_lines {
                    writeln!(
                        out,
                        "{} {:<width$} {} lines",
                        icon,
                        text,
                        entry.line_count,
                        width = max_len + 2
                    ).unwrap();
                } else {
                    writeln!(out, "{} {:<width$}", icon, text, width = max_len + 2).unwrap();
                }
            }

            if self.show_lines {
                let total_lines: usize = files.iter().map(|f| f.line_count).sum();

                writeln!(out, "\n====").unwrap();
                writeln!(
                    out,
                    "{:<width$} {} lines\n",
                    "📦 Total LOC".bright_magenta().bold(),
                    total_lines.to_string().bright_magenta().bold(),
                    width = max_len + 2
                ).unwrap();
            }

            writeln!(
                out,
                "\n{}",
                "===============================================".truecolor(255, 255, 0)
            ).unwrap();
            writeln!(out, "{}", "📑 File Contents".bright_magenta().bold()).unwrap();
        } else {
            writeln!(out, "📄 Files").unwrap();
            for entry in files {
                if self.show_lines {
                    writeln!(
                        out,
                        "📄 {:<width$} {} lines",
                        entry.path,
                        entry.line_count,
                        width = max_len + 2
                    ).unwrap();
                } else {
                    writeln!(out, "📄 {:<width$}", entry.path, width = max_len + 2).unwrap();
                }
            }

            if self.show_lines {
                let total_lines: usize = files.iter().map(|f| f.line_count).sum();

                writeln!(out, "\n====").unwrap();
                writeln!(
                    out,
                    "📄 {:<width$} {} lines\n",
                    "📦 Total LOC",
                    total_lines,
                    width = max_len + 2
                ).unwrap();
            }

            writeln!(out, "\n===============================================").unwrap();
            writeln!(out, "📑 File Contents").unwrap();
        }
    }


    fn print_contents(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        for entry in files {
            if self.colored {
                writeln!(
                    out,
                    "{} <{}> {}",
                    "<<< FILE START:".bright_magenta().bold(),
                    entry.path,
                    ">>>".bright_magenta().bold()
                ).unwrap();
            } else {
                writeln!(out, "<<< FILE START: {} >>>", entry.path).unwrap();
            }

            if let Ok(content) = fs::read_to_string(&entry.path) {
                write!(out, "{}", content).unwrap();
            } else {
                writeln!(out, "❌ Error reading file").unwrap();
            }

            if self.colored {
                writeln!(
                    out,
                    "{} <{}> {}",
                    "<<< FILE END:".bright_magenta().bold(),
                    entry.path,
                    ">>>".bright_magenta().bold()
                ).unwrap();
            } else {
                writeln!(out, "<<< FILE END: {} >>>", entry.path).unwrap();
            }

            writeln!(out).unwrap();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileEntry;

    fn sample_files() -> Vec<FileEntry> {
        vec![
            FileEntry { path: "src/main.rs".into(), line_count: 10 },
            FileEntry { path: "src/formatter.rs".into(), line_count: 5 },
        ]
    }

    #[test]
    fn test_markdown_preamble() {
        let mut buf = Vec::new();
        let fmt = MarkdownFormatter { show_lines: false };
        fmt.print_preamble(".", &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("# ✨ Project Snapshot: ."));
        assert!(out.contains("## 📄 Files"));
    }

    #[test]
    fn test_markdown_index_contains_links() {
        let mut buf = Vec::new();
        let fmt = MarkdownFormatter { show_lines: false };
        fmt.print_index(&sample_files(), &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("- [src/main.rs](#src-main-rs)"));
        assert!(out.contains("- [src/formatter.rs](#src-formatter-rs)"));
    }

    #[test]
    fn test_cli_index_plain() {
        let mut buf = Vec::new();
        let fmt = CliFormatter { colored: false, show_lines: false };
        fmt.print_index(&sample_files(), &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("📄 Files"));
        assert!(out.contains("src/main.rs"));
        assert!(!out.contains("10 lines")); // ensure no line counts
    }

    #[test]
    fn test_cli_preamble_plaintext() {
        let mut buf = Vec::new();
        let fmt = CliFormatter { colored: false, show_lines: false };
        fmt.print_preamble(".", &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("✨ Project Snapshot: ."));
        assert!(out.contains("📄 Files"));
    }

    #[test]
    fn test_file_contents_marker() {
        let mut buf = Vec::new();
        let fmt = CliFormatter { colored: false, show_lines: false };
        fmt.print_contents(&sample_files(), &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("<<< FILE START: src/main.rs >>>"));
        assert!(out.contains("<<< FILE END: src/main.rs >>>"));
    }
}
