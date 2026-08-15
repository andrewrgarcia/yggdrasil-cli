use std::io::Write;

use colored::{ColoredString, Colorize};

use super::node::TreeNode;

#[derive(Debug, Clone)]
pub struct RenderOpts {
    /// Levels below the root to expand. `Some(0)` = root's children only.
    /// `None` = unlimited.
    pub max_depth: Option<usize>,
    pub colored: bool,
    pub show_stats: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts {
            max_depth: None,
            colored: false,
            show_stats: true,
        }
    }
}

/// 1234 -> "1.2k", 1_500_000 -> "1.5M"
pub fn human_tokens(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Context cost, as a colour. This is the whole product thesis in four lines.
fn heat(tokens: usize, text: &str) -> ColoredString {
    if tokens >= 4_000 {
        text.truecolor(255, 95, 95)
    } else if tokens >= 1_000 {
        text.truecolor(255, 200, 50)
    } else if tokens >= 200 {
        text.truecolor(120, 220, 120)
    } else {
        text.truecolor(130, 130, 130)
    }
}

fn paint_name(node: &TreeNode, opts: &RenderOpts) -> String {
    let label = node.display_name();
    if !opts.colored {
        return label;
    }
    if node.is_dir {
        label.truecolor(0, 255, 255).bold().to_string()
    } else {
        label.normal().to_string()
    }
}

fn stats_cell(node: &TreeNode, opts: &RenderOpts) -> String {
    if !opts.show_stats {
        return String::new();
    }

    let tokens = human_tokens(node.stats.tokens);
    let painted = if opts.colored {
        heat(node.stats.tokens, &tokens).to_string()
    } else {
        tokens.clone()
    };
    // Pad on the plain string, colour after, so ANSI never skews the column.
    let pad = 7usize.saturating_sub(tokens.chars().count());
    let cell = format!("{}{} tok", " ".repeat(pad), painted);

    if node.is_dir {
        let tail = format!("  ({} files)", node.file_count);
        if opts.colored {
            format!("{}{}", cell, tail.truecolor(110, 110, 110))
        } else {
            format!("{}{}", cell, tail)
        }
    } else {
        cell
    }
}

fn descends(depth: usize, opts: &RenderOpts) -> bool {
    match opts.max_depth {
        Some(max) => depth < max,
        None => true,
    }
}

fn measure(node: &TreeNode, depth: usize, indent: usize, opts: &RenderOpts, max: &mut usize) {
    for child in &node.children {
        let width = indent + 4 + child.display_name().chars().count();
        *max = (*max).max(width);
        if child.is_dir && descends(depth, opts) {
            measure(child, depth + 1, indent + 4, opts, max);
        }
    }
}

fn walk(node: &TreeNode, depth: usize, prefix: &str, width: usize, opts: &RenderOpts, out: &mut dyn Write) {
    let total = node.children.len();

    for (i, child) in node.children.iter().enumerate() {
        let is_last = i + 1 == total;
        let connector = if is_last { "└── " } else { "├── " };

        let plain_width = prefix.chars().count() + 4 + child.display_name().chars().count();
        let pad = width.saturating_sub(plain_width) + 2;

        let stem = format!("{}{}", prefix, connector);
        let stem = if opts.colored {
            stem.truecolor(90, 90, 90).to_string()
        } else {
            stem
        };

        writeln!(
            out,
            "{}{}{}{}",
            stem,
            paint_name(child, opts),
            " ".repeat(pad),
            stats_cell(child, opts)
        )
        .unwrap();

        if child.is_dir && descends(depth, opts) {
            let next = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            walk(child, depth + 1, &next, width, opts, out);
        }
    }
}

/// Full box-drawn tree, rooted at `root`.
pub fn render_tree(root: &TreeNode, opts: &RenderOpts, out: &mut dyn Write) {
    let mut width = root.display_name().chars().count();
    measure(root, 0, 0, opts, &mut width);

    let header = root.display_name();
    if opts.colored {
        writeln!(out, "{}", header.truecolor(0, 255, 255).bold()).unwrap();
    } else {
        writeln!(out, "{}", header).unwrap();
    }

    walk(root, 0, "", width, opts, out);
    write_footer(root, opts, out);
}

/// Flat single-level listing — what bare `ygg` prints.
pub fn render_list(root: &TreeNode, opts: &RenderOpts, out: &mut dyn Write) {
    let width = root
        .children
        .iter()
        .map(|c| c.display_name().chars().count())
        .max()
        .unwrap_or(0);

    for child in &root.children {
        let icon = if child.is_dir { "📁" } else { "📄" };
        let pad = width.saturating_sub(child.display_name().chars().count()) + 2;

        writeln!(
            out,
            "{} {}{}{}",
            icon,
            paint_name(child, opts),
            " ".repeat(pad),
            stats_cell(child, opts)
        )
        .unwrap();
    }

    write_footer(root, opts, out);
}

fn write_footer(root: &TreeNode, opts: &RenderOpts, out: &mut dyn Write) {
    if !opts.show_stats {
        return;
    }

    let summary = format!(
        "{} dirs, {} files · {} lines · {} tokens",
        root.dir_count,
        root.file_count,
        root.stats.lines,
        human_tokens(root.stats.tokens)
    );

    writeln!(out).unwrap();
    if opts.colored {
        writeln!(out, "🌳 {}", summary.bright_magenta().bold()).unwrap();
    } else {
        writeln!(out, "🌳 {}", summary).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::{build_tree, RawEntry, Stats};
    use std::path::Path;

    fn entry(rel: &[&str], is_dir: bool, tokens: usize) -> RawEntry {
        RawEntry {
            rel: rel.iter().map(|s| s.to_string()).collect(),
            is_dir,
            stats: Stats {
                lines: 1,
                words: tokens,
                tokens,
            },
        }
    }

    fn sample() -> TreeNode {
        build_tree(
            "proj".into(),
            Path::new("."),
            vec![
                entry(&["src"], true, 0),
                entry(&["src", "deep"], true, 0),
                entry(&["src", "deep", "buried.rs"], false, 300),
                entry(&["src", "main.rs"], false, 500),
                entry(&["README.md"], false, 100),
            ],
        )
    }

    fn render(opts: RenderOpts) -> String {
        let mut buf = Vec::new();
        render_tree(&sample(), &opts, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn draws_box_characters() {
        let out = render(RenderOpts {
            colored: false,
            ..Default::default()
        });
        assert!(out.contains("├── src/"));
        assert!(out.contains("└── README.md"));
        assert!(out.contains("│   "));
    }

    #[test]
    fn depth_zero_shows_only_root_children() {
        let out = render(RenderOpts {
            max_depth: Some(0),
            colored: false,
            ..Default::default()
        });
        assert!(out.contains("src/"));
        assert!(out.contains("README.md"));
        assert!(!out.contains("main.rs"));
        assert!(!out.contains("buried.rs"));
    }

    #[test]
    fn depth_one_shows_grandchildren_but_not_great() {
        let out = render(RenderOpts {
            max_depth: Some(1),
            colored: false,
            ..Default::default()
        });
        assert!(out.contains("main.rs"));
        assert!(out.contains("deep/"));
        assert!(!out.contains("buried.rs"));
    }

    #[test]
    fn collapsed_directory_still_reports_full_subtree_cost() {
        // src/ is collapsed at depth 0 but must still show 500 + 300 = 800.
        let out = render(RenderOpts {
            max_depth: Some(0),
            colored: false,
            ..Default::default()
        });
        let src_line = out.lines().find(|l| l.contains("src/")).unwrap();
        assert!(src_line.contains("800 tok"), "got: {}", src_line);
        assert!(src_line.contains("(2 files)"), "got: {}", src_line);
    }

    #[test]
    fn humanizes_large_counts() {
        assert_eq!(human_tokens(999), "999");
        assert_eq!(human_tokens(1_718), "1.7k");
        assert_eq!(human_tokens(2_400_000), "2.4M");
    }
}