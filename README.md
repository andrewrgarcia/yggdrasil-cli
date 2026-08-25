<!-- LOGO -->
<p align="center">
  <img width="200" alt="ygg" src="https://github.com/user-attachments/assets/5b59f4b6-10a0-4258-b0c3-06f21da08422" />
</p>

<h1 align="center"> Yggdrasil</h1>
<p align="center">
  <strong>The god-tree of your codebase</strong><br/>
Flatten any subset of your project into an AI-ready codex — index + contents, in one command.
</p>

---

# What is Yggdrasil?

Yggdrasil is a **project flattener and diff engine**.
It builds a single, deterministic codex from whatever subset of your codebase you choose:

* A full index of files
* Accurate line counts and token estimates
* Language-tagged code blocks
* Markdown or plain text output
* Optional rich diff mode
* Optional movement annotations (`[MOVED]`)

Use it for:

* LLM prompts
* Documentation snapshots
* Code reviews
* Reproducible archives
* Project comparisons

Yggdrasil does not guess what you want.
You explicitly choose the files — this makes your snapshot deterministic and deeply controllable.

---

# Quick Start

```bash
cargo install yggdrasil-cli
```

Then, in any project:

```bash
ygg                          # token-weighted listing of this directory
ygg tree                     # the full box-drawn tree
ygg pick                     # choose files interactively, copy or write
ygg --only src --printed     # flatten src/ into SHOW.md
```

If you only remember one command, make it `ygg pick`.

---

# Pick Mode — Interactive Selection

`ygg pick` opens your project as a navigable tree. Expand folders, tick the
files you want, and watch the token cost of your selection update live before
you commit to anything.

```bash
ygg pick
```

```
✨ ygg pick  yggdrasil-cli/  46 files · 22.1k tok

▶ [~] ├── ▾ src/                15.5k tok  (44 files)
  [x] │   ├── ▸ diff/            1.3k tok  (10 files)
  [ ] │   ├── ▾ formatters/      4.3k tok   (9 files)
  [x] │   │   ├──   symbols.rs   2.0k tok
  [ ] │   │   ├──   markdown.rs   690 tok
  [ ] │   └──   main.rs          1.0k tok
  [ ] └──   README.md            1.7k tok

🌳 12 selected · 4.6k tok  → SHOW.md
```

## Controls

| Action | Keys |
|---|---|
| Move | `↑` `↓` / `k` `j` / scroll wheel |
| Open folder | `→` / `l` / `Enter` / left-click |
| Close folder | `←` / `h` |
| Open everything | `*` |
| Select / deselect | `Space` / click a file / right-click a folder |
| Select all files | `a` |
| Copy codex to clipboard | `c` |
| Write codex to file | `w` |
| Leave without doing either | `q` / `Esc` / `Ctrl-C` |

Ticking a folder takes its whole subtree. Folder checkboxes are derived, not
stored: `[x]` means every file beneath it is selected, `[~]` means some are,
`[ ]` means none.

## The token footer

The running total at the bottom is the reason pick mode exists. It sums the
token estimate of everything currently selected and colours it on the same
heat scale as the tree — grey, green, amber, red. You can see a codex
approaching your model's context window while you build it, instead of finding
out after you paste.

## Copying instead of writing

Press `c` and the codex goes straight to your system clipboard. No file is
written, nothing to clean up later.

```
📋 12 files · 47 KB → clipboard (xclip)
```

Yggdrasil uses whichever native clipboard tool is available — `wl-copy`,
`xclip`, `xsel`, `pbcopy`, `clip.exe` — and falls back to an OSC 52 terminal
escape sequence when none is found.

**macOS and Windows need nothing installed** (`pbcopy` and `clip.exe` ship with
the OS). On Linux you may need one:

```bash
sudo apt install xclip          # X11
sudo apt install wl-clipboard   # Wayland
```

Without a tool, the OSC 52 fallback is attempted, but many terminals (GNOME
Terminal among them) silently ignore it and there is no way to detect this. In
that case Yggdrasil reports the codex as *sent* rather than *copied*, and
writes it to disk as a fallback so your selection is never lost.

## Flags

```bash
ygg pick                    # current directory → SHOW.md on `w`
ygg pick src/               # root the picker deeper
ygg pick --out CTX.md       # change the write target
ygg pick --all              # include dotfiles
ygg pick --no-ignore        # ignore .gitignore
```

---

# Tree Mode — Seeing What Things Cost

Bare `ygg` prints a flat, token-weighted listing of the current directory:

```bash
ygg
```

```
📁 src/            15.5k tok  (44 files)
📁 tests/            739 tok   (2 files)
📄 Cargo.lock       3.7k tok
📄 README.md        1.7k tok

🌳 12 dirs, 52 files · 6331 lines · 22.1k tokens
```

