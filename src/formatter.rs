use colored::*;
use std::fs;

/// Trait all formatters must implement
pub trait OutputFormatter {
    fn print_preamble(&self, root: &str);
    fn print_index(&self, files: &Vec<String>);
    fn print_contents(&self, files: &Vec<String>);
}

/// Markdown-style formatter
pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
    fn print_preamble(&self, root: &str) {
        println!("# 📂 Project Listing: {}", root);
        println!("\n## 📄 Files");
    }

    fn print_index(&self, files: &Vec<String>) {
        for file in files {
            println!("- {}", file);
        }
        println!("\n---\n\n## 📑 File Contents\n");
    }

    fn print_contents(&self, files: &Vec<String>) {
        for file in files {
            println!("### {}", file);
            println!("```");
            if let Ok(content) = fs::read_to_string(file) {
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
        let title = "📂 Listing directory:".bright_cyan().bold();
        let path = root.truecolor(255, 100, 0).bold(); // vivid orange
        println!("{} {}", title, path);
    }

    fn print_index(&self, files: &Vec<String>) {
        for file in files {
            let icon = "📄".truecolor(255, 20, 147); // neon pink
            let text = file.truecolor(0, 255, 255); // neon cyan
            println!("{} {}", icon, text);
        }
        println!("\n{}", "===============================================".truecolor(255, 100, 0));
    }

    fn print_contents(&self, files: &Vec<String>) {
        for file in files {
            println!(
                "{} {}",
                "<<< FILE START:".bright_magenta().bold(),
                file.truecolor(0, 255, 255)
            );

            if let Ok(content) = fs::read_to_string(file) {
                print!("{}", content);
            } else {
                println!("❌ Error reading file");
            }

            println!(
                "{} {}",
                "<<< FILE END:".bright_magenta().bold(),
                file.truecolor(0, 255, 255)
            );
            println!();
        }
    }
}
