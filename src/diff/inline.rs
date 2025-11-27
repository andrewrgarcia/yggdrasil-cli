use similar::{TextDiff, ChangeTag};

/// Print a single-file unified inline diff.
pub fn diff_file_contents(file: &str, from: &str, to: &str) {
    println!("\n📄 Diff for {file}:\n");

    let diff = TextDiff::from_lines(from, to);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };

        let colored = match change.tag() {
            ChangeTag::Delete => format!("\x1b[91m{}{}\x1b[0m", sign, change),
            ChangeTag::Insert => format!("\x1b[92m{}{}\x1b[0m", sign, change),
            ChangeTag::Equal => format!(" {}{}", sign, change),
        };

        print!("{}", colored);
    }

    println!();
}

