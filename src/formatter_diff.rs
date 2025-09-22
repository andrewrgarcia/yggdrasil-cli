// src/formatter_diff.rs
use std::fs;
use std::io::Write;
use colored::*;
use crate::types::GroupedMatches;

/// Trait for diff-specific formatters
pub trait DiffFormatter {
    fn print_preamble(&self, out: &mut dyn Write);
    fn print_index(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write);
    fn print_contents(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write);
}

/// Markdown-style diff formatter
pub struct DiffMarkdownFormatter;

/// CLI-style diff formatter
pub struct DiffCliFormatter {
    pub colored: bool,
}

impl DiffFormatter for DiffMarkdownFormatter {
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
                for (lineno, line) in lines.iter().enumerate() {
                    let tag = String::new();

                    for bwv in &g.blocks {
                        let m = &bwv.block;
                        let tag = if bwv.is_addition {
                            format!(" // [ADD] ({}–{} → {}–{})",
                                m.from_range.0, m.from_range.1, m.to_file, m.to_range.0)
                        } else {
                            format!(" // [MOVED] ({}–{} → {}–{})",
                                m.from_range.0, m.from_range.1, m.to_file, m.to_range.0)
                        };

                        if lineno >= m.from_range.0 && lineno < m.from_range.1 {
                            writeln!(out, "{:>4} {}{}", lineno + 1, line, tag).unwrap();
                            break;
                        }
                    }


                    writeln!(out, "{:>4} {}{}", lineno + 1, line, tag).unwrap();
                }
            }
            writeln!(out, "```\n").unwrap();
        }
    }
}

impl DiffFormatter for DiffCliFormatter {
    fn print_preamble(&self, out: &mut dyn Write) {
        if self.colored {
            writeln!(out, "{}\n", "📦 Cross-file Diff Report".bright_magenta().bold()).unwrap();
        } else {
            writeln!(out, "📦 Cross-file Diff Report\n").unwrap();
        }
    }

    fn print_index(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write) {
        writeln!(out, "📄 File Pairs").unwrap();
        for g in groups {
            if self.colored {
                writeln!(
                    out,
                    "{} {} {}",
                    g.from_file.truecolor(0, 255, 255),
                    "→".bright_magenta(),
                    g.to_file.truecolor(0, 255, 255)
                ).unwrap();
            } else {
                writeln!(out, "- {} → {}", g.from_file, g.to_file).unwrap();
            }
        }
        writeln!(out).unwrap();
    }

    fn print_contents(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write) {
        for g in groups {
            if self.colored {
                writeln!(
                    out,
                    "{} {} {}",
                    g.from_file.truecolor(0, 255, 255),
                    "→".bright_magenta(),
                    g.to_file.truecolor(0, 255, 255)
                ).unwrap();
            } else {
                writeln!(out, "{} → {}", g.from_file, g.to_file).unwrap();
            }

            if let Ok(src) = fs::read_to_string(&g.from_file) {
                let lines: Vec<&str> = src.lines().collect();
                for (lineno, line) in lines.iter().enumerate() {
                    let mut tag = String::new();

                    for bwv in &g.blocks {
                        let m = &bwv.block;
                        if lineno >= m.from_range.0 && lineno < m.from_range.1 {
                            tag = if bwv.is_addition {
                                format!(" [ADD] ({}–{} → {}–{})",
                                    m.from_range.0, m.from_range.1, m.to_file, m.to_range.0)
                            } else {
                                format!(" [MOVED] ({}–{} → {}–{})",
                                    m.from_range.0, m.from_range.1, m.to_file, m.to_range.0)
                            };
                            break;
                        }
                    }

                    if self.colored {
                        writeln!(
                            out,
                            "{:>4} {}{}",
                            lineno + 1,
                            line,
                            tag.bright_yellow()
                        ).unwrap();
                    } else {
                        writeln!(out, "{:>4} {}{}", lineno + 1, line, tag).unwrap();
                    }
                }
            }

            writeln!(out).unwrap();
        }
    }
}
