use blake3;

/// Hash a single line (first 8 bytes of BLAKE3).
pub fn hash_line(line: &str) -> u64 {
    let h = blake3::hash(line.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&h.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

/// Hash a block of lines as a joined string.
pub fn hash_block(lines: &[&str]) -> u64 {
    let joined = lines.join("\n");
    let h = blake3::hash(joined.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&h.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

