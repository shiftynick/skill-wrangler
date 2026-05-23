# Skill Wrangler

A cross-platform desktop app for discovering and copying agent skills. Skill Wrangler recursively scans a folder you choose, finds every directory that contains `SKILL.md`, and lets you copy whole skill folders to another location.

Built with **Tauri 2**, **Rust**, and **Svelte 5**.

## Features

- Recursive scan from any root folder
- Discovers skills anywhere under the root (not limited to `.claude` or `.agents`)
- Parses `name` and `description` from SKILL.md frontmatter
- Browse skills in a flat alphabetical list (one entry per unique folder name + content)
- Content hash detects when same-named skills differ
- Preview panel lists all files in a skill folder with click-to-view
- Multi-select copy with conflict handling: rename, skip, or overwrite
- Recent destination folders
- Preview first 40 lines of SKILL.md

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.77+
- Platform build tools for Tauri ([see Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Installers and binaries are written to `src-tauri/target/release/bundle/`.

## Usage

1. **Browse** to choose a scan root (e.g. your user profile or `C:\Users\you\.agents`).
2. Click **Rescan** to find all `SKILL.md` folders.
3. Browse the **folder tree** (expand/collapse) and select skills (Ctrl/Cmd+A selects all filtered).
4. Choose a **destination** folder (e.g. `~/.agents/skills` for another machine or project).
5. Pick a **conflict policy** and click **Copy**.

## Default ignore patterns

These directory names are skipped during scan:

- `node_modules`, `.git`, `target`, `dist`, `.pnpm-store`
- `plugins/cache`, `skills-cursor`
- Recycle bin folders: `$Recycle.Bin`, `RECYCLER`, `.Trash`, and similar (case-insensitive)

## Project structure

```
src/                 Svelte frontend
src-tauri/src/
  skill.rs           SKILL.md parsing and skill model
  scan.rs            Filesystem discovery
  copy.rs            Folder copy with conflict policies
  commands.rs        Tauri commands and settings store
```

## License

MIT
