use crate::cli::Args;
use crate::scanner::collect_files;
use crate::snapshot::filelist::prepare_file_list;
use crate::snapshot::writer::open_writer;
use crate::snapshot::format_selection::select_formatter;

/// Run the project snapshot (default command)
pub fn run_snapshot(mut args: Args) {

    //
    // ============================================================
    // 1. HANDLE --whited (legacy Yggdrasil behavior)
    // ============================================================
    //
    if let Some(opt) = &args.whited {
        // Enable contents
        args.contents = true;

        // Output file
        args.out = Some(match opt {
            Some(name) => name.clone(),
            None => "SHOW.md".to_string(),
        });

        // Force interactive white-pattern mode
        //
        // NOTE:
        //     collect_files() handles stdin prompting.
        //
        if args.white.is_none() {
            args.white = Some(None);
        }
    }

    //
    // ============================================================
    // 2. Run scan
    // ============================================================
    //

    let root = args.dir.clone();
    let mut writer = open_writer(&args);

    // collect_files() handles:
    //     - --white file
    //     - --white with stdin prompt (None)
    //     - --black equivalents
    let files = collect_files(&args);
    let prepared = prepare_file_list(files);

    let fmt = select_formatter(&args);

    //
    // ============================================================
    // 3. Render snapshot
    // ============================================================
    //

    fmt.print_preamble(&root, &mut *writer);
    fmt.print_index(&prepared, &mut *writer);

    if args.contents {
        fmt.print_contents(&prepared, &mut *writer);
    }
}
