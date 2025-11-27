use std::io::Write;
use crate::types::FileEntry;
use crate::types::GroupedMatches;

pub trait OutputFormatter {
    fn print_preamble(&self, root: &str, out: &mut dyn Write);
    fn print_index(&self, files: &Vec<FileEntry>, out: &mut dyn Write);
    fn print_contents(&self, files: &Vec<FileEntry>, out: &mut dyn Write);
}

pub trait DiffFormatter {
    fn print_preamble(&self, out: &mut dyn Write);
    fn print_index(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write);
    fn print_contents(&self, groups: &Vec<GroupedMatches>, out: &mut dyn Write);
}

