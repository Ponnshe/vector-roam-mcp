# Python Notes MCP Server

Part of the "Personal AI Agent" ecosystem, this MCP server is responsible for direct interaction with your local **Org-roam** notes and journaling system.

## 🧠 Purpose
This server provides tools to:
- **Read** your daily journal entries.
- **Search** your knowledge base (org-roam notes) semantically (via filename/title matching currently).
- **Append** new entries to your journal automatically.
- **Initialize** new journal files if they don't exist.

## ⚙️ Configuration

This server requires environment variables to locate your notes. You should have a `.env` file in this directory or `export` these variables in your shell (handled automatically if using `direnv` and the root `.envrc`).

### Required Variables
- `NOTES_PATH`: Absolute path to your root Org-roam directory.

### Optional Variables
- `JOURNAL_REL_PATH`: Path relative to `NOTES_PATH` where journal entries are stored. Defaults to `journal`.

## 🛠️ Available Tools

| Tool | Description |
|------|-------------|
| `notes://hoy` | **Resource**. Returns the content of today's journal entry. |
| `search_org_roam_note` | Searches for notes by filename, ignoring the Org-roam timestamp prefix. |
| `read_specific_note` | Reads the full content of a note given its relative path. |
| `initialize_journal_day` | Creates a new `.org` file for a specific date if it doesn't exist. |
| `add_journal_entry` | Appends a timestamped entry to a specific day's journal. |
| `edit_journal_entry` | **Experimental**. Replaces specific text within a note. Use with caution. |

## 🚀 Development

Run using `uv` and the MCP CLI inspector:

```bash
uv run mcp dev main.py
```