`ygg tree` gives the full box-drawn view. Every directory reports the total
token cost of its entire subtree, **even when collapsed** — so you can tell what
will fit in a context window before building anything.

```bash
ygg tree                   # whole tree
ygg tree src/              # rooted deeper
ygg tree -L 1              # expand one level below the root
ygg tree --dirs-only       # structure only
ygg tree --no-stats        # hide the token column
ygg tree -a                # include dotfiles
ygg tree --ignore target   # exclude paths or globs
```

## Symbols

`--symbols` (`-s`) shows what each file *declares* — the thing you actually
want when deciding whether a file belongs in a codex:

```bash
ygg tree src/ --symbols
ygg tree src/ --symbols --max-symbols 5
```

```
├── main.rs                1.0k tok
│   ◆ Cli
│   ◆ Commands
│   ◆ Args
│   ◆ main
```

Supported for Rust, Python, JavaScript, TypeScript, JSX, and TSX. Files in
languages without an extractor stay silent rather than claiming they declare
nothing. Test functions and `#[cfg(test)]` modules are excluded.

---

# How Yggdrasil Selects Files (Critical)

Yggdrasil **never prints the entire repo by default**.
You must specify *what* to include using any of:

* `--only <paths…>`
* `--show <extensions…>`
* `--white <manifest>`
* `--sniff <entry file>`
* or interactively, with `ygg pick`

You may also exclude using:

* `--ignore`
* `--black`

Formatting is separate:

* `--printed` → Markdown (`SHOW.md` by default)
* `--contents --out FILE` → explicit output mode

**`--printed` does not select files.**
It only specifies the output format.

## Hidden files and `.gitignore`

Dotfiles are skipped by default. Two things override that:

* `--hidden` includes them in any scan.
* **Naming one explicitly is enough.** If a `--only` or `--white` pattern
  mentions a dot-path, Yggdrasil lifts the hidden filter for that walk — so a
  manifest containing `.windsurf/rules/viceroy.md` works without extra flags.
  This is narrow on purpose: `--only src` still skips `src/.cache`.

`.gitignore` is a separate filter and still applies. If a path you named is
gitignored, add `--no-ignore`:

```bash
ygg --white WHITE.md --no-ignore --printed
```

---

# Snapshot Examples

These examples always include a file-selection flag, so they are guaranteed to
produce output.

## Export all `.rs` and `.md` files as Markdown

```bash
ygg --show rs md --printed
```

## Export specific files and directories

```bash
ygg --only src/main.rs \
        src/scanner \
        src/snapshot/format_selection.rs \
        src/snapshot/writer.rs \
    --printed
```

## Export files listed in a manifest

`WHITE.md`:

```
src/lib/model.rs
src/app/main.tsx
README.md
```

Command:

```bash
ygg --white WHITE.md --printed
```

## Use explicit `--contents --out`

Markdown:

```bash
ygg --show py --contents --out PY_SNAPSHOT.md
```

Plain text:

```bash
ygg --show rs --contents --out snapshot.txt
```

## List file paths without contents

```bash
ygg --show rs
ygg --show py md txt
```

## Flatten everything under a directory

```bash
ygg --only src --printed
```

## Package the output as a ZIP

```bash
ygg --only src --printed --zip
```

`SHOW.md` becomes `SHOW.zip`. Combined with `--split`, every shard is bundled
into a single archive — convenient for upload interfaces that accept one file.

---

# Sniff Mode — Semantic File Expansion

`--sniff` is the fastest way to build a codex when you have a single entry point
and want everything it depends on — without manually listing files.

Given an entry file, Yggdrasil reads its static imports, resolves them to local
files inside `--dir`, and repeats recursively until no new local files are found.
The full reachable set is fed into the snapshot pipeline exactly like `--only`.

```bash
ygg --sniff path/to/entry.py --dir path/to/project --printed
```

## How it works

1. Start from the entry file
2. Read its top-level (preamble) imports only
3. Resolve each import to local files inside `--dir`
4. Recursively repeat for every discovered file
5. Stop when no new local files are found
6. Pass the complete set to the snapshot pipeline

External libraries (`numpy`, `pandas`, etc.) are silently ignored — only files
that exist inside `--dir` are included.

## Examples

```bash
# Snapshot an analysis script and all its local dependencies
ygg --sniff scripts/analysis/audit.py --dir ../my-project --printed

# Same, but split into LLM-safe shards
ygg --sniff scripts/analysis/audit.py --dir ../my-project --printed --split 10

# Combine with --ignore to exclude noisy files
ygg --sniff src/main.py --dir . --ignore tests --printed
```

## What sniff resolves

For Python, given `from graveyard.meta.macro_runner import load_data_macro`:

* `graveyard/meta/macro_runner.py`
* `graveyard/meta/macro_runner/__init__.py`
* truncated forms: `graveyard/meta.py`, `graveyard.py`

