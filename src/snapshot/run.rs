use crate::cli::Args;
use crate::scanner::collect_files;
use crate::snapshot::filelist::prepare_file_list;
use crate::snapshot::writer::{open_writer, OutputTarget};
use crate::snapshot::format_selection::select_formatter;
use crate::snapshot::split::split_files_by_tokens;

use std::fs::File;
use std::io::Write;

/// Inject FUR-style stats into the markdown buffer
fn finalize_markdown(buf: &[u8], out_path: &str, shard_idx: Option<(usize, usize)>) {
    let text = String::from_utf8_lossy(buf);

    let word_count = text.split_whitespace().count();
    let token_est = ((word_count as f32) * 1.33).round() as usize;

    let shard_line = shard_idx.map(|(i, total)| {
        format!("> 🔹 SHARD {} / {}\n", i, total)
    }).unwrap_or_default();

    let inject = format!(
        "{}> ✍️ Words: {}\n> 🪙 Tokens (est.): {}\n\n## INDEX",
        shard_line,
        word_count,
        token_est
    );

    let final_text = text.replacen("## INDEX", &inject, 1);

    let mut file = File::create(out_path)
        .expect("Failed to write final markdown file");
    file.write_all(final_text.as_bytes()).unwrap();
}


/// Run the project snapshot (default command)
pub fn run_snapshot(mut args: Args) {

    //
    // ============================================================
    // 1. HANDLE --whited (legacy) AND --printed (new)
    // ============================================================
    //

    // Legacy: --whited
    if let Some(opt) = &args.whited {
        args.contents = true;

        args.out = Some(match opt {
            Some(name) => name.clone(),
            None => "SHOW.md".to_string(),
        });

        if args.white.is_none() {
            args.white = Some(None); // triggers stdin pattern prompt
        }
    }

    // New: --printed
    if let Some(opt) = &args.printed {
        args.contents = true;

        args.out = Some(match opt {
            Some(name) => name.clone(),
            None => "SHOW.md".to_string(),
        });
    }

    //
    // ============================================================
    // 2. Run scan
    // ============================================================
    //

    let root = args.dir.clone();
    let mut writer = open_writer(&args);

    let files = collect_files(&args);
    let prepared = prepare_file_list(files);

    let fmt = select_formatter(&args);

    //
    // ============================================================
    // 3. Render snapshot
    // ============================================================
    //

    match &mut writer {
        //
        //  A) Markdown to file → buffer → inject → write at end
        //
        OutputTarget::Memory(buf) => {

            let split_k = args.split.as_ref()
                .map(|opt| opt.unwrap_or(32))
                .unwrap_or(0);

            if split_k > 0 {
                let target_tokens = split_k * 1000;
                let packets = split_files_by_tokens(prepared, target_tokens);

                let base = args.out.as_ref().unwrap().trim_end_matches(".md");

                
                let total = packets.len();

                for (i, packet) in packets.iter().enumerate() {
                    let mut local_buf = Vec::new();

                    fmt.print_preamble(&root, &mut local_buf);
                    fmt.print_index(packet, &mut local_buf);

                    if args.contents {
                        fmt.print_contents(packet, &mut local_buf);
                    }

                    let out_path = format!("{}.shard{:02}.md", base, i + 1);

                    finalize_markdown(&local_buf, &out_path, Some((i + 1, total)));
                }
            } else {
                // original behavior
                fmt.print_preamble(&root, buf);
                fmt.print_index(&prepared, buf);

                if args.contents {
                    fmt.print_contents(&prepared, buf);
                }

                let out_path = args.out.as_ref().unwrap();
                finalize_markdown(buf.as_slice(), out_path, None);                
            }
        }


        //
        //  B) stdout (no injection)
        //
        OutputTarget::Stdout => {
            let out = &mut std::io::stdout();

            fmt.print_preamble(&root, out);
            fmt.print_index(&prepared, out);

            if args.contents {
                fmt.print_contents(&prepared, out);
            }
        }

        //
        //  C) Should never occur
        //
        OutputTarget::File(_) => {
            panic!("OutputTarget::File should not be used; use Memory for markdown.");
        }
    }
}
