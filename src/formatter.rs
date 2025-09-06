use colored::*;
use std::fs;
use atty::Stream;

pub trait OutputFormatter {
    fn print_preamble(&self, root: &str);
    fn print_index(&self, files: &Vec<String>);
    fn print_contents(&self, files: &Vec<String>);
}

/// Markdown-style formatter
pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
    fn print_preamble(&self, root: &str) {
        println!("# ✨ Directory Codex: {}\n", root);
        println!("*Made with [Yggdrasil](https://crates.io/crates/yggdrasil)*  \n");
        println!("*This document contains two sections:*  ");
        println!("- **Files** → index of all paths.  ");
        println!("- **File Contents** → full text for each file under `### <path>`.  \n");
        println!("## 📄 Files");
    }

    fn print_index(&self, files: &Vec<String>) {
        for file in files {
            // Anchor = sanitize file path (replace / and . with -)
            let anchor = file.replace("/", "-").replace(".", "-");
            println!("- [{}](#{})", file, anchor);
        }
        println!("\n---\n\n## 📑 File Contents\n");
    }

    fn print_contents(&self, files: &Vec<String>) {
        for file in files {
            // Heading shows raw path for AI-readability
            println!("### <{}>", file);
            println!("```");
            if let Ok(content) = std::fs::read_to_string(file) {
                print!("{}", content);
            } else {
                println!("❌ Error reading file");
            }
            println!("```\n");
        }
    }
}

pub struct CliFormatter;


impl OutputFormatter for CliFormatter {
    fn print_preamble(&self, root: &str) {
        let title = "✨ Directory Codex:".bright_magenta().bold();
        let path = root.truecolor(0, 255, 255).bold();
        println!("{} {}", title, path);

        // If stdout is a terminal → clickable yellow link
        // Else → plain text + URL
        if atty::is(Stream::Stdout) {
            let link = format!(
                "\x1b]8;;https://crates.io/crates/yggdrasil-cli\x1b\\{}\x1b]8;;\x1b\\",
                "*Made with Yggdrasil*".truecolor(255, 255, 0).bold()
            );
            println!("{}", link);
        } else {
            println!(
                "*Made with Yggdrasil* <https://crates.io/crates/yggdrasil-cli>"
            );
        }

        let note = "\nSchema: index first, then file contents.\n\
        - Files are listed under '📄 Files'.\n\
        - Contents are shown with markers <<< FILE START: <path> >>> … <<< FILE END: <path> >>>\n";
        println!("{}", note.truecolor(150, 150, 150));
    }

    fn print_index(&self, files: &Vec<String>) {
        println!("{}", "📄 Files".bright_magenta().bold());
        for file in files {
            let icon = "📄".truecolor(255, 20, 147);
            let text = file.truecolor(0, 255, 255);
            println!("{} {}", icon, text);
        }
        println!("\n{}", "===============================================".truecolor(255, 255, 0));
        println!("{}", "📑 File Contents".bright_magenta().bold());
    }

    fn print_contents(&self, files: &Vec<String>) {
        for file in files {
            println!(
                "{} <{}> {}",
                "<<< FILE START:".bright_magenta().bold(),
                file,
                ">>>".bright_magenta().bold()
            );

            if let Ok(content) = fs::read_to_string(file) {
                print!("{}", content);
            } else {
                println!("❌ Error reading file");
            }

            println!(
                "{} <{}> {}",
                "<<< FILE END:".bright_magenta().bold(),
                file,
                ">>>".bright_magenta().bold()
            );
            println!();
        }
    }
}
