use std::io::Write;

use crate::types::{FileEntry, GroupedMatches};

pub trait OutputFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write);
    fn print_index(&self, files: &[FileEntry], out: &mut dyn Write);
    fn print_contents(&self, files: &[FileEntry], out: &mut dyn Write);
}

pub trait DiffFormatter {
    fn print_preamble(&self, out: &mut dyn Write);
    fn print_index(&self, groups: &[GroupedMatches], out: &mut dyn Write);
    fn print_contents(&self, groups: &[GroupedMatches], out: &mut dyn Write);
}
