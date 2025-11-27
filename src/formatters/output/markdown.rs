use std::fs;
use std::io::Write;
use crate::formatters::traits::OutputFormatter;
use crate::types::FileEntry;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MarkdownFormatter {
    pub show_lines: bool,
}

impl OutputFormatter for MarkdownFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write) {
        let abs_path = std::path::Path::new(root)
            .canonicalize()
            .unwrap_or_else(|_| root.into());

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
            writeln!(out, "{}: {}", entry.path, entry.line_count).unwrap();
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

