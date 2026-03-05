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
use thiserror::Error;

/// Represents failures that occur during the translation of absolute byte offsets
/// to human-readable line numbers within the `LocationTranslator`.
#[derive(Error, Debug, PartialEq)]
pub enum LineError {
    /// The requested byte offset exceeds the total size of the source buffer.
    #[error("Byte offset {0} is out of bound")]
    OffsetOutOfBound(usize),

    /// A logically invalid range was provided where the start index is strictly
    /// greater than the end index.
    #[error("Invalid range: start ({start}) is greater than end ({end})")]
    InvalidRange { start: usize, end: usize },
}

/// Represents failures encountered during the Type-State transition from
/// `Unvalidated` to `Validated` in an `OrgSection`.
/// These errors ensure that extracted ranges are physically safe to slice from a UTF-8 string.
#[derive(Error, Debug, PartialEq)]
pub enum SectionError {
    /// The global byte range for the section exceeds the buffer length or
    /// attempts to slice within a multi-byte UTF-8 character boundary.
    #[error("Range {0:?} is out of bounds or splits UTF-8 boundaries in content")]
    InvalidSectionRange(Range<usize>),

    /// A specific internal range associated with an `OrgKind` variant
    /// (e.g., a `content_range` or `headline_range`) is invalid.
    #[error("Internal range {range:?} for field '{field}' is invalid")]
    InvalidOrgKindRange {
        range: Range<usize>,
        field: &'static str,
    },
}

/// The global error type for the `Parser` operations.
/// It wraps lower-level mapping and validation errors, and handles specific
/// metadata and AST traversal failures.
#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    /// The provided Org-mode string buffer is completely empty.
    #[error("The provided content is empty")]
    EmptyContent,

    /// Propagates errors originating from the `LocationTranslator` when calculating line numbers.
    #[error("Coordinate mapping failed: {0}")]
    MappingError(#[from] LineError),

    /// Indicates a corruption or misalignment in the DFS traversal stack of `ParserContext`,
    /// such as attempting to pop a section from an empty stack.
    #[error("Problem with the stack of pendings")]
    InternalStackError,

    /// The Org-mode file is missing the mandatory `:ID:` property drawer at the top level,
    /// which is strictly required for vector database payload tracking.
    #[error("The file has no org-id")]
    EmptyOrgId,

    /// Propagates errors originating from the structural validation phase of an `OrgSection`.
    #[error("Section validation failed: {0}")]
    ValidationError(#[from] SectionError),
}
