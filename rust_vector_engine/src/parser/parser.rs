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

use std::{ops::Range, string::ParseError};

use orgize::{
    Org,
    ast::Headline,
    export::{Event, from_fn},
    rowan::ast::AstNode,
};

use crate::parser::{
    errors::{LineError, ParserError},
    location_translate::LocationTranslator,
    org_kind::OrgKind,
    org_section::{OrgSection, Unvalidated, Validated},
};

enum AuxiliarOrgKind {
    Preamble,
    Headline {
        handle: Headline,
        headline_range: Range<usize>,
    },
    SrcBlock {
        language: String,
    },
}

struct PendingOrgSection {
    auxiliar_kind: AuxiliarOrgKind,
    content_ranges: Vec<Range<usize>>,
    parents_at_creation: Vec<Headline>,
}

impl PendingOrgSection {
    pub fn new(auxiliar_kind: AuxiliarOrgKind, parents_at_creation: Vec<Headline>) -> Self {
        let content_ranges = vec![];
        Self {
            auxiliar_kind,
            content_ranges,
            parents_at_creation,
        }
    }

    pub fn push_content_range(&mut self, range: Range<usize>) {
        self.content_ranges.push(range);
    }
}

struct ParserContext<'a> {
    translator: &'a LocationTranslator,
    finished_sections: Vec<OrgSection<Unvalidated>>,
    stack: Vec<PendingOrgSection>,
    in_preamble: bool,
}

impl<'a> ParserContext<'a> {
    pub fn new(translator: &'a LocationTranslator) -> Self {
        Self {
            translator,
            finished_sections: Vec::new(),
            stack: vec![],
            in_preamble: true,
        }
    }

    pub fn add_pending_section(&mut self, auxiliar_kind: AuxiliarOrgKind) {
        let parents_at_creation = if let Some(last) = self.stack.last() {
            let mut parents = last.parents_at_creation.clone();

            if let AuxiliarOrgKind::Headline { handle, .. } = &last.auxiliar_kind {
                parents.push(handle.clone());
            }
            parents
        } else {
            vec![]
        };

        let pending_section = PendingOrgSection {
            auxiliar_kind,
            content_ranges: vec![],
            parents_at_creation,
        };

        self.stack.push(pending_section);
    }

    pub fn add_content_range_to_last(&mut self, range: Range<usize>) {
        if let Some(pending) = self.stack.last_mut() {
            pending.push_content_range(range);
        }
    }

    pub fn pop_and_finish_section(&mut self) -> Result<(), ParserError> {
        let pending = self.stack.pop().ok_or(ParserError::InternalStackError)?;

        let finished = match pending.auxiliar_kind {
            AuxiliarOrgKind::Preamble => {
                let start = pending.content_ranges.first().map(|r| r.start).unwrap_or(0);
                let end = pending.content_ranges.last().map(|r| r.end).unwrap_or(0);

                let byte_range = start..end;
                let line_range = self.translator.resolve_range(byte_range.clone())?;

                OrgSection::new(
                    OrgKind::Preamble {
                        content: byte_range.clone(),
                    },
                    byte_range,
                    line_range,
                    pending.parents_at_creation,
                )
            }
            AuxiliarOrgKind::Headline {
                handle,
                headline_range,
            } => {
                let full_range = handle.syntax().text_range();
                let byte_range = usize::from(full_range.start())..usize::from(full_range.end());
                let line_range = self.translator.resolve_range(byte_range.clone())?;

                let kind = OrgKind::Headline {
                    level: handle.level(),
                    headline_range,
                    content_ranges: pending.content_ranges,
                };

                OrgSection::new(kind, byte_range, line_range, pending.parents_at_creation)
            }
            AuxiliarOrgKind::SrcBlock { language } => {
                let content_range = pending.content_ranges.first().cloned().unwrap_or(0..0);
                let line_range = self.translator.resolve_range(content_range.clone())?;

                OrgSection::new(
                    OrgKind::SrcBlock {
                        language,
                        content_range: content_range.clone(),
                    },
                    content_range,
                    line_range,
                    pending.parents_at_creation,
                )
            }
        };

        self.finished_sections.push(finished);
        Ok(())
    }
}

pub struct Parser {
    sections: Vec<OrgSection<Validated>>,
    org_id: String,
    file_title: String,
    tags: Vec<String>,
    category: Option<String>,
}

impl Parser {
    fn parse_global_metadata(org: &Org) -> (String, String, Vec<String>, Option<String>) {
        todo!()
    }

    fn handle_event(event: Event, context: ParserContext) {
        todo!()
    }

