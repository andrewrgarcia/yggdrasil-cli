//! Picker UI — the raw-mode terminal loop.
//!
//! All decisions about *what* is visible or selected live in `state.rs`;
//! this file only draws rows and translates key/mouse events into state
//! mutations. Keeping that seam clean is what lets the state be tested
//! without a tty.

use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use super::state::{Mark, PickState, Row};

/// First list row on screen: one header line, one blank line above the tree.
const LIST_TOP: u16 = 2;
/// Rows reserved below the list: blank + status + keys.
const FOOTER_ROWS: u16 = 3;

/// Same thresholds as `tree::render::heat` — context cost, as a colour.
fn heat_color(tokens: usize) -> Color {
    if tokens >= 4_000 {
        Color::Rgb { r: 255, g: 95, b: 95 }
    } else if tokens >= 1_000 {
        Color::Rgb { r: 255, g: 200, b: 50 }
    } else if tokens >= 200 {
        Color::Rgb { r: 120, g: 220, b: 120 }
    } else {
        Color::Rgb { r: 130, g: 130, b: 130 }
    }
}

/// Same humanizer as `tree::render::human_tokens`.
fn human_tokens(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Restores the terminal even if the loop panics — a picker that leaves the
/// shell in raw mode is worse than no picker.
struct TermGuard;

impl TermGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            Hide
        )?;
        Ok(TermGuard)
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

/// What the user asked for on the way out.
pub enum Outcome {
    /// Left without producing anything.
    Quit,
    /// `w` — write the codex to a file.
    Write(Vec<String>),
    /// `c` — render the codex and put it on the clipboard, no file.
    Copy(Vec<String>),
    /// `z` — pack the selected files, at their real paths, under a README
    /// that is the codex index.
    Zip(Vec<String>),
}

/// Run the interactive picker over an already-scanned tree.
///
/// Paths come back in the same form `collect_files` emits, so they can be fed
/// straight into `args.only`.
pub fn run_picker(state: &mut PickState) -> io::Result<Outcome> {
    let _guard = TermGuard::enter()?;
    let mut out = io::stdout();

    // A one-shot message shown in place of the key hints — cleared by the
    // next keypress, so it never goes stale.
    let mut flash: Option<String> = None;

    loop {
        let rows = state.rows();
        if state.cursor >= rows.len() && !rows.is_empty() {
            state.cursor = rows.len() - 1;
        }

        draw(state, &rows, flash.as_deref(), &mut out)?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                flash = None;
                let verdict = on_key(state, &rows, key);

                match verdict {
                    Verdict::Continue => {}
                    Verdict::Quit => return Ok(Outcome::Quit),
                    Verdict::Write | Verdict::Copy | Verdict::Zip => {
                        let paths = state.selection_paths();

                        // Nothing selected: an empty codex would be a
                        // confusing no-op. Say so — silently ignoring the
                        // key is indistinguishable from a broken key.
                        if paths.is_empty() {
                            flash = Some(
                                "nothing selected — press space to tick a file first"
                                    .to_string(),
                            );
                            continue;
                        }

                        return Ok(match verdict {
                            Verdict::Copy => Outcome::Copy(paths),
                            Verdict::Zip => Outcome::Zip(paths),
                            _ => Outcome::Write(paths),
                        });
                    }
                }
            }
            Event::Mouse(m) => {
                flash = None;
                on_mouse(state, &rows, m);
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

enum Verdict {
    Continue,
    Quit,
    Write,
    Copy,
    Zip,
}

fn on_key(state: &mut PickState, rows: &[Row], key: KeyEvent) -> Verdict {
    let cur = rows.get(state.cursor).cloned();

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Verdict::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Verdict::Quit
        }
        KeyCode::Char('w') => return Verdict::Write,
        KeyCode::Char('c') => return Verdict::Copy,
        KeyCode::Char('z') => return Verdict::Zip,

        KeyCode::Up | KeyCode::Char('k') => {
            state.cursor = state.cursor.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.cursor + 1 < rows.len() {
                state.cursor += 1;
            }
        }

        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(row) = &cur {
                if row.is_dir {
                    state.set_expanded(&row.path, true);
                }
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(row) = &cur {
                if row.is_dir && row.expanded {
                    state.set_expanded(&row.path, false);
                } else if let Some(parent) = row.path.parent() {
                    // Jump to the parent row instead of doing nothing.
                    let parent = parent.to_path_buf();
                    if let Some(i) = rows.iter().position(|r| r.path == parent) {
                        state.cursor = i;
                    }
                }
            }
        }

        KeyCode::Enter => {
            if let Some(row) = &cur {
                if row.is_dir {
                    state.toggle_expanded(&row.path);
                } else {
                    state.toggle(&row.path);
                }
            }
        }
        KeyCode::Char(' ') => {
            if let Some(row) = &cur {
                state.toggle(&row.path);
            }
        }
        KeyCode::Char('a') => state.toggle_all(),
        KeyCode::Char('*') => state.expand_all(),
        _ => {}
    }

    Verdict::Continue
}

