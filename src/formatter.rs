use colored::*;
use std::fs;
use std::io::Write;
use crate::types::FileEntry;
use std::time::{SystemTime, UNIX_EPOCH};

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
        // Try to resolve absolute path and derive project name
        let abs_path = std::path::Path::new(root)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(root));

        // Get final directory name (project folder)
        let project_name = abs_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(root);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        writeln!(out, "# CODEX").unwrap();
        writeln!(out, "project: {}", project_name).unwrap();
        writeln!(out, "project_path: {}", abs_path.display()).unwrap();
        writeln!(out, "generated_by: yggdrasil-cli").unwrap();
        writeln!(out, "timestamp_unix: {}", timestamp).unwrap();
        writeln!(out, "format: markdown\n").unwrap();
        writeln!(out, "## INDEX").unwrap();
    }

    fn print_index(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        let mut total_lines = 0usize;
        for entry in files {
            total_lines += entry.line_count;
            writeln!(
                out,
                "{}: {}",
                entry.path,
                entry.line_count
            ).unwrap();
        }
        writeln!(out, "total_loc: {}\n", total_lines).unwrap();

        writeln!(out, "## FILES").unwrap();
    }

    fn print_contents(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        for entry in files {
            let lang = match entry.path.split('.').last() {
                Some("rs") => "rust",
                Some("py") => "python",
                Some("tex") => "latex",
                Some("md") => "markdown",
                Some("js") | Some("ts") | Some("tsx") => "typescript",
                Some(ext) => ext,
                None => "text",
            };

            writeln!(
                out,
                "<file path=\"{}\" lang=\"{}\" lines=\"{}\">",
                entry.path, lang, entry.line_count
            ).unwrap();
            writeln!(out, "```{}", lang).unwrap();

            match fs::read_to_string(&entry.path) {
                Ok(content) => {
                    if content.ends_with('\n') {
                        write!(out, "{}", content).unwrap();
                    } else {
                        writeln!(out, "{}", content).unwrap();
                    }
                }
                Err(_) => writeln!(out, "❌ Error reading file").unwrap(),
            }

            writeln!(out, "```\n</file>\n").unwrap();
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
