#!/usr/bin/env python3
"""
Python MCP Server for Org-roam Notes.
Handles reading, searching, and managing journal entries in an Org-roam directory.
"""

#    This program is free software: you can redistribute it and/or modify
#    it under the terms of the GNU General Public License as published by
#    the Free Software Foundation, either version 3 of the License, or
#    (at your option) any later version.
#
#    This program is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#    GNU General Public License for more details.
#
#    You should have received a copy of the GNU General Public License
#    along with this program.  If not, see <https://www.gnu.org/licenses/>.

from mcp.server.fastmcp import FastMCP
from datetime import date, datetime
import os
import re
from pathlib import Path
from dotenv import load_dotenv

# Load variables from .env file
load_dotenv()

# Dynamic Configuration
NOTES_PATH = os.getenv("NOTES_PATH")
if not NOTES_PATH:
    raise ValueError("Error: NOTES_PATH is not defined in the environment or .env")

BASE_PATH = Path(NOTES_PATH).expanduser().resolve()
if not BASE_PATH.exists() or not BASE_PATH.is_dir():
    raise FileNotFoundError(f"Error: NOTES_PATH '{BASE_PATH}' does not exist or is not a directory.")

JOURNAL_FOLDER = os.getenv("JOURNAL_REL_PATH", "journal")

mcp = FastMCP("My_Digital_Brain")

@mcp.resource("notes://today")
def get_daily_note() -> str: 
    """Return the content of today's journal"""
    today = date.today()
    formatted_date = today.strftime("%Y-%m-%d")
    path = BASE_PATH / JOURNAL_FOLDER / f"{formatted_date}.org"
    
    if not path.exists():
        return f"No journal entry for today ({formatted_date})."
    
    return path.read_text(encoding="utf-8")

@mcp.tool()
def search_org_roam_note(query: str) -> list[str]:
    """Search a note, ignoring UUID of org-roam"""
    results = []
    roam_pattern = re.compile(r"^\d{14}-") 

    for root, _, files in os.walk(BASE_PATH):
        for file in files:
            if file.endswith(".org"):
                clean_name = roam_pattern.sub("", file)
                if query.lower() in clean_name.lower():
                    rel_path = os.path.relpath(os.path.join(root, file), BASE_PATH)
                    results.append(rel_path)
    return results

@mcp.tool()
def read_specific_note(relative_path: str) -> str:
    """Read the content of a note using its relative path"""
    full_path = (BASE_PATH / relative_path).resolve()
    
    if not str(full_path).startswith(str(BASE_PATH)):
        return "Error: Access denied (path out of bounds)."

    try:
        return full_path.read_text(encoding="utf-8")
    except Exception as e:
        return f"Reading error: {str(e)}"

@mcp.tool()
def initialize_journal_day(formatted_date: str, day_title: str = "") -> str:
    """Creates an .org file for a specific day with metadata."""
    path = BASE_PATH / JOURNAL_FOLDER / f"{formatted_date}.org"
    
    if path.exists():
        return f"Warning: The journal for {formatted_date} is already initialized."

    path.parent.mkdir(parents=True, exist_ok=True)
    
    content = (
        f"#+TITLE: {day_title if day_title else formatted_date}\n"
        f"#+DATE: {formatted_date}\n"
        f"#+FILETAGS: :journal:automated:\n\n"
    )
    
    path.write_text(content, encoding="utf-8")
    return f"File created: {path.name}"

@mcp.tool()
def add_journal_entry(formatted_date: str, entry_body: str, entry_title: str = "") -> str:
    """Adds a timestamped entry to the day's journal."""
    path = BASE_PATH / JOURNAL_FOLDER / f"{formatted_date}.org"

    if not path.exists():
        initialize_journal_day(formatted_date)

    now = datetime.now()
    timestamp = now.strftime("<%Y-%m-%d %a %H:%M>")
    header = f"* {timestamp} {entry_title} :AUTOMATED:".strip()
    
    with open(path, "a", encoding="utf-8") as f:
        f.write(f"\n{header}\n{entry_body}\n")
    
    return f"Entry recorded with active timestamp: {timestamp}"

@mcp.tool()
def edit_journal_entry(formatted_date: str, old_text: str, new_text: str) -> str:
    """
    Replaces specific text within a journal entry.
    WARNING: This is a simple string replacement. Use with caution to avoid unintended changes.
    """
    path = BASE_PATH / JOURNAL_FOLDER / f"{formatted_date}.org"
    
    if not path.exists():
        return f"Error: The file {formatted_date}.org does not exist."

    content = path.read_text(encoding="utf-8")
    
    if old_text not in content:
        return "Error: Original text not found. Update failed."

    new_content = content.replace(old_text, new_text)
    path.write_text(new_content, encoding="utf-8")
    
    return "Journal entry updated successfully."

if __name__ == "__main__":
    mcp.run()