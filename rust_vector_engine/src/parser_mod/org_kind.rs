/*
    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::ops::Range;

/// Defines the semantic category and physical boundaries of a parsed Org-mode section.
///
/// `OrgKind` uses `Range<usize>` to maintain a zero-copy mapping to the original
/// file buffer. Instead of allocating strings during the parsing phase, it stores
/// the exact byte offsets where the content lives, allowing for efficient memory usage
/// and precise physical coordinate tracking for AI-driven edits.
#[derive(Debug, Clone)]
pub enum OrgKind {
    /// Represents the introductory text of the document before the first headline.
    ///
    /// This captures standard text (paragraphs, lists) but intentionally excludes
    /// global metadata blocks (like `#+TITLE`, `#+FILETAGS`, or `:PROPERTIES:` drawers).
    Preamble {
        /// The byte range covering the preamble's text content.
        content: Range<usize>,
    },

    /// Represents a structural headline and its immediate text content.
    ///
    /// To prevent the "matryoshka effect" (where parent nodes in the AST physically
    /// encapsulate all their child sub-headlines, causing massive text duplication
    /// in RAG chunks), the headline's title and its content blocks are kept strictly separate.
    Headline {
        /// The nesting depth of the headline (e.g., `*` is level 1, `**` is level 2).
        level: usize,
        /// The byte range of the title line itself, stopping at the first newline `\n`.
        headline_range: Range<usize>,
        /// A collection of byte ranges representing the fragmented text blocks
        /// (paragraphs, lists) that belong directly to this headline, explicitly
        /// ignoring any nested sub-headlines.
        content_ranges: Vec<Range<usize>>,
    },

    /// Represents a code block (`#+begin_src ... #+end_src`).
    SrcBlock {
        /// The programming language identifier (e.g., `"rust"`, `"python"`, `"C"`).
        /// Defaults to `"text"` if none is specified.
        language: String,
        /// The byte range of the actual code inside the block, excluding the
        /// `#+begin_src` and `#+end_src` boundary wrappers.
        content_range: Range<usize>,
    },
}