    pub fn new(content: &str) -> Self {
        todo!()
    }

    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn file_title(&self) -> &str {
        &self.file_title
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn category(&self) -> Option<&String> {
        self.category.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use orgize::{
        Org,
        export::{Container, Event, from_fn},
    };

    #[test]
    fn test_new_context_state() {
        let content = "";
        let translator = LocationTranslator::new(content);
        let mut ctx = ParserContext::new(&translator);
        assert!(ctx.stack.is_empty());
        assert!(ctx.finished_sections.is_empty());
        assert!(ctx.in_preamble);
    }

    #[test]
    fn test_preamble_lifecycle() {
        let content = "Linea 1\nLinea 2";
        let lt = LocationTranslator::new(content);
        let mut ctx = ParserContext::new(&lt);

        ctx.add_pending_section(AuxiliarOrgKind::Preamble);
        ctx.add_content_range_to_last(0..7);
        ctx.add_content_range_to_last(8..15);

        ctx.pop_and_finish_section()
            .expect("Error al cerrar Preamble");

        assert_eq!(ctx.finished_sections.len(), 1);
        let sec = ctx.finished_sections.remove(0).validate(content).unwrap();

        assert_eq!(sec.byte_range(), 0..15);
        assert!(matches!(sec.kind(), OrgKind::Preamble { .. }));
    }

    #[test]
    fn test_headline_hierarchy_accumulation() {
        let content = "* H1\n** H2";
        let org = Org::parse(content);
        let lt = LocationTranslator::new(content);
        let mut ctx = ParserContext::new(&lt);

        let mut handler = from_fn(|event| match event {
            Event::Enter(Container::Headline(hdl)) => {
                ctx.add_pending_section(AuxiliarOrgKind::Headline {
                    handle: hdl.clone(),
                    headline_range: 0..4,
                });
            }
            _ => {}
        });

        org.traverse(&mut handler);

        assert!(ctx.stack.first().unwrap().parents_at_creation.is_empty());

        let h2_pending = ctx.stack.last().unwrap();
        assert_eq!(h2_pending.parents_at_creation.len(), 1);
        assert_eq!(h2_pending.parents_at_creation[0].level(), 1);

        ctx.pop_and_finish_section().unwrap();
        ctx.pop_and_finish_section().unwrap();

        assert_eq!(ctx.finished_sections.len(), 2);
        assert_eq!(
            ctx.finished_sections
                .remove(1)
                .validate(content)
                .unwrap()
                .get_parent_titles()
                .len(),
            0
        );
    }

    #[test]
    fn test_src_block_capture() {
        let content = "#+begin_src rust\nfn main() {}\n#+end_src";
        let lt = LocationTranslator::new(content);
        let mut ctx = ParserContext::new(&lt);

        ctx.add_pending_section(AuxiliarOrgKind::SrcBlock {
            language: "rust".to_string(),
        });

        let content_range = 17..29;
        ctx.add_content_range_to_last(content_range.clone());

        ctx.pop_and_finish_section().unwrap();

        let sec = ctx.finished_sections.remove(0);
        if let OrgKind::SrcBlock {
            language,
            content_range: r,
        } = sec.validate(content).unwrap().kind()
        {
            assert_eq!(language, "rust");
            assert_eq!(r, content_range);
        } else {
            panic!("Debería ser un SrcBlock");
        }
    }

    #[test]
    fn test_empty_stack_error() {
        let lt = LocationTranslator::new("");
        let mut ctx = ParserContext::new(&lt);
        let result = ctx.pop_and_finish_section();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParserError::InternalStackError
        ));
    }

    #[test]
    fn test_complex_nesting_and_parents() {
        let content = "* L1\n** L2\n*** L3";
        let org = Org::parse(content);
        let lt = LocationTranslator::new("");
        let mut ctx = ParserContext::new(&lt);

        let mut handler = from_fn(|event| match event {
            Event::Enter(Container::Headline(hdl)) => {
                ctx.add_pending_section(AuxiliarOrgKind::Headline {
                    handle: hdl.clone(),
                    headline_range: 0..1,
                });
            }
            _ => {}
        });

        org.traverse(&mut handler);

        let l3_parents = &ctx.stack.last().unwrap().parents_at_creation;
        assert_eq!(l3_parents.len(), 2, "L3 should have 2 parents (L1 and L2)");
        assert_eq!(l3_parents[0].level(), 1);
        assert_eq!(l3_parents[1].level(), 2);
    }

    #[test]
    fn test_parse_global_metadata_full() {
        let content = r#"#+ID: 12345
#+TITLE: My Great Note
#+FILETAGS: :work:rust:
#+CATEGORY: systems
* A headline"#;
        let org = Org::parse(content);
        let (id, title, tags, cat) = Parser::parse_global_metadata(&org);

        assert_eq!(id, "12345");
        assert_eq!(title, "My Great Note");
        assert_eq!(tags, vec!["work", "rust"]);
        assert_eq!(cat, Some("systems".to_string()));
    }

    #[test]
    fn test_parse_global_metadata_defaults() {
        let content = "* Just a headline";
        let org = Org::parse(content);
        let (id, title, tags, cat) = Parser::parse_global_metadata(&org);

        assert!(id.is_empty());
        assert!(title.is_empty());
        assert!(tags.is_empty());
        assert!(cat.is_none());
    }
}
