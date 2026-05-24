# Skill Wrangler

**Find, compare, and copy agent skills across your machine.**

Skill Wrangler is a cross-platform desktop app for anyone working with [Agent Skills](https://agentskills.io/) — the `SKILL.md` folders used by Claude, Cursor, Codex, and other AI coding tools. Point it at a folder, discover every skill on disk, preview their contents, and copy whole skill folders wherever you need them.

Built with **Tauri 2**, **Rust**, and **Svelte 5**. Runs on **Windows**, **macOS**, and **Linux**.

---

## Why Skill Wrangler?

Agent skills live in scattered places — `~/.claude`, `~/.agents`, `~/.cursor`, project repos, plugin caches. Moving skills between machines, projects, or agent setups usually means manual folder hunting and copy-paste.

Skill Wrangler gives you one place to:

- **Discover** every folder containing a `SKILL.md` file under a root you choose
- **Deduplicate** skills by folder name and compare contents when names collide
- **Preview** every file inside a skill before you copy it
- **Copy** entire skill folders — scripts, references, assets, and all — to one or many destinations

---

## Features

### Discovery & browsing

- **Recursive scan** from any root folder (home directory, a repo, a drive)
- **Alphabetical skill list** grouped by folder name
- **Content-aware deduplication** — same name + identical files merge into one entry; different contents show as separate variants
- **Context filters** for each supported agent path (`.claude`, `.agents`, `.windsurf`, …) plus “other”
- **Compact mode** for a names-only list when you're scanning hundreds of skills
- **Search** across name, description, path, and context

### Preview

- **File browser** for each skill folder — not just `SKILL.md`
- Click any file to view its contents (text files up to 512 KB; binary files show a size note)
- See all copy locations when identical skills exist in multiple places

### Copy & deploy

- **Multi-select** with Select all / Clear and `Ctrl/Cmd+A`
- **Conflict policies**: rename, skip, or overwrite when a folder already exists
- **Copy to all agent skill folders** — pick a repo root and copy into every known `{agent}/skills` directory found underneath (see [Supported agent paths](#supported-agent-paths))
- **Recent destinations** for quick re-use
- Copies the **entire skill folder**, not just the markdown file

### Smart defaults

- Skips `node_modules`, `.git`, `target`, recycle bins, and other noise during scans
- Remembers your last scan root and recent copy destinations
- Cancellable long-running scans

---

## Screenshots

<!-- Add screenshots before publishing:
![Skill Wrangler main window](docs/screenshot-main.png)
-->

_Screenshots coming soon._

---

## Quick start

### Prerequisites

| Requirement | Version |
|---|---|
| [Node.js](https://nodejs.org/) | 18+ |
| [Rust](https://www.rust-lang.org/tools/install) | 1.77+ |
| Platform toolchain | [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS |

### Run from source

```bash
git clone https://github.com/shiftynick/skill-wrangler.git
cd skill-wrangler
npm install
npm run tauri dev
```

### Build a release

```bash
npm run tauri build
```

Installers and binaries are written to `src-tauri/target/release/bundle/`.

---

## Usage

### 1. Choose a scan root

Click **Browse** and pick a folder to search — your home directory, a project repo, or a specific agent config path like `~/.agents`.

Skill Wrangler walks the tree and finds every directory that contains a `SKILL.md` file.

### 2. Browse and select skills

Skills appear in an alphabetical list. Each entry is named after its **folder name**.

- **Variants** appear when multiple folders share a name but have different file contents
- **Identical copies** (same name, same content, different paths) collapse into one entry with a copy count
- Toggle **Compact** for a dense names-only view
- Filter by agent context (all supported agent roots) or search by keyword

Select skills with the checkboxes, or press `Ctrl/Cmd+A` to select all filtered results.

### 3. Preview a skill

Click a skill row to open the preview panel. You'll see every file in that folder — click any file to read it. `SKILL.md` opens by default.

### 4. Copy skills

1. Choose a **destination** folder
2. Pick an **on conflict** policy (rename is the safe default)
3. Optionally enable **Copy to all agent skill folders under destination**
4. Click **Copy**

When multi-folder copy is enabled, Skill Wrangler finds known agent skill directories under your destination and copies into each one. For example, selecting `~/projects/my-app` might copy into both `.agents/skills` and `.claude/skills` if both exist in that repo.

---

## Supported agent paths

When using **Copy to all agent skill folders**, only canonical `{agent}/skills` directories are targeted:

| Agent / IDE | Path |
|---|---|
| Claude | `.claude/skills` |
| Agents | `.agents/skills` |
| Cursor | `.cursor/skills` |
| Windsurf | `.windsurf/skills` |
| Codex | `.codex/skills` |
| Gemini | `.gemini/skills` |
| Goose | `.goose/skills` |
| Continue | `.continue/skills` |

Generic `skills/` folders and paths like `.cursor/skills-cursor` are not included in multi-folder copy.

---

## Scan ignore list

These directories are skipped during discovery to keep scans fast and relevant:

- `node_modules`, `.git`, `target`, `dist`, `.pnpm-store`
- `plugins/cache`, `skills-cursor`
- Recycle bin folders (`$Recycle.Bin`, `RECYCLER`, `.Trash`, and similar)

---

## What counts as a skill?

A **skill** is any directory that directly contains a `SKILL.md` file. A skill folder often includes much more:

```
my-skill/
├── SKILL.md          # Required — instructions + frontmatter
├── reference.md      # Optional docs
├── scripts/          # Helper scripts
└── assets/           # Templates, images, etc.
```

Skill Wrangler always copies the **entire folder**, preserving scripts and assets alongside the markdown.

---

## Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://v2.tauri.app/) |
| Backend | Rust (`walkdir`, content hashing, filesystem copy) |
| Frontend | Svelte 5 + TypeScript + Vite |
| Persistence | Tauri Store plugin (scan root, recent destinations) |

---

## Project structure

```
skill-wrangler/
├── src/                      # Svelte frontend
│   └── lib/
│       ├── components/       # UI panels
│       └── stores/           # scan, preview, copy, ui state
└── src-tauri/src/
    ├── skill.rs              # SKILL.md parsing, agent path rules
    ├── scan.rs               # Filesystem discovery
    ├── walk.rs               # Shared filtered directory walk
    ├── group.rs              # Skill deduplication / grouping
    ├── folder/               # Content hashing, file listing, read
    ├── copy.rs               # Copy engine + multi-destination logic
    └── commands.rs           # Tauri command handlers (settings store)
```

---

## Development

```bash
# Type-check the frontend
npm run check

# Run Rust tests
cd src-tauri && cargo test
```

---

## Roadmap

- [ ] Release builds via GitHub Actions
- [ ] App screenshots in this README
- [ ] Move / cut (copy + delete source)
- [ ] Filesystem watch mode for live refresh
- [ ] Configurable scan ignore patterns in the UI

---

## Contributing

Contributions are welcome! Feel free to open an issue for bugs or feature requests, or submit a pull request.

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Commit your changes
4. Push and open a PR

Please run `npm run check` and `cargo test` before submitting.

---

## License

MIT — see [LICENSE](LICENSE) for details.
