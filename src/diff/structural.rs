/// Detect a "structural boundary" — lines that must terminate a block.
pub fn is_boundary(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty()
        || trimmed.starts_with("class ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("@")
}

/// Extend a block using structural consistency.
pub fn extend_structural_block(
    from_lines: &[&str],
    to_lines: &[&str],
    from_start: usize,
    to_start: usize,
) -> (usize, usize, usize, usize) {
    let mut f1 = from_start;
    let mut t1 = to_start;
    let mut f2 = from_start;
    let mut t2 = to_start;

    // Extend forward
    while f2 < from_lines.len() && t2 < to_lines.len() && from_lines[f2] == to_lines[t2] {
        if is_boundary(from_lines[f2]) || is_boundary(to_lines[t2]) {
            break;
        }
        f2 += 1;
        t2 += 1;
    }

    // Extend backward
    while f1 > 0 && t1 > 0 && from_lines[f1 - 1] == to_lines[t1 - 1] {
        if is_boundary(from_lines[f1 - 1]) || is_boundary(to_lines[t1 - 1]) {
            break;
        }
        f1 -= 1;
        t1 -= 1;
    }

    (f1, f2, t1, t2)
}

