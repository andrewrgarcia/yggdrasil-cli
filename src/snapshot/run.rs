use crate::cli::Args;
use crate::scanner::collect_files;
use crate::snapshot::filelist::prepare_file_list;
use crate::snapshot::writer::{open_writer, OutputTarget};
use crate::snapshot::format_selection::select_formatter;
use crate::snapshot::split::split_files_by_tokens;
use crate::sniff::sniff_forward_paths;

use std::fs::File;
use std::io::Write;
use atty;

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


/// Emit a Yggdrasil-flavoured sniff header block into any writer.
fn write_sniff_header(
    entry: &str,
    paths: &[String],
    is_markdown: bool,
    colored: bool,
    out: &mut dyn std::io::Write,
) {
    if is_markdown {
        writeln!(out, "<!-- sniff: roots traced from {} -->", entry).unwrap();
        writeln!(out, "> 🐺 **Yggdrasil Sniff** — branches traced from `{}`", entry).unwrap();
        writeln!(out, ">").unwrap();
        writeln!(out, "> The world-tree read the runes of `{}`,", entry).unwrap();
        writeln!(out, "> and followed {} branch{} to their roots.",
            paths.len(),
            if paths.len() == 1 { "" } else { "es" }
        ).unwrap();
        writeln!(out, ">").unwrap();
        for p in paths {
            writeln!(out, "> - `{}`", p).unwrap();
        }
        writeln!(out).unwrap();
    } else {
        use colored::Colorize;
        let sep  = "━".repeat(54);
        let sep2 = "─".repeat(54);
        if colored {
            writeln!(out, "{}", sep.truecolor(255,200,50)).unwrap();
            writeln!(out, "{}  {}",
                "🐺".truecolor(100,220,100),
                "YGGDRASIL SNIFF".bright_magenta().bold()
            ).unwrap();
            writeln!(out, "The world-tree traced the runes of").unwrap();
            writeln!(out, "  {}",
                entry.truecolor(0,255,255).bold()
            ).unwrap();
            writeln!(out, "and followed {} branch{} to their roots:",
                paths.len().to_string().bright_magenta().bold(),
                if paths.len() == 1 { "" } else { "es" }
            ).unwrap();
            writeln!(out, "{}", sep2.truecolor(255,200,50)).unwrap();
            for p in paths {
                writeln!(out, "  {} {}",
                    "⎇".truecolor(255,200,50),
                    p.truecolor(0,255,255)
                ).unwrap();
            }
            writeln!(out, "{}", sep.truecolor(255,200,50)).unwrap();
        } else {
            writeln!(out, "{}", sep).unwrap();
            writeln!(out, "🐺  YGGDRASIL SNIFF").unwrap();
            writeln!(out, "The world-tree traced the runes of").unwrap();
            writeln!(out, "  {}", entry).unwrap();
            writeln!(out, "and followed {} branch{} to their roots:",
                paths.len(),
                if paths.len() == 1 { "" } else { "es" }
            ).unwrap();
            writeln!(out, "{}", sep2).unwrap();
            for p in paths {
                writeln!(out, "  ⎇ {}", p).unwrap();
            }
            writeln!(out, "{}", sep).unwrap();
        }
        writeln!(out).unwrap();
    }
}

/// Run the project snapshot (default command)
pub fn run_snapshot(mut args: Args) {

    //
    // ============================================================
    // 0. HANDLE --sniff
    //    Expand entry file → reachable local deps → populate args.only
    //    Must run before all other flag handling so downstream logic
    //    (--ignore, --show, --split, etc.) applies normally.
    // ============================================================
    //

    // sniff_meta: Some((entry_file, discovered_paths)) when --sniff was used.
    // Stored here so we can emit a header block into the snapshot in step 3.
    let sniff_meta: Option<(String, Vec<String>)> = if let Some(ref target) = args.sniff.clone() {
        let discovered = sniff_forward_paths(target, &args.dir);

        if discovered.is_empty() {
            eprintln!(
                "⚠️  Yggdrasil could not trace the branches of '{}'\nunder root '{}'. Verify paths and --dir.",
                target, args.dir
            );
        } else {
            eprintln!("🌿 {} branches traced from the root.", discovered.len());
        }

        // Merge with any explicit --only the user also passed.
        // sniff paths go first so they appear before any manual additions.
        let mut merged = discovered.clone();
        merged.extend(args.only.drain(..));

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        args.only = merged
            .into_iter()
            .filter(|p| seen.insert(p.clone()))
            .collect();

        if discovered.is_empty() { None } else { Some((target.clone(), discovered)) }
    } else {
        None
    };

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

                    if let Some((ref entry, ref paths)) = sniff_meta {
                        write_sniff_header(entry, paths, true, false, &mut local_buf);
                    }
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
                if let Some((ref entry, ref paths)) = sniff_meta {
                    write_sniff_header(entry, paths, true, false, buf);
                }
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

            if let Some((ref entry, ref paths)) = sniff_meta {
                let use_color = atty::is(atty::Stream::Stdout);
                write_sniff_header(entry, paths, false, use_color, out);
            }
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