<!-- LOGO -->
<p align="center">
  <img src="https://github.com/user-attachments/assets/ad569d6e-1e01-407c-a282-3e3d2abb97dd" width="200" alt="yggdrasil logo"/>
</p>

<h1 align="center"> Yggdrasil</h1>
<p align="center">
  <strong>The god-tree of your codebase</strong><br/>
Flatten your entire project into an AI-ready codex — index + contents, in one command.
</p>

---

## 🤔 What is Yggdrasil?

Yggdrasil CLI is a **project flattener and diff tool**:  
it takes your codebase and transforms it into a single, structured document — or compares snapshots with rich, annotated diffs.  

Think of it as **`tree` + `cat` + `diff`**, but with superpowers:

- 📂 **Files** → indexed, filtered by extension, glob, or blacklist.  
- 📑 **Contents** → full text for each file, neatly marked.  
- 🔗 **Anchors** → clickable links from index → content (Markdown mode).  
- 🎨 **Stylish CLI** → cyberpunk colors, or plain mode for piping.  
- 🛡 **Controls** → `--only`, `--ignore`, `--blacklist`, `--out`.  
- 🧩 **Diff Mode** → cross-file block detection with `[MOVED]` annotations.  
- 📐 **Align Tags** → `--align-tags` keeps metadata comments lined up.  

---

## 🌟 Why would I want this?

- 🤖 **AI Prompts**: Feed your repo or a diff as one codex to ChatGPT/Claude.  
- 📚 **Docs & Reviews**: Export a clean snapshot for collaborators.  
- 🧑‍💻 **Developers**: Browse or compare projects with context in your terminal.  
- 🗄️ **Archival**: Serialize project state for reproducibility.  

Yggdrasil doesn’t just *list files* — it builds a **codex of your project**, and now shows how files evolve.  

---

## 🛠 How does it work?

Yggdrasil generates two kinds of outputs:

1. **Snapshot Mode** — index + file contents.  
2. **Diff Mode** — compares two sets of files, showing inline diffs *and* cross-file `[MOVED]` metadata.

### Snapshot Examples

```bash
# Export your repo as Markdown (index + contents)
ygg --show --md --contents --out SHOW.md

# List only file paths (no contents)
ygg --show rs
ygg --show py
```

### Diff Examples

```bash
# Compare two versions of a controller
ygg diff controller.py -- controller_old.py

# Compare multiple files against a single snapshot
ygg diff controller.py updates sampling trainer.py -- controller_old.py

# Align tags neatly at a column
ygg diff --align-tags src/ -- old_src/
```

---

## 📄 Manifest files

A manifest is just a plain text file with **one path per line**.
Only the files listed in the manifest will be shown.

Example `WHITE.md`:

```
src/pages/codebase.tsx
src/data/codebaseAssets.tsx
src/i18n/codebase.en.json
src/i18n/codebase.es.json
src/types/codebase.ts
```

Run with:

```bash
ygg --show --manifest WHITE.md --contents
```

---

## 🚀 Installation

You’ll need [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo install yggdrasil-cli
```

Then ensure `~/.cargo/bin` is in your `PATH`.

Upgrade after edits:

```bash
cargo install --path . --force
```

---

## 🌲 Philosophy

In Norse myth, **Yggdrasil** is the world-tree connecting all realms.
In your terminal, Yggdrasil connects all files — flattening complexity into a single codex, and now diffing branches of your code-tree.

It’s built to be:

* **Minimal**: no configs, just flags.
* **Readable**: AI-friendly and human-friendly.
* **Extensible**: Markdown, CLI, diff formatters, ignore lists, output redirection.

Goal: Make your project’s structure **transparent and portable**.

---

## 🛣 Roadmap

### v0.1 → v0.2

* ✅ Index & contents export (`--show`, `--contents`)
* ✅ Markdown mode (`--md`)
* ✅ Ignore & blacklist support (`--ignore`, `--blacklist`)
* ✅ Output to file (`--out`)
* ✅ Cross-file diff engine (`ygg diff`)
* ✅ `[MOVED]` metadata overlay
* ✅ `--align-tags` flag

### Future (v0.3 → v1.0)

* ⏳ Tree vs flat mode toggle
* ⏳ Configurable themes / styles
* ⏳ Unified codex+diff export

---

## 📜 License

MIT, like almost everything else that’s friendly and open-source.
