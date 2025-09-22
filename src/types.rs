// src/types.rs
#[derive(Debug, Clone)]
pub struct BlockMatch {
    pub from_file: String,
    pub from_range: (usize, usize),
    pub to_file: String,
    pub to_range: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct BlockWithVote {
    pub block: BlockMatch,
    pub is_addition: bool,
}

#[derive(Debug, Clone)]
pub struct GroupedMatches {
    pub from_file: String,
    pub to_file: String,
    pub blocks: Vec<BlockWithVote>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub line_count: usize,
}