The first candidate that exists inside `--dir` is followed.

## What sniff does not do

* It does not scan the entire repo
* It does not analyze runtime behavior or call graphs
* It does not follow relative imports (`.sibling`, `..parent`) — planned
* It does not include external libraries
* Python only, for now

## Mental model

| Flag | Selection method |
|------|-----------------|
| `--only` | manual paths |
| `--white` | manifest file |
| `--sniff` | semantic expansion from entry point |
| `pick` | interactive |

Sniff is just a smart way to fill `--only`.
All other flags (`--ignore`, `--split`, `--printed`, etc.) apply normally after expansion.

---

# Large Codices & Context Limits

Yggdrasil can split large codices into **LLM-safe shards** while preserving structure.

Use `--split` to divide output into multiple standalone codex files:

```bash
ygg --only <...> --printed --split
ygg --only <...> --printed --split 8
ygg --white <WHITE.md> --printed --split 10
ygg --whited --split
ygg --whited --split 30
ygg --sniff entry.py --dir ../project --printed --split 10
```

Each shard:

* preserves canonical file order
* never breaks files mid-content
* includes full INDEX + FILES structure
* is independently valid for AI ingestion

Splitting is expressed in **thousands of tokens**, not raw token counts.

---

# Interactive Modes

There are two ways to choose files interactively.

## `ygg pick` — the tree

Navigate, tick, see the running token cost, then copy or write. Documented
above. This is the recommended workflow.

## `--whited` — paste mode

The original interactive flow: paste a list of paths, get `SHOW.md`.

```bash
ygg --whited
```

```
Enter WHITE patterns (one per line):
Tip: Paste your paths (e.g., from VS Code → Copy Relative Path).
Finish with Ctrl+D (Linux/macOS) or Ctrl+Z then Enter (Windows).
```

Paste:

```
src/main.rs
src/utils/io.rs
README.md
```

`--whited` implies `--white`, implies `--contents`, and writes Markdown to
`SHOW.md`. `--treed` is the index-only counterpart: same paste flow, but the
FILES section is omitted.

**`--white` never triggers interactive input** — only `--whited` and `--treed` do.

---

# Diff Mode

Compare directories:

```bash
ygg diff src/ -- old_src/
```

Compare specific files:

```bash
ygg diff controller.py -- controller_old.py
```

Align annotations:

```bash
ygg diff --align-tags src/ -- old_src/
```

Diff features:

* inline diff visualization
* contextual additions/removals
* cross-file movement detection
* `[MOVED → file:line]` annotations
* optional aligned metadata

---

# Installation

Requires Rust:

```bash
cargo install yggdrasil-cli
```

Ensure `~/.cargo/bin` is in your path.

Install from local source:

```bash
cargo install --path . --force
```

Optional, for clipboard support on Linux:

```bash
sudo apt install xclip          # X11
sudo apt install wl-clipboard   # Wayland
```

---

# Philosophy

In Norse myth, Yggdrasil is the world-tree unifying realms.
This tool unifies your project structure into one portable artifact.

Design principles:

* Explicit over implicit
* Deterministic, repeatable output
* Minimal configuration
* LLM-friendly structure
* Complete control over what's included
* Show the cost before it is paid

---

# Roadmap

### Completed (v0.2.4)

* Snapshot export
* Markdown and plain-text modes
* Manifests: `--white` and interactive `--whited`
* Blacklists: `--ignore`, `--black`
* `--only` and `--show` filters
* Diff engine
* Block movement detection
* `--align-tags`
* `--printed`

### Completed (v0.3.0)

* `--split`: LLM-safe shard output

### Completed (v0.4.0)

* `--sniff`: semantic file expansion from a single entry point
* Recursive static import resolution bounded to `--dir`
* Nordic-flavoured sniff header in both CLI and Markdown output
* `--dir` promoted to named flag for robustness

### Completed (v0.7.0)

* `ygg tree`: box-drawn directory view, weighted by token cost
* Bare `ygg`: flat token-weighted listing
* `--symbols`: declared-symbol extraction for Rust, Python, JS, TS
* `--zip`: package generated output into an archive

### Completed (v0.8.0)

* `ygg pick`: interactive terminal file selection with live token budgeting
* Tri-state subtree selection
* Mouse and keyboard navigation

### Completed (v0.9.0)

* `c` in pick mode: copy the codex straight to the system clipboard
* Native clipboard tool detection with OSC 52 fallback
* Explicitly named dot-paths are no longer skipped by the walker

### Planned (v1.0)

* Relative import resolution in `--sniff` (`.sibling`, `..parent`)
* Multi-language sniff (Rust `use`, TypeScript `import`)
* Filter-as-you-type inside `ygg pick`
* Symbols shown inline in the picker
* Themeable CLI output
* HTML codex export
* Combined codex+diff bundles

---

# License

MIT License.