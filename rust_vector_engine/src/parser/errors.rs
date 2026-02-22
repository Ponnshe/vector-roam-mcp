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
#[derive(Error, Debug, PartialEq)]
pub enum LineError {
    #[error("Byte offset {0} is out of bound")]
    OffsetOutOfBound(usize),
    #[error("Invalid range: start ({start}) is greater than end ({end})")]
    InvalidRange { start: usize, end: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum SectionError{
    #[error("Range {0:?} is out of bounds or splits UTF-8 boundaries in content")]
    InvalidSectionRange(Range<usize>),
    #[error("Failed to retrieve title from parent headline")]
    ParentTitleError,
    #[error("Range {0:?} is out of bounds or splits UTF-8 boundaries in content")]
    InvalidOrgKindRange(Range<usize>),
}

