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

#[derive(Debug)]
pub struct Unvalidated;
#[derive(Debug)]
pub struct Validated;

#[derive(Debug)]
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

    fn check(
        &self,
        content: &str,
        range: &Range<usize>,
        field: &'static str,
    ) -> Result<(), SectionError> {
        if content.get(range.clone()).is_none() {
            return Err(SectionError::InvalidOrgKindRange {
                range: range.clone(),
                field,
            });
        }
        Ok(())
    }

    pub fn validate(self, content: &str) -> Result<OrgSection<Validated>, (Self, SectionError)> {
        if content.get(self.byte_range.clone()).is_none() {
            let range = self.byte_range.clone();
            return Err((self, SectionError::InvalidSectionRange(range)));
        }
        let res = match &self.kind {
            OrgKind::Preamble { content: r } => self.check(content, r, "preamble_content"),
            OrgKind::Headline {
                headline_range,
                content_ranges,
                ..
            } => self
                .check(content, headline_range, "headline_range")
                .and_then(|_| {
                    for r in content_ranges {
                        self.check(content, r, "headline_content_range")?;
                    }
                    Ok(())
                }),
            OrgKind::SrcBlock {
                content_range: r, ..
            } => self.check(content, r, "srcblock_content"),
        };
        if let Err(err) = res {
            return Err((self, err));
        }
        Ok(OrgSection {
            state: std::marker::PhantomData,
            kind: self.kind,
            byte_range: self.byte_range,
            line_range: self.line_range,
            parent_headlines: self.parent_headlines,
        })
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

    pub fn get_text(&self, content: &str) -> String {
        match &self.kind {
            OrgKind::Preamble {
                content: content_range,
            } => content[content_range.clone()].to_string(),
            OrgKind::Headline {
                headline_range,
                content_ranges,
                ..
            } => {
                let total_len = content[headline_range.clone()].len()
                    + content_ranges
                        .iter()
                        .map(|r| content[r.clone()].len())
                        .sum::<usize>();

                let mut result = String::with_capacity(total_len);

                result.push_str(&content[headline_range.clone()]);
                for range in content_ranges {
                    result.push_str(&content[range.clone()]);
                }
                result
            }
            OrgKind::SrcBlock {
                language,
                content_range,
            } => {
                format!("```{}\n{}\n```", language, &content[content_range.clone()])
            }
        }
    }

    pub fn get_parent_titles(&self) -> Vec<String> {
        self.parent_headlines
            .iter()
            .map(|h| h.title_raw().to_string())
            .collect()
    }

    pub fn kind(&self) -> OrgKind {
        self.kind.clone()
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }

    pub fn line_range(&self) -> (usize, usize) {
        self.line_range
    }
}

#[cfg(test)]
mod test {
    use orgize::{
        Org,
        export::{Container, Event, from_fn},
    };

    use super::*;

    fn get_headlines(content: &str) -> Vec<Headline> {
        let org = Org::parse(content);
        let mut headlines = vec![];
        let mut handler = from_fn(|event| {
            if let Event::Enter(Container::Headline(hdl)) = event {
                headlines.push(hdl);
            }
        });
        org.traverse(&mut handler);
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
        assert!(matches!(
            result,
            Err((
                _,
                SectionError::InvalidSectionRange(Range { start: 0, end: 85 })
            ))
        ));
    }

    #[test]
    fn validate_invalid_headline_range() {
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
        assert!(matches!(
            result,
            Err((_, SectionError::InvalidOrgKindRange { ref range, .. })) if *range == (0..18)
        ));
    }

    #[test]
    fn validate_invalid_headline_content_ranges() {
        let content = "* Some headline 🦀 with emoji";
        let section = OrgSection::new(
            OrgKind::Headline {
                level: 1,
                headline_range: 0..31,
                content_ranges: vec![0..3, 20..41],
            },
            0..31,
            (1, 1),
            vec![],
        );
        let result = section.validate(content);
        assert!(matches!(
            result,
            Err((_, SectionError::InvalidOrgKindRange { ref range, .. })) if *range == (20..41)
        ));
    }

    #[test]
    fn validate_invalid_preamble_content_range() {
        let content = "* Some headline 🦀 with emoji";
        let section = OrgSection::new(OrgKind::Preamble { content: 1..18 }, 0..31, (1, 1), vec![]);
        let result = section.validate(content);
        assert!(matches!(
            result,
            Err((_, SectionError::InvalidOrgKindRange { ref range, .. })) if *range == (1..18)
        ));
    }

    #[test]
    fn validate_invalid_srcblock_content_range() {
        let content = "* Some headline 🦀 with emoji";
        let section = OrgSection::new(
            OrgKind::SrcBlock {
                language: "rust".to_string(),
                content_range: 0..18,
            },
            0..31,
            (1, 1),
            vec![],
        );
        let result = section.validate(content);
        assert!(matches!(
            result,
            Err((_, SectionError::InvalidOrgKindRange { ref range, .. })) if *range == (0..18)
        ));
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
        let org_section = OrgSection::<Validated>::new_test(
            OrgKind::Preamble { content: 0..37 },
            0..37,
            (1, 1),
            vec![],
        );
        let parents = org_section.get_parent_titles();
        assert_eq!(parents, Vec::<String>::new());
    }

    #[test]
    fn get_parent_titles_order_ok() {
        let content = "* Headline 1\n** Headline 2\n*** Headline 3\n**** Headline 4";
        let headlines = get_headlines(content);
        let parents = vec![
            headlines[0].clone(),
            headlines[1].clone(),
            headlines[2].clone(),
        ];
        let section = OrgSection::<Validated>::new_test(
            OrgKind::Preamble { content: 0..37 },
            0..37,
            (1, 1),
            parents,
        );
        let parents_results = section.get_parent_titles();
        assert_eq!(parents_results[0], "Headline 1");
        assert_eq!(parents_results[1], "Headline 2");
        assert_eq!(parents_results[2], "Headline 3");
    }
}
