use colored::*;

/// Trait all formatters must implement
pub trait OutputFormatter {
    fn print_preamble(&self, root: &str);
    fn print_index(&self, files: &Vec<String>);
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
    }
}

/// Colorful CLI-style formatter
pub struct CliFormatter;

impl OutputFormatter for CliFormatter {
    fn print_preamble(&self, root: &str) {
        let title = "📂 Listing directory:".bright_cyan().bold();
        let path = root.green();
        println!("{} {}", title, path);
    }

    fn print_index(&self, files: &Vec<String>) {
        for file in files {
            let icon = "📄".yellow();
            println!("{} {}", icon, file.blue());
        }
    }
}
