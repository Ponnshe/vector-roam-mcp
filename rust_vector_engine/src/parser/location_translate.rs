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

use std::ops::RangeBounds;

use crate::parser::errors::LineError;

pub struct LocationTranslator {
    positions: Vec<usize>,
    total_bytes: usize,
}

impl LocationTranslator {
    pub fn new(content: &str) -> Self {
        let mut positions = vec![0];
        let total_bytes = content.len();

        for (i, byte) in content.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                positions.push(i + 1);
            }
        }

        Self {
            positions,
            total_bytes,
        }
    }

    pub fn resolve_line(&self, byte_offset: usize) -> Result<usize, LineError> {
        if byte_offset >= self.total_bytes {
            return Err(LineError::OffsetOutOfBound(byte_offset));
        }

        match self.positions.binary_search(&byte_offset) {
            Ok(idx) => Ok(idx + 1),
            Err(idx) => Ok(idx),
        }
    }

    pub fn resolve_range<R>(&self, range: R) -> Result<(usize, usize), LineError>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(s) => *s,
            std::ops::Bound::Excluded(s) => s + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(s) => *s,
            std::ops::Bound::Excluded(s) => s.saturating_sub(1),
            std::ops::Bound::Unbounded => self.total_bytes.saturating_sub(1),
        };

        if start > end {
            let err_start = match range.start_bound() {
                std::ops::Bound::Included(s) | std::ops::Bound::Excluded(s) => *s,
                std::ops::Bound::Unbounded => 0,
            };

            let err_end = match range.end_bound() {
                std::ops::Bound::Included(s) | std::ops::Bound::Excluded(s) => *s,
                std::ops::Bound::Unbounded => self.total_bytes.saturating_sub(1),
            };

            return Err(LineError::InvalidRange {
                start: err_start,
                end: err_end,
            });
        };

        let line_start = self.resolve_line(start)?;
        let line_end = self.resolve_line(end)?;
        Ok((line_start, line_end))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolve_line_offset_out_of_bound_err() {
        // Max offset is 45
        let content = "Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_line(46);
        assert_eq!(result, Err(LineError::OffsetOutOfBound(46)));
    }

    #[test]
    fn resolve_line_with_emojis_ok() {
        let content = "🦀 Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        //This is the first line
        let result1 = location_translator.resolve_line(6).unwrap();
        //This is the start of second line
        let result2 = location_translator.resolve_line(20).unwrap();
        //This is the start of the third line
        let result3 = location_translator.resolve_line(43).unwrap();

        assert_eq!(result1, 1);
        assert_eq!(result2, 2);
        assert_eq!(result3, 3);
    }

    #[test]
    fn resolve_range_end_out_of_bound_err() {
        let content = "Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_range(0..=46);
        assert_eq!(result, Err(LineError::OffsetOutOfBound(46)));
    }

    #[test]
    fn resolve_range_start_out_of_bound_err() {
        let content = "Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_range(47..=50);
        assert_eq!(result, Err(LineError::OffsetOutOfBound(47)));
    }

    #[test]
    fn resolver_range_with_emoji_ok() {
        let content = "🦀 Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_range(0..=20).unwrap();
        assert_eq!(result, (1, 2));
    }

    #[test]
    fn resolve_range_same_line_ok() {
        let content = "🦀 Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_range(0..=2).unwrap();
        assert_eq!(result, (1, 1));
    }

    #[test]
    fn resolve_range_empty_valid_range_ok() {
        let content = "🦀 Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_range(4..=4).unwrap();
        assert_eq!(result, (1, 1));
    }

    #[test]
    fn resolve_range_invert_range_err() {
        let content = "🦀 Hi how are you\nI'm fine, thanks, you?\nAll good";
        let location_translator = LocationTranslator::new(content);
        let result = location_translator.resolve_range(20..15);
        assert_eq!(result, Err(LineError::InvalidRange { start: 20, end: 15 }));
    }
}
