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

use orgize::ast::Headline;

use crate::parser::{errors::SectionError, org_kind::OrgKind};

pub struct Unvalidated;
pub struct Validated;

pub struct OrgSection<S = Unvalidated> {
    state: std::marker::PhantomData<S>,
    kind: OrgKind,
    byte_range: Range<usize>,
    line_range: (usize, usize),
    parent_headlines: Vec<Headline>,
}

impl OrgSection<Unvalidated> {
    pub fn new(
        kind: OrgKind,
        byte_range: Range<usize>,
        line_range: (usize, usize),
        parent_headlines: Vec<Headline>,
    ) -> Self {
        Self {
            state: std::marker::PhantomData,
            kind,
            byte_range,
            line_range,
            parent_headlines,
        }
    }

    pub fn validate(self, content: &str) -> Result<OrgSection<Validated>, (Self, SectionError)> {
        todo!();
    }
}

impl OrgSection<Validated> {
    #[cfg(test)]
    pub fn new_test(
        kind: OrgKind,
        byte_range: Range<usize>,
        line_range: (usize, usize),
        parent_headlines: Vec<Headline>,
    ) -> Self {
        Self {
            state: std::marker::PhantomData,
            kind,
            byte_range,
            line_range,
            parent_headlines,
        }
    }

    pub fn get_text<'a>(&self, content: &'a str) -> &'a str {
        todo!()
    }

    pub fn get_parent_titles(&self) -> Result<Vec<String>, SectionError> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::{error::Error, iter::from_fn};

    use orgize::{
        Org,
        export::{Container, Event},
        rowan::ast::AstNode,
    };

    use super::*;

    fn get_headlines(content: &str) -> Vec<Headline> {
        let org = Org::parse(content);
        let mut headlines = vec![];
        org.traverse(&mut |event| {
            if let Event::Enter(Container::Headline(hdl)) = event {
                headlines.push(hdl);
            }
        });
        headlines
    }

    #[test]
    fn validate_ok() {
        let content = "Some text as preamble of the document";
        let org_section =
            OrgSection::new(OrgKind::Preamble { content: 0..37 }, 0..37, (1, 1), vec![]);
        let result = org_section.validate(content);
        assert!(result.is_ok(), "Validation failed for a correct range");
    }

    #[test]
    fn validate_invalid_section_range() {
        let content = "Short";
        let section = OrgSection::new(
            OrgKind::Headline {
                level: 1,
                headline_range: 0..17,
                content_ranges: vec![17..45],
            },
            0..85,
            (1, 2),
            vec![],
        );
        let result = section.validate(content);
        assert_eq!(
            result,
            Err(SectionError::InvalidSectionRange(Range {
                start: 0,
                end: 85
            }))
        )
    }

    #[test]
    fn validate_invalid_org_kind_range() {
        let content = "* Some headline 🦀 with emoji";
        let section = OrgSection::new(
            OrgKind::Headline {
                level: 1,
                headline_range: 0..18,
                content_ranges: vec![],
            },
            0..31,
            (1, 1),
            vec![],
        );
        let result = section.validate(content);
        assert_eq!(
            result,
            Err(SectionError::InvalidOrgKindRange(Range {
                start: 0,
                end: 18
            }))
        )
    }

    #[test]
    fn get_text_preamble_ok() {
        let content = "Some text as preamble of the document";
        let org_section = OrgSection::<Validated>::new_test(
            OrgKind::Preamble { content: 0..37 },
            0..37,
            (1, 1),
            vec![],
        );
        let text = org_section.get_text(content);
        assert_eq!(text, "Some text as preamble of the document")
    }

    #[test]
    fn get_text_first_headline_ok() {
        let content = "* Headline Target\nSome text inside headline 1\n** Headline 2\nSome text inside headline 2";
        let section = OrgSection::<Validated>::new_test(
            OrgKind::Headline {
                level: 1,
                headline_range: 0..17,
                content_ranges: vec![17..45],
            },
            0..85,
            (1, 2),
            vec![],
        );
        let text = section.get_text(content);
        assert_eq!(text, "* Headline Target\nSome text inside headline 1");
    }

    #[test]
    fn get_text_second_headline_ok() {
        let content = "* Headline 1\nSome text inside headline 1\n** Headline Target\nSome text inside headline 2";
        let section = OrgSection::<Validated>::new_test(
            OrgKind::Headline {
                level: 2,
                headline_range: 41..60,
                content_ranges: vec![60..87],
            },
            41..87,
            (3, 4),
            //I put none parents because the get_text method does have to do nothing with this
            //info, get_parent_titles is the one that works with this param
            vec![],
        );
        let text = section.get_text(content);
        assert_eq!(text, "** Headline Target\nSome text inside headline 2");
    }

    #[test]
    fn get_parent_titles_none_ok() {
        let content = "Some text as preamble of the document";
        let org_section = OrgSection::<Validated>::new_test(
            OrgKind::Preamble { content: 0..37 },
            0..37,
            (1, 1),
            vec![],
        );
        let parents = org_section.get_parent_titles().unwrap();
        assert_eq!(parents, vec![]);
    }

    #[test]
    fn get_parent_titles_order_ok() {
        let content = "* Headline 1\n** Headline 2\n*** Headline 3\n**** Headline 4";
        let headlines = get_headlines(content);
        let parents = vec![headlines[0], headlines[1], headlines[2]];
        let section = OrgSection::<Validated>::new_test(
            OrgKind::Preamble { content: 0..37 },
            0..37,
            (1, 1),
            parents,
        );
        let parents_results = section.get_parent_titles().unwrap();
        assert_eq!(parents_results[0], "Headline 1");
        assert_eq!(parents_results[1], "Headline 2");
        assert_eq!(parents_results[2], "Headline 3");
    }
}
