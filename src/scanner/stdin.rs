use std::io::{self, Read};
use super::normalize::normalize_pattern;

pub fn read_multiline_stdin(prompt: &str) -> Option<Vec<String>> {
    use colored::Colorize;

    println!("{}", prompt.white());
    println!("{}", "💡 Tip: You can paste multiple file paths copied from VS Code (Right-Click → Copy Relative Path).".white());
    println!("{}", "↪ Finish with Ctrl+D (Linux/macOS) or Ctrl+Z then Enter (Windows).".white());
    println!("{}", "↪ Press Ctrl+C to cancel.".white());
    println!();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).ok()?;

    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        println!("⚠️ No content provided. Aborting.");
        None
    } else {
        Some(
            trimmed
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(normalize_pattern)
                .collect(),
        )
    }
}