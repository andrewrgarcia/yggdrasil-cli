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

# Large Codices & Context Limits

Yggdrasil can split large codices into **LLM-safe shards** while preserving structure.

Use `--split` to divide output into multiple standalone codex files:

```bash
ygg --only <...> --printed --split
ygg --only <...> --printed --split 8
ygg --white <WHITE.md> --printed --split 10
ygg --whited --split 
ygg --whited --split 30
```

Each shard:

* preserves canonical file order
* never breaks files mid-content
* includes full INDEX + FILES structure
* is independently valid for AI ingestion

Splitting is expressed in **thousands of tokens**, not raw token counts.

This allows large projects to pass through constrained context windows intact — piece by piece.

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
* Complete control over what’s included

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

### Planned (v0.3 → v1.0)

* Tree-view / flat-view toggle
* Themeable CLI output
* HTML codex export
* Combined codex+diff bundles

---

# License

MIT License.
