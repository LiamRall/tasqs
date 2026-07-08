# tasqs

A CLI/TUI task and note management tool for developers, designed so both humans (via a ratatui interface) and AI agents (via scriptable subcommands) work against the same data.

## Language

**Item**:
The single unit of content in a project. Every Item has a kind (Task or Note) and may be nested under any other Item, to any depth.
_Avoid_: Node, entry, todo

**Task**:
An Item that carries a Status and represents work to be done.
_Avoid_: Todo (as an entity name), ticket

**Status**:
The lifecycle state of a Task: Todo, InProgress, Done, or Cancelled.

**Result**:
Optional free-text recorded when a Task reaches Done or Cancelled, capturing outcomes and decisions. Serves as persistent memory for humans and AI agents.
_Avoid_: Outcome, resolution

**Note**:
An Item with no status; free-form written content that can parent or be parented by any Item.

**Focus**:
An ordered set of at most three Task references the user has chosen to work on. Adding a fourth requires removing one; a Task leaving Todo/InProgress leaves the Focus automatically. Persists across sessions.
_Avoid_: Pinned, starred, priority list

**Meeting**:
A dated, first-class record outside the project tree, holding meeting notes and links to zero or more Projects. The Meetings panel is a date-filtered view of Meetings.
_Avoid_: Event, appointment

**Action Item**:
A Task created from within a Meeting. It lives in a Project's tree like any Task but keeps an origin link back to the Meeting it came from.

**Project**:
The top-level container for Items. Represents a development project or a general work project; may point at a code repo (URL and local path) but never lives inside one.
_Avoid_: Workspace, folder

**Slug**:
A Project's unique short key, used in CLI commands and as its storage filename.
