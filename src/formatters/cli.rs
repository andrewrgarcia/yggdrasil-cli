use std::fs;
use std::io::Write;
use colored::*;

use crate::types::FileEntry;

use super::traits::OutputFormatter;

pub struct CliFormatter {
    pub colored: bool,
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

        let path_width = files
        .iter()
        .map(|f| f.path.len() + 2) // space + icon
        .max()
        .unwrap_or(0)
        .max(4);

        let line_width = files
            .iter()
            .map(|f| f.line_count.to_string().len())
            .max()
            .unwrap_or(1)
            .max(5);

        let word_width = files
            .iter()
            .map(|f| f.word_count.to_string().len())
            .max()
            .unwrap_or(1)
            .max(5);

        let token_width = files
            .iter()
            .map(|f| f.token_est.to_string().len())
            .max()
            .unwrap_or(1)
            .max(6);

        if self.colored {
            writeln!(out, "{}", "📄 Files".bright_magenta().bold()).unwrap();
        } else {
            writeln!(out, "📄 Files").unwrap();
        }

        // header
        writeln!(
            out,
            "{:<path_w$} : {:>line_w$} | {:>word_w$} | {:>token_w$}",
            "path",
            "lines",
            "words",
            "tokens",
            path_w = path_width,
            line_w = line_width,
            word_w = word_width,
            token_w = token_width
        ).unwrap();

        writeln!(out).unwrap();

        let mut total_lines = 0;

        for entry in files {

            total_lines += entry.line_count;

            if self.colored {
                writeln!(
                    out,
                    "{} {:<path_w$} : {:>line_w$} | {:>word_w$} | {:>token_w$}",
                    "📄".truecolor(255,255,0),
                    entry.path.truecolor(0,255,255),
                    entry.line_count,
                    entry.word_count,
                    entry.token_est,
                    path_w = path_width,
                    line_w = line_width,
                    word_w = word_width,
                    token_w = token_width
                ).unwrap();
            } else {
                writeln!(
                    out,
                    "📄 {:<path_w$} : {:>line_w$} | {:>word_w$} | {:>token_w$}",
                    entry.path,
                    entry.line_count,
                    entry.word_count,
                    entry.token_est,
                    path_w = path_width,
                    line_w = line_width,
                    word_w = word_width,
                    token_w = token_width
                ).unwrap();
            }
        }

        writeln!(out, "\n====").unwrap();
        writeln!(out, "📦 Total LOC: {}", total_lines).unwrap();

        writeln!(
            out,
            "\n{}",
            "===============================================".truecolor(255,255,0)
        ).unwrap();

        if self.colored {
            writeln!(out, "{}", "📑 File Contents".bright_magenta().bold()).unwrap();
        } else {
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

            match fs::read_to_string(&entry.path) {
                Ok(content) => write!(out, "{}", content).unwrap(),
                Err(_) => writeln!(out, "❌ Error reading file").unwrap(),
            };

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
            FileEntry {
                path: "src/main.rs".into(),
                line_count: 10,
                word_count: 20,
                token_est: 27,
            },
            FileEntry {
                path: "src/formatter.rs".into(),
                line_count: 5,
                word_count: 12,
                token_est: 16,
            }
        ]
    }

    #[test]
    fn test_cli_index_plain() {
        let mut buf = Vec::new();
        let fmt = CliFormatter { colored: false };
        fmt.print_index(&sample_files(), &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("📄 Files"));
        assert!(out.contains("src/main.rs"));
        assert!(out.contains("src/formatter.rs"));
        assert!(out.contains("lines")); // column header is present
    }

    #[test]
    fn test_cli_preamble_plaintext() {
        let mut buf = Vec::new();
        let fmt = CliFormatter { colored: false };
        fmt.print_preamble(".", &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("✨ Project Snapshot: ."));
        assert!(out.contains("📄 Files"));
    }

    #[test]
    fn test_file_contents_marker() {
        let mut buf = Vec::new();
        let fmt = CliFormatter { colored: false };
        fmt.print_contents(&sample_files(), &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("<<< FILE START: src/main.rs >>>"));
        assert!(out.contains("<<< FILE END: src/main.rs >>>"));
    }
}
