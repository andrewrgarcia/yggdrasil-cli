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

Yggdrasil CLI is a **project flattener**: it takes your codebase and transforms it into a single, structured document.  

Think of it as **`tree` meets `cat`**, but with superpowers:

- 📂 **Files** → indexed, filtered by extension, glob, or blacklist.  
- 📑 **Contents** → full text for each file, neatly marked.  
- 🔗 **Anchors** → clickable links from index → content (Markdown mode).  
- 🎨 **Stylish CLI** → cyberpunk colors, or plain mode for piping.  
- 🛡 **Controls** → `--only`, `--ignore`, `--blacklist`, `--out`.  

Why? To make your repo **AI-ready, doc-ready, and share-ready.**

---

## 🌟 Why would I want this?

- 🤖 **AI Prompts**: Feed your repo as one codex to ChatGPT/Claude.  
- 📚 **Docs & Reviews**: Export a clean snapshot for collaborators.  
- 🧑‍💻 **Developers**: Browse projects with context in your terminal.  
- 🗄️ **Archival**: Serialize project state for reproducibility.  

Yggdrasil doesn’t just *list files* — it builds a **codex of your project**.  

---

## 🛠 How does it work?

Yggdrasil generates two sections:  

1. **Files** — index of all discovered paths.  
2. **File Contents** — full file text, wrapped in markers.  

### Example commands

```bash
# Export your repo as Markdown (index + contents)
ygg --show --md --contents --out SHOW.md

# List only file paths (no contents)
ygg --show rs
ygg --show py
ygg --show json --ignore node_modules .next

# Restrict scan to a subdir
ygg --show md --only src

# Exclude files listed in BLACK.md
ygg --show --blacklist BLACK.md --contents

# Show only files listed in a manifest (WHITE.md)
ygg --show --manifest WHITE.md --contents

# Pipe into another tool (AI, pager, etc.)
ygg --show --md --contents | less
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
In your terminal, Yggdrasil connects all files — flattening complexity into a single codex.

It’s built to be:

* **Minimal**: no configs, just flags.
* **Readable**: AI-friendly and human-friendly.
* **Extensible**: Markdown, CLI, ignore lists, output redirection.

Goal: Make your project’s structure **transparent and portable**.

---

## 🛣 Roadmap (v0.1 → v1.0)

* ✅ Index & contents export (`--show`, `--contents`)
* ✅ Markdown mode (`--md`)
* ✅ Ignore & blacklist support (`--ignore`, `--blacklist`)
* ✅ Output to file (`--out`)
* ⏳ Tree vs flat mode toggle
* ⏳ Configurable themes / styles

---

## 📜 License

MIT, like almost everything else that’s friendly and open-source.
