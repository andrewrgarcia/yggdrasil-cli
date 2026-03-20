<!-- LOGO -->
<p align="center">
  <img src="https://github.com/user-attachments/assets/055bc31e-db08-41f8-8056-31ce0dd5a422" width="200" alt="yggdrasil logo"/>
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
* Accurate line counts
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

# How Yggdrasil Selects Files (Critical)

Yggdrasil **never prints the entire repo by default**.
You must specify *what* to include using any of:

* `--only <paths…>`
* `--show <extensions…>`
* `--white <manifest>`
* `--sniff <entry file>`

You may also exclude using:

* `--ignore`
* `--black`

Formatting is separate:

* `--printed` → Markdown (`SHOW.md` by default)
* `--contents --out FILE` → explicit output mode

**`--printed` does not select files.**
It only specifies the output format.

---

# Snapshot Examples

These examples are accurate and guaranteed to work because they always include a file-selection flag.

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
* It does not follow relative imports (`.sibling`, `..parent`) — planned for a future release
* It does not include external libraries

## Mental model

| Flag | Selection method |
|------|-----------------|
| `--only` | manual paths |
| `--white` | manifest file |
| `--sniff` | semantic expansion from entry point |

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

# Interactive Mode

### **Interactive paste mode is ONLY triggered by `--whited`.**

`--white` never triggers interactive input.

## The `--whited` Shortcut (Interactive)

`--whited` enables the fastest workflow:

* launches interactive paste mode
* implies `--white`
* implies `--contents`
* writes **Markdown** to `SHOW.md` automatically

Run:

```bash
ygg --whited
```

You will see:

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

Then Yggdrasil generates `SHOW.md` automatically.

This is the only flag that triggers interactive paste mode.

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

### Planned (v0.5 → v1.0)

* Relative import resolution in `--sniff` (`.sibling`, `..parent`)
* Multi-language sniff (Rust `use`, TypeScript `import`)
* Tree-view / flat-view toggle
* Themeable CLI output
* HTML codex export
* Combined codex+diff bundles

---

# License

MIT License.