# Turn Around

**Version control for video. Commits, branches, and a time-travel slider through your project history.**

Turn Around is a desktop app that brings a Git-like workflow to video production. Stop juggling `Project_final_v3_really_FINAL.prproj` and guessing which timeline was the good one. Initialize a project, commit when it matters, branch to experiment, and scrub back through every version you’ve ever saved.

---

## Why this exists

Developers have Git. Video editors have … duplicate project files and hope. Turn Around fixes that: one place to track project files, timelines, and media, with real history you can browse and restore.

- **Commits** — Snapshot your project with a message (and optional milestone).
- **Branches** — Try a risky cut or alternate edit without touching the main timeline.
- **Time-travel slider** — Move through commit history and see what changed, when.
- **Timeline diffs** — Compare two versions side-by-side: tracks and clips, added/removed/modified.
- **Smart storage** — Content-addressable dedup so large media doesn’t duplicate. Small and project files are stored in full; big assets are hashed and referenced.
- **File watcher** — Edits are detected; you get a nudge to commit instead of forgetting.

Supports **OpenTimelineIO (OTIO)** and **Final Cut Pro XML (FCPXML)**. Your data stays on your machine. No cloud lock-in.

---

## What it looks like

- **Dashboard** — All your Turn Around projects: open, rename, archive, see stats.
- **Workspace** — Timeline diff viewer (ghost timeline), history sidebar, commit dialog, branch list.
- **Compare mode** — Pick two commits and see track- and clip-level changes.

---

## Tech

| Layer   | Stack |
|--------|--------|
| Frontend | Angular 21, TypeScript, RxJS |
| Backend  | Rust |
| Desktop  | Tauri 2 |
| Storage  | SQLite (per-project + global registry), content-addressable object store |

Rust handles VCS, timeline parsing, file watching, and backups. Angular handles the UI. Tauri wires them into a single native app.

---

## Get started

**Prerequisites:** Node.js (v18+), Rust toolchain, pnpm/npm.

```bash
git clone https://github.com/your-username/TrunAround.git
cd TrunAround
npm install
npm run tauri dev
```

**First run:** Open the app → “New project” or “Open folder” → choose a directory (or create one). Turn Around initializes a `.editgit` project there. Add your project files and media, then use the workspace to commit, branch, and browse history.

**Build for production:**

```bash
npm run tauri build
```

Binaries land in `src-tauri/target/release/`.

---

## License

MIT. See [LICENSE](LICENSE). Use it, fork it, ship it.

---

## Contributing

Issues and PRs welcome. If you’re into timeline formats (OTIO, FCPXML, AAF, etc.), parsers and diff improvements are high-impact. Check open issues or open one with a use case.

---

*Turn Around — so you can go back.*
