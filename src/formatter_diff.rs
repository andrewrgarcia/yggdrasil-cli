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
    pub align_tags: bool,   // 🔹 NEW toggle
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

                // find longest line for alignment
                let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);

                for (lineno, line) in lines.iter().enumerate() {
                    let mut tag = String::new();

                    for bwv in &g.blocks {
                        let m = &bwv.block;
                        // ✅ read is_addition so compiler doesn’t complain
                        let status = if bwv.is_addition { "[ADDED]" } else { "[MOVED]" };

                        if lineno >= m.from_range.0 && lineno < m.from_range.1 {
                            tag = format!(
                                " // {} ({}–{} → {}–{})",
                                status,
                                m.from_range.0, m.from_range.1,
                                m.to_file, m.to_range.0
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
                let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);

                for (lineno, line) in lines.iter().enumerate() {
                    let mut tag = String::new();

                    for bwv in &g.blocks {
                        let m = &bwv.block;
                        // ✅ read is_addition so compiler doesn’t complain
                        let status = if bwv.is_addition { "[ADDED]" } else { "[MOVED]" };

                        if lineno >= m.from_range.0 && lineno < m.from_range.1 {
                            tag = format!(
                                " {} ({}–{} → {}–{})",
                                status,
                                m.from_range.0, m.from_range.1,
                                m.to_file, m.to_range.0
                            );
                            break;
                        }
                    }



                    if self.align_tags {
                        // pad code so tags align
                        if self.colored {
                            writeln!(
                                out,
                                "{:>4} {:<width$}{}",
                                lineno + 1,
                                line,
                                tag.bright_yellow(),
                                width = max_len + 1
                            ).unwrap();
                        } else {
                            writeln!(
                                out,
                                "{:>4} {:<width$}{}",
                                lineno + 1,
                                line,
                                tag,
                                width = max_len + 1
                            ).unwrap();
                        }
                    } else {
                        // original inline style
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
            }

            writeln!(out).unwrap();
        }
    }
}
