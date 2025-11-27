use std::fs;
use std::io::Write;
use crate::types::GroupedMatches;

pub struct DiffMarkdownFormatter;

impl crate::formatters::traits::DiffFormatter for DiffMarkdownFormatter {
    fn print_preamble(&self, out: &mut dyn Write) {
        writeln!(out, "# 📦 Cross-file Diff Report\n").unwrap();
    }

    fn print_index(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write) {
        writeln!(out, "## 📄 File Pairs\n").unwrap();
        for g in groups {
            writeln!(out, "- {} → {}", g.from_file, g.to_file).unwrap();
        }
        writeln!(out).unwrap();
    }

    fn print_contents(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write) {
        writeln!(out, "## 📑 Diff Contents\n").unwrap();

        for g in groups {
            writeln!(out, "### {} → {}\n", g.from_file, g.to_file).unwrap();
            writeln!(out, "```python").unwrap();

            if let Ok(src) = fs::read_to_string(&g.from_file) {
                let lines: Vec<&str> = src.lines().collect();
                let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);

                for (lineno, line) in lines.iter().enumerate() {
                    let mut tag = String::new();

                    for bwv in &g.blocks {
                        let m = &bwv.block;
                        let status = if bwv.is_addition { "[ADDED]" } else { "[MOVED]" };

                        if lineno >= m.from_range.0 && lineno < m.from_range.1 {
                            tag = format!(
                                " // {} ({}–{} → {}–{})",
                                status,
                                m.from_range.0,
                                m.from_range.1,
                                m.to_file,
                                m.to_range.0
                            );
                            break;
                        }
                    }

                    writeln!(out, "{:>4} {}", lineno + 1, line).unwrap();
                    if !tag.is_empty() {
                        writeln!(out, "{:>width$}{}", "", tag, width = 6 + max_len).unwrap();
                    }
                }
            }

            writeln!(out, "```\n").unwrap();
        }
    }
}

