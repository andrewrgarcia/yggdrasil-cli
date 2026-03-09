use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::FileEntry;

use super::traits::OutputFormatter;

#[allow(dead_code)]
pub struct MarkdownFormatter {
    pub show_lines: bool,
}

impl OutputFormatter for MarkdownFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write) {
        let abs_path = std::path::Path::new(root)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(root));

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

        let header = "path";

        let path_width = files
            .iter()
            .map(|f| f.path.len())
            .max()
            .unwrap_or(0)
            .max(header.len());

        let line_width = files
            .iter()
            .map(|f| f.line_count.to_string().len())
            .max()
            .unwrap_or(1);

        let word_width = files
            .iter()
            .map(|f| f.word_count.to_string().len())
            .max()
            .unwrap_or(1);

        let token_width = files
            .iter()
            .map(|f| f.token_est.to_string().len())
            .max()
            .unwrap_or(1);

        writeln!(
            out,
            "{:<path_w$} : {:>line_w$} | {:>word_w$} | {:>token_w$}",
            "path",
            "lines",
            "words",
            "tokens",
            path_w = path_width,
            line_w = line_width.max(5),
            word_w = word_width.max(5),
            token_w = token_width.max(6),
        ).unwrap();

        writeln!(out).unwrap();

        for entry in files {

            total_lines += entry.line_count;

            writeln!(
                out,
                "{:<path_w$} : {:>line_w$} | {:>word_w$} | {:>token_w$}",
                entry.path,
                entry.line_count,
                entry.word_count,
                entry.token_est,
                path_w = path_width,
                line_w = line_width.max(5),
                word_w = word_width.max(5),
                token_w = token_width.max(6),
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
    fn test_markdown_preamble() {
        let mut buf = Vec::new();
        let fmt = MarkdownFormatter { show_lines: false };
        fmt.print_preamble(".", &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("# CODEX"));
        assert!(out.contains("project_path:"));
        assert!(out.contains("timestamp_unix:"));
        assert!(out.contains("## INDEX"));
    }

    #[test]
    fn test_markdown_index_lists_files() {
        let mut buf = Vec::new();
        let fmt = MarkdownFormatter { show_lines: false };
        fmt.print_index(&sample_files(), &mut buf);
        let out = String::from_utf8(buf).unwrap();

        assert!(out.contains("src/main.rs"));
        assert!(out.contains("src/formatter.rs"));
        assert!(out.contains("total_loc: 15"));
    }
}

