use crate::cli::Args;
use crate::scanner::collect_files;
use crate::snapshot::archive::{archive_path_for, write_zip_archive};
use crate::snapshot::filelist::prepare_file_list;
use crate::snapshot::format_selection::select_formatter;
use crate::snapshot::split::split_files_by_tokens;
use crate::snapshot::writer::{open_writer, OutputTarget};
use crate::sniff::sniff_forward_paths;

use atty;
use std::fs;
use std::path::Path;

/// Inject FUR-style stats into the Markdown buffer.
fn finalize_markdown(buf: &[u8], shard_idx: Option<(usize, usize)>) -> Vec<u8> {
    let text = String::from_utf8_lossy(buf);

    let word_count = text.split_whitespace().count();
    let token_est = ((word_count as f32) * 1.33).round() as usize;

    let shard_line = shard_idx
        .map(|(i, total)| format!("> 🔹 SHARD {} / {}\n", i, total))
        .unwrap_or_default();

    let inject = format!(
        "{}> ✍️ Words: {}\n> 🪙 Tokens (est.): {}\n\n## INDEX",
        shard_line, word_count, token_est
    );

    text.replacen("## INDEX", &inject, 1).into_bytes()
}

/// Return only the file name used inside the ZIP archive.
fn archive_entry_name(output_path: &str) -> String {
    Path::new(output_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(output_path)
        .to_string()
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
        writeln!(
            out,
            "> 🐺 **Yggdrasil Sniff** — branches traced from `{}`",
            entry
        )
        .unwrap();
        writeln!(out, ">").unwrap();
        writeln!(out, "> The world-tree read the runes of `{}`,", entry).unwrap();
        writeln!(
            out,
            "> and followed {} branch{} to their roots.",
            paths.len(),
            if paths.len() == 1 { "" } else { "es" }
        )
        .unwrap();
        writeln!(out, ">").unwrap();
        for p in paths {
            writeln!(out, "> - `{}`", p).unwrap();
        }
        writeln!(out).unwrap();
    } else {
        use colored::Colorize;

        let sep = "━".repeat(54);
        let sep2 = "─".repeat(54);

        if colored {
            writeln!(out, "{}", sep.truecolor(255, 200, 50)).unwrap();
            writeln!(
                out,
                "{}  {}",
                "🐺".truecolor(100, 220, 100),
                "YGGDRASIL SNIFF".bright_magenta().bold()
            )
            .unwrap();
            writeln!(out, "The world-tree traced the runes of").unwrap();
            writeln!(out, "  {}", entry.truecolor(0, 255, 255).bold()).unwrap();
            writeln!(
                out,
                "and followed {} branch{} to their roots:",
                paths.len().to_string().bright_magenta().bold(),
                if paths.len() == 1 { "" } else { "es" }
            )
            .unwrap();
            writeln!(out, "{}", sep2.truecolor(255, 200, 50)).unwrap();

            for p in paths {
                writeln!(
                    out,
                    "  {} {}",
                    "⎇".truecolor(255, 200, 50),
                    p.truecolor(0, 255, 255)
                )
                .unwrap();
            }

            writeln!(out, "{}", sep.truecolor(255, 200, 50)).unwrap();
        } else {
            writeln!(out, "{}", sep).unwrap();
            writeln!(out, "🐺  YGGDRASIL SNIFF").unwrap();
            writeln!(out, "The world-tree traced the runes of").unwrap();
            writeln!(out, "  {}", entry).unwrap();
            writeln!(
                out,
                "and followed {} branch{} to their roots:",
                paths.len(),
                if paths.len() == 1 { "" } else { "es" }
            )
            .unwrap();
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
            eprintln!("⚠️ could not trace '{}'\nunder '{}'", target, args.dir);
        } else {
            eprintln!("🌿 {} branches traced", discovered.len());
        }

        let mut merged = discovered.clone();
        merged.extend(args.only.drain(..));

        let mut seen = std::collections::HashSet::new();
        args.only = merged
            .into_iter()
            .filter(|p| seen.insert(p.clone()))
            .collect();

        if discovered.is_empty() {
            None
        } else {
            Some((target.to_string(), discovered))
        }
    } else {
        None
    };

    // ============================================================
    // 1. HANDLE SHORTCUT FLAGS
    // ============================================================

    // --treed = interactive selection + markdown output + index only
    if let Some(opt) = &args.treed {
        args.contents = false;
        args.out = Some(match opt {
            Some(name) => name.clone(),
            None => "SHOW.md".to_string(),
        });

        if args.white.is_none() {
            args.white = Some(None);
        }
    }

    // legacy full codex interactive shortcut
    if let Some(opt) = &args.whited {
        args.contents = true;
        args.out = Some(match opt {
            Some(name) => name.clone(),
            None => "SHOW.md".to_string(),
        });

        if args.white.is_none() {
            args.white = Some(None);
        }
    }

    // --printed stays full codex unless contents was intentionally left false
    if let Some(opt) = &args.printed {
        args.contents = true;
        args.out = Some(match opt {
            Some(name) => name.clone(),
            None => "SHOW.md".to_string(),
        });
    }

    // If user explicitly asked for treed, force index-only after printed/whited handling
    if args.treed.is_some() {
        args.contents = false;
    }

    // ============================================================
    // 2. RUN SCAN
    // ============================================================

    let root = args.dir.clone();
    let mut writer = open_writer(&args);

    let files = collect_files(&args);
    let prepared = prepare_file_list(files);

    let fmt = select_formatter(&args);

    // ============================================================
    // 3. RENDER
    // ============================================================

    match &mut writer {
        OutputTarget::Memory(buf) => {
            let split_k = args
                .split
                .as_ref()
                .map(|opt| opt.unwrap_or(32))
                .unwrap_or(0);

            if split_k > 0 {
                let target_tokens = split_k * 1000;
                let packets = split_files_by_tokens(prepared, target_tokens);
                let output_path = args.out.as_ref().unwrap();
                let base = output_path.trim_end_matches(".md");
                let total = packets.len();
                let mut archive_entries = Vec::new();

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

                    let shard_path = format!("{}.shard{:02}.md", base, i + 1);
                    let finalized =
                        finalize_markdown(&local_buf, Some((i + 1, total)));

                    if args.zip {
                        archive_entries.push((
                            archive_entry_name(&shard_path),
                            finalized,
                        ));
                    } else {
                        fs::write(&shard_path, finalized)
                            .expect("Failed to write final Markdown shard");
                    }
                }

                if args.zip {
                    let archive_path = archive_path_for(output_path);

                    write_zip_archive(&archive_path, &archive_entries)
                        .expect("Failed to write ZIP archive");

                    eprintln!("📦 {}", archive_path.display());
                }
            } else {
                if let Some((ref entry, ref paths)) = sniff_meta {
                    write_sniff_header(entry, paths, true, false, buf);
                }

                fmt.print_preamble(&root, buf);
                fmt.print_index(&prepared, buf);

                if args.contents {
                    fmt.print_contents(&prepared, buf);
                }

                let output_path = args.out.as_ref().unwrap();
                let finalized = finalize_markdown(buf.as_slice(), None);

                if args.zip {
                    let archive_path = archive_path_for(output_path);
                    let entries = vec![(
                        archive_entry_name(output_path),
                        finalized,
                    )];

                    write_zip_archive(&archive_path, &entries)
                        .expect("Failed to write ZIP archive");

                    eprintln!("📦 {}", archive_path.display());
                } else {
                    fs::write(output_path, finalized)
                        .expect("Failed to write final Markdown file");
                }
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
            panic!("File target should not be used");
        }
    }
}
