use crate::args::Args;
use crate::scanner::collect_files;
use crate::snapshot::filelist::prepare_file_list;
use crate::snapshot::writer::open_writer;
use crate::snapshot::format_selection::select_formatter;

pub fn run_snapshot(args: Args) {
    let root = args.dir.clone();
    let mut writer = open_writer(&args);

    let files = collect_files(&args);
    let prepared = prepare_file_list(files);

    let fmt = select_formatter(&args, &writer);

    fmt.print_preamble(&root, &mut *writer);
    fmt.print_index(&prepared, &mut *writer);

    if args.contents {
        fmt.print_contents(&prepared, &mut *writer);
    }
}