fn on_mouse(state: &mut PickState, rows: &[Row], m: MouseEvent) {
    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let Some(idx) = (m.row >= LIST_TOP)
                .then(|| state.scroll + (m.row - LIST_TOP) as usize)
                .filter(|i| *i < rows.len())
            else {
                return;
            };
            state.cursor = idx;
            let row = &rows[idx];
            // Click semantics mirror Enter: dirs open, files select.
            if row.is_dir {
                state.toggle_expanded(&row.path);
            } else {
                state.toggle(&row.path);
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            let Some(idx) = (m.row >= LIST_TOP)
                .then(|| state.scroll + (m.row - LIST_TOP) as usize)
                .filter(|i| *i < rows.len())
            else {
                return;
            };
            state.cursor = idx;
            // Right-click always toggles selection — subtree for dirs.
            state.toggle(&rows[idx].path);
        }
        MouseEventKind::ScrollUp => state.cursor = state.cursor.saturating_sub(1),
        MouseEventKind::ScrollDown => {
            if state.cursor + 1 < rows.len() {
                state.cursor += 1;
            }
        }
        _ => {}
    }
}

fn draw(
    state: &mut PickState,
    rows: &[Row],
    flash: Option<&str>,
    out: &mut impl Write,
) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    let list_height = height.saturating_sub(LIST_TOP + FOOTER_ROWS).max(1) as usize;

    // Keep the cursor inside the viewport.
    if state.cursor < state.scroll {
        state.scroll = state.cursor;
    } else if state.cursor >= state.scroll + list_height {
        state.scroll = state.cursor + 1 - list_height;
    }

    // One frame, one write. Painting straight to stdout across many small
    // queued calls lets the terminal present a half-drawn tree; building the
    // frame here and flushing it once is what removes the flicker.
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    let out_final = out;
    let out = &mut buf;

    queue!(out, MoveTo(0, 0), Clear(ClearType::UntilNewLine))?;

    // ── header ────────────────────────────────────────────────────────
    let root_name = state.root.display_name();
    queue!(
        out,
        SetForegroundColor(Color::Magenta),
        SetAttribute(Attribute::Bold),
        Print("✨ ygg pick"),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("  "),
        SetForegroundColor(Color::Rgb { r: 0, g: 255, b: 255 }),
        SetAttribute(Attribute::Bold),
        Print(&root_name),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print(format!(
            "  {} files · {} tok",
            state.root.file_count,
            human_tokens(state.root.stats.tokens)
        )),
    )?;

    // ── rows ──────────────────────────────────────────────────────────
    let name_width = rows
        .iter()
        .map(|r| r.prefix.chars().count() + r.name.chars().count() + 2)
        .max()
        .unwrap_or(0);

    for (line, idx) in (state.scroll..rows.len().min(state.scroll + list_height)).enumerate() {
        let row = &rows[idx];
        let y = LIST_TOP + line as u16;
        // Clear only the line about to be rewritten — never the whole screen.
        queue!(out, MoveTo(0, y), Clear(ClearType::UntilNewLine))?;

        let is_cursor = idx == state.cursor;

        // checkbox
        let (box_str, box_color) = match row.mark {
            Mark::All => ("[x]", Color::Rgb { r: 120, g: 220, b: 120 }),
            Mark::Partial => ("[~]", Color::Rgb { r: 255, g: 200, b: 50 }),
            Mark::None => ("[ ]", Color::Rgb { r: 110, g: 110, b: 110 }),
        };
        queue!(
            out,
            Print(if is_cursor { "▶ " } else { "  " }),
            SetForegroundColor(box_color),
            Print(box_str),
            ResetColor,
            Print(" ")
        )?;

        // stems
        queue!(
            out,
            SetForegroundColor(Color::Rgb { r: 90, g: 90, b: 90 }),
            Print(&row.prefix),
            ResetColor
        )?;

        // fold marker + name
        if row.is_dir {
            queue!(
                out,
                SetForegroundColor(Color::Rgb { r: 0, g: 255, b: 255 }),
                SetAttribute(Attribute::Bold),
                Print(if row.expanded { "▾ " } else { "▸ " }),
                Print(&row.name),
                SetAttribute(Attribute::Reset),
                ResetColor
            )?;
        } else {
            if is_cursor {
                queue!(out, SetAttribute(Attribute::Bold))?;
            }
            queue!(out, Print("  "), Print(&row.name))?;
            if is_cursor {
                queue!(out, SetAttribute(Attribute::Reset))?;
            }
        }

        // token column, aligned past the longest name
        let used = row.prefix.chars().count() + row.name.chars().count() + 2;
        let pad = name_width.saturating_sub(used) + 2;
        let tok = human_tokens(row.tokens);
        let cell_pad = 7usize.saturating_sub(tok.chars().count());

        if (used + pad + 12) < width as usize {
            queue!(
                out,
                Print(" ".repeat(pad + cell_pad)),
                SetForegroundColor(heat_color(row.tokens)),
                Print(&tok),
                ResetColor,
                Print(" tok")
            )?;
            if row.is_dir {
                queue!(
                    out,
                    SetForegroundColor(Color::Rgb { r: 110, g: 110, b: 110 }),
                    Print(format!("  ({} files)", row.file_count)),
                    ResetColor
                )?;
            }
        }
    }

    // ── footer ────────────────────────────────────────────────────────
    let (sel_count, sel_tokens) = state.selection_cost();
    queue!(
        out,
        MoveTo(0, height.saturating_sub(2)),
        Clear(ClearType::UntilNewLine),
        SetAttribute(Attribute::Bold),
        Print("🌳 "),
        SetForegroundColor(heat_color(sel_tokens)),
        Print(format!(
            "{} selected · {} tok",
            sel_count,
            human_tokens(sel_tokens)
        )),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("  → SHOW.md"),
        MoveTo(0, height.saturating_sub(1)),
        Clear(ClearType::UntilNewLine)
    )?;

    // The flash takes the hint line when present — same row, louder colour.
    match flash {
        Some(msg) => queue!(
            out,
            SetForegroundColor(Color::Rgb { r: 255, g: 200, b: 50 }),
            SetAttribute(Attribute::Bold),
            Print(format!("⚠  {msg}")),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?,
        None => queue!(
            out,
            SetForegroundColor(Color::Rgb { r: 130, g: 130, b: 130 }),
            Print("space/click select · enter/→ open · ← close · * open all · a all · c copy · w write · z zip · q quit"),
            ResetColor
        )?,
    }

    // Rows the list no longer fills (tree shrank after a collapse) would
    // otherwise keep their stale text, since nothing clears the screen now.
    let drawn = rows.len().saturating_sub(state.scroll).min(list_height);
    for line in drawn..list_height {
        queue!(
            out,
            MoveTo(0, LIST_TOP + line as u16),
            Clear(ClearType::UntilNewLine)
        )?;
    }

    out_final.write_all(&buf)?;
    out_final.flush()
}