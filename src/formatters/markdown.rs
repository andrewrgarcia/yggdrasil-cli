use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use super::metadata::infer_metadata;
use super::symbols::extract_symbols;
use super::traits::OutputFormatter;
use crate::types::FileEntry;

#[allow(dead_code)]
pub struct MarkdownFormatter {
    pub show_lines: bool,
    pub show_files_section: bool,
}


fn escape_meta_value(value: &str) -> String {
    value.replace('\n', "\\n").replace('\r', "\\r")
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

        writeln!(out, "# YGG CODEX").unwrap();

        writeln!(out, "transport: ygg_packet").unwrap();

        writeln!(out, "schema: ai-native").unwrap();

        writeln!(
            out,
            "project: {}",
            project_name
        ).unwrap();

        writeln!(
            out,
            "maintainer: Andrew R. Garcia"
        ).unwrap();

        writeln!(
            out,
            "project_path: {}",
            abs_path.display()
        ).unwrap();

        writeln!(
            out,
            "generated_by: yggdrasil-cli"
        ).unwrap();

        writeln!(
            out,
            "timestamp_unix: {}",
            timestamp
        ).unwrap();

        writeln!(
            out,
            "format: markdown\n"
        ).unwrap();
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
        )
        .unwrap();

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
            )
            .unwrap();
        }

        writeln!(out, "total_loc: {}\n", total_lines).unwrap();

        if self.show_files_section {
            writeln!(out, "## FILES").unwrap();
        }
    }

    fn print_contents(&self, files: &Vec<FileEntry>, out: &mut dyn Write) {
        for entry in files.iter() {

            let meta = infer_metadata(&entry.path);

            let content = match fs::read_to_string(&entry.path) {
                Ok(c) => c,
                Err(_) => "❌ Error reading file\n".to_string(),
            };

            let symbols =
                extract_symbols(
                    &meta.lang,
                    &content
                );

            writeln!(
                out,
                "=== FILE_BEGIN ==="
            ).unwrap();

            writeln!(
                out,
                "path={}",
                escape_meta_value(
                    &entry.path
                )
            ).unwrap();

            writeln!(
                out,
                "lang={}",
                meta.lang
            ).unwrap();

            writeln!(
                out,
                "lines={}",
                entry.line_count
            ).unwrap();

            writeln!(
                out,
                "words={}",
                entry.word_count
            ).unwrap();

            writeln!(
                out,
                "tokens={}",
                entry.token_est
            ).unwrap();

            if !symbols.is_empty() {

                writeln!(
                    out,
                    "symbols={}",
                    symbols.join(",")
                ).unwrap();

            }

            writeln!(out).unwrap();

            writeln!(
                out,
                "--- CONTENT ---"
            ).unwrap();

            writeln!(out).unwrap();

            if content.ends_with('\n') {
                write!(
                    out,
                    "{}",
                    content
                ).unwrap();

            } else {

                writeln!(
                    out,
                    "{}",
                    content
                ).unwrap();
            }

            writeln!(out).unwrap();

            writeln!(
                out,
                "=== FILE_END ==="
            ).unwrap();

            writeln!(out).unwrap();
        }
    }
    
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::super::metadata::infer_metadata;
    use crate::types::FileEntry;


    #[test]
    fn test_markdown_preamble() {

        let mut buf=Vec::new();

        let fmt=MarkdownFormatter{
            show_lines:false,
            show_files_section:true,
        };

        fmt.print_preamble(
            ".",
            &mut buf
        );

        let out=
            String::from_utf8(buf)
            .unwrap();

        assert!(
            out.contains(
                "# YGG CODEX"
            )
        );

        assert!(
            out.contains(
                "maintainer: Andrew R. Garcia"
            )
        );

        assert!(
            out.contains(
                "schema: ai-native"
            )
        );
    }

    #[test]
    fn test_metadata_detection() {

        let meta=
            infer_metadata(
                "src/main.rs"
            );

        assert_eq!(
            meta.lang,
            "rust"
        );
    }

    #[test]
    fn test_escape_metadata() {

        let s=
            escape_meta_value(
                "hello\nworld\rtest"
            );

        assert_eq!(
            s,
            "hello\\nworld\\rtest"
        );
    }

    #[test]
    fn test_packet_generation() {

        let mut buf=
            Vec::new();

        let fmt=
            MarkdownFormatter{
                show_lines:false,
                show_files_section:true,
            };

        let files=vec![
            FileEntry{
                path:"Cargo.toml".into(),
                line_count:1,
                word_count:1,
                token_est:1,
            }
        ];

        fmt.print_contents(
            &files,
            &mut buf
        );

        let out=
            String::from_utf8(buf)
            .unwrap();

        assert!(
            out.contains(
                "=== FILE_BEGIN ==="
            )
        );

        assert!(
            out.contains(
                "path=Cargo.toml"
            )
        );

        assert!(
            out.contains(
                "lang=toml"
            )
        );

        assert!(
            out.contains(
                "--- CONTENT ---"
            )
        );

        assert!(
            out.contains(
                "=== FILE_END ==="
            )
        );
    }
}