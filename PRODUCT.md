# tasqs — Product Document

A task and note management tool for developers who live in the terminal. One binary, two faces: a three-panel ratatui TUI for humans, and scriptable CLI subcommands that make the same data a first-class workspace for AI agents.

Built in Rust — this is also its author's first Rust project, and the roadmap is deliberately sequenced to make that survivable.

See [CONTEXT.md](./CONTEXT.md) for canonical terminology and [docs/adr/](./docs/adr/) for recorded decisions.

## The problem

Developers scatter their working state across sticky notes, markdown files, Jira tickets they don't control, and their own heads. Meanwhile, AI coding agents are becoming daily collaborators but have no durable, structured view of what the developer is actually working on. tasqs gives both parties — the human and their agents — one shared, git-friendly source of truth for projects, tasks, notes, and meetings, without leaving the terminal.

## Product principles

1. **The terminal is home.** No web UI, no cloud service. TUI for browsing and daily driving; CLI for scripting and agents.
2. **Humans and agents are peers.** Every capability in the TUI has a CLI equivalent. An agent completing your task shows up live in your open TUI.
3. **Your data is plain text.** Git-friendly JSONL you can read, grep, diff, sync, and repair by hand ([ADR-0001](./docs/adr/0001-jsonl-flat-file-storage.md)).
4. **Constraint as feature.** The Focus panel holds at most three tasks — the product enforces prioritization rather than becoming another infinite list.
5. **Don't build what the OS has.** Multi-line editing suspends into `$EDITOR`; tasqs never grows an embedded text editor.

## Domain model

The canonical definitions live in [CONTEXT.md](./CONTEXT.md). Summary:

- **Project** — top-level container for Items. Has a unique `slug` (used in CLI commands and as its storage filename), display name, optional description, optional `repo_url` and `local_path`, and an archived flag. A project points at a code repo; it never lives inside one.
- **Item** — the single unit of content inside a project. Has `kind: Task | Note`, a title, a markdown body, and an optional `parent_id`. Items nest infinitely in any combination (tasks under notes, notes under tasks).
- **Task** — an Item with a **Status**: `Todo → InProgress → Done`, with `Cancelled` available from any active state. On close, a task may record a free-text **Result** (outcomes, decisions) — the persistent-memory feature that makes tasqs useful to agents across sessions.
- **Note** — an Item with no status; free-form content.
- **Meeting** — a first-class dated record *outside* any project tree: title, scheduled date/time, links to zero or more projects, and a notes body. The Meetings panel is just a date-filtered view.
- **Action Item** — a Task created from within a meeting. It lives in a project's tree like any other task but keeps a `meeting_id` origin link, so the meeting can list "tasks created here" and the task can answer "where did this come from?"
- **Focus** — an ordered set of at most three task references, persisted across sessions. Adding a fourth requires unfocusing one. A task leaving Todo/InProgress leaves the Focus automatically.

### Identity

Every Item and Meeting gets a random 8-character base32 ID (e.g. `k3f9x2ab`). The CLI accepts any unambiguous prefix, git-style (`tasqs task done k3f`). Random IDs are what make multi-machine git sync and parallel agent writes merge-safe; sequential per-project counters were rejected for exactly that reason (ADR-0001).

## The TUI

```
┌ PROJECTS ────────┐┌ MAIN ─────────────────────────────┐
│ ▸ api            ││ Project: api                      │
│   tasqs          ││ ├─ □ Refactor auth middleware     │
│   home-lab       ││ │   ├─ ✎ JWT library research     │
├ MEETINGS ────────┤│ │   └─ □ Write middleware tests   │
│ 09:30 standup    ││ ├─ ■ Migrate to axum 0.8          │
│ 14:00 sprint plan││ └─ ✎ Design ideas                 │
├ FOCUS ───────────┤│     └─ □ Try connection pooling   │
│ ① Refactor auth  ││                                   │
│ ② Storage bench  ││                                   │
│ ③ (empty)        ││                                   │
└──────────────────┘└───────────────────────────────────┘
```

- **Three panels on the left**: Projects, Meetings (today's + upcoming, date-filtered), Focus. **One main panel on the right.**
- Selecting a project or meeting drives the main panel: a collapsible item tree for projects; notes body, linked projects, and action items for meetings.
- The Focus panel never drives the main panel ("purely to show"), but it is interactive: from it you can mark a task done, unfocus it, or jump to it — jumping switches the main panel to that task's project with the task selected.
- **Keybindings** are vim-flavored but not modal: `j/k` (and arrows) move, `h/l` collapse/expand, `Tab`/`1..3` switch panels, single mnemonic keys act (`a` add, `e` edit, `d` done, `f` focus, `x` cancel, `/` filter), `?` opens a help overlay.
- **Editing**: titles and short fields edit inline; any multi-line body (`e` on a note or meeting) suspends the TUI and opens `$EDITOR` on a temp markdown file, reloading on save — the git-commit pattern devs already trust.
- **Live reload**: the TUI watches the data directory and re-renders when files change, so mutations made by the CLI — including by an AI agent mid-session — appear immediately.

## The CLI

Noun-verb subcommands over the same core:

```
tasqs                                    # no args → open the TUI
tasqs project add api --repo github.com/me/api --path E:\repos\api
tasqs task add "Refactor auth middleware" -p api
tasqs task add "Write tests" --parent k3f9
tasqs note add "Design ideas" -p api
tasqs task start k3f9
tasqs task done k3f9 --result "Went with tower middleware; see PR #42"
tasqs task list -p api --json
tasqs focus add k3f9 | tasqs focus list | tasqs focus remove 2
tasqs meeting add "Sprint planning" --at "2026-07-08 14:00" --project api
tasqs meeting note m4x2      # opens $EDITOR on the meeting body
```

Contract: every read command supports `--json`; every mutation prints the affected ID; exit codes are meaningful. When run inside a directory matching a project's `local_path`, commands auto-scope to that project.

## AI agent integration

The CLI *is* the agent API — an agent harness (Claude Code, etc.) manages tasks by shelling out, exactly like the human does:

- `--json` everywhere for parseable reads; stable IDs for reliable references.
- The Task **Result** field gives agents durable memory: what was done, what was decided, what to pick up next session (the dex insight).
- Plain-text JSONL means agents can also *read* state directly when that's cheaper than invoking the CLI.
- Planned: `tasqs agent` — prints concise usage instructions designed to be pasted into an agent's context (and later, a possible `tasqs mcp` server exposing the same core as typed MCP tools).

## Storage

Everything lives in one global, git-versionable data directory (platform data dir, e.g. `~/.local/share/tasqs/`):

```
tasqs/
├── projects/
│   ├── api.jsonl        # one Item per line
│   └── tasqs.jsonl
├── meetings.jsonl
├── focus.json
└── config.toml
```

- Scope is person-level, not repo-level: meetings and non-code projects have no repo to live in, so a single global store avoids running two storage systems. (A per-repo "linked project" mode is a possible future direction.)
- Writes rewrite the target file to a temp path and atomically rename it into place; concurrent writers are last-write-wins (accepted for a single-user tool). Details and consequences in [ADR-0001](./docs/adr/0001-jsonl-flat-file-storage.md).

## Architecture

A single installed binary backed by a Cargo workspace:

```
tasqs/
├── crates/
│   ├── tasqs-core/   # domain model, storage, ID generation — no UI deps
│   ├── tasqs-cli/    # clap subcommands over core
│   └── tasqs-tui/    # ratatui frontend over core
└── src/main.rs       # dispatch: no args → TUI, otherwise CLI
```

The core crate owns all behavior; both frontends stay thin. Likely dependencies: `serde`/`serde_json`, `clap`, `ratatui` + `crossterm`, `notify` (file watching), `chrono` or `jiff` (dates), `directories` (platform paths).

## Roadmap

Each release is independently useful; the hardest Rust learning (ownership, serde, error handling) happens on pure logic before any UI event loop.

| Version | Theme | Contents |
|---|---|---|
| 0.1 | Core + CLI | `tasqs-core` (Item/Project/Focus model, JSONL store, atomic writes, ID+prefix matching); CLI for project/task/note/focus; `--json` output. Usable and agent-ready with no TUI. |
| 0.2 | TUI | Three-panel layout (Meetings panel stubbed), project tree navigation, focus panel interaction, `$EDITOR` integration, file-watch live reload. |
| 0.3 | Meetings | Meeting entity + CLI, Meetings panel, meeting view in main panel, action items with origin links. |
| 0.4 | Agents | `tasqs agent` instructions command, JSON coverage audit, auto-scoping by `local_path`, evaluate `tasqs mcp`. |

Explicit non-goals for now: sync service, mobile/web UI, multi-user collaboration, notifications/reminders, calendar integration, embedded text editing.

## Open source

- License: **MIT OR Apache-2.0** (Rust ecosystem convention).
- Name: `tasqs` (verify crates.io availability before first publish).
- Repository will carry this document, `CONTEXT.md` as the terminology source of truth, and ADRs under `docs/adr/`.
