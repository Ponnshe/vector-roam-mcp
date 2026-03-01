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

use std::{collections::HashMap, ops::Range};

use orgize::{
    Org, SyntaxKind,
    ast::{Headline, SourceBlock},
    export::{Container, Event, from_fn},
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
        if let Some(last_range) = self.content_ranges.last() {
            if last_range.contains(&range.start)
                && (range.end == last_range.end || last_range.contains(&(range.end - 1)))
            {
                return;
            }
        }
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

#[derive(Debug)]
pub struct Parser {
    sections: Vec<OrgSection<Validated>>,
    org_id: String,
    file_title: String,
    keywords: HashMap<String, Vec<String>>,
}

impl Parser {
    fn parse_global_metadata(
        org: &Org,
    ) -> Result<(String, String, HashMap<String, Vec<String>>), ParserError> {
        let document = org.document();
        let mut keywords: HashMap<String, Vec<String>> = HashMap::new();
        let properties = org.document().properties().ok_or(ParserError::EmptyOrgId)?;
        let org_id = properties
            .get("ID")
            .ok_or(ParserError::EmptyOrgId)?
            .to_string();
        let title = match document.title() {
            Some(doc_title) => doc_title,
            None => String::new(),
        };

        for kw in document.keywords() {
            let key = kw.key().to_uppercase();
            let val = kw.value().trim().to_string();

            match key.as_str() {
                "ID" | "TITLE" => {}
                _ => {
                    keywords.entry(key).or_default().push(val);
                }
            }
        }

        Ok((org_id, title, keywords))
    }

    fn handle_event(event: Event, context: &mut ParserContext) {
        match event {
            Event::Enter(Container::Headline(hdl)) => {
                if context.in_preamble {
                    context.in_preamble = false;
                    context.pop_and_finish_section();
                }
                let full_range = hdl.syntax().text_range();
                let end_of_line = hdl
                    .section()
                    .map(|s| s.syntax().text_range().start())
                    .unwrap_or_else(|| full_range.end());
                let start = usize::from(full_range.start());
                let end = usize::from(end_of_line);
                let range = start..end;
                context.add_pending_section(AuxiliarOrgKind::Headline {
                    handle: hdl,
                    headline_range: range,
                });
            }
            Event::Leave(Container::Headline(hdl)) => {
                context.pop_and_finish_section();
            }
            Event::Enter(Container::SourceBlock(src_block)) => {
                let language = src_block
                    .language()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "text".to_string());
                context.add_pending_section(AuxiliarOrgKind::SrcBlock { language });
                if let Some(content_node) = src_block
                    .syntax()
                    .children()
                    .find(|n| n.kind() == SyntaxKind::BLOCK_CONTENT)
                {
                    let text_range = content_node.text_range();
                    let range = usize::from(text_range.start())..usize::from(text_range.end());

                    context.add_content_range_to_last(range);
                }
            }
            Event::Leave(Container::SourceBlock(_)) => {
                context.pop_and_finish_section();
            }
            Event::Enter(container) => {
                if let Some(range) = Self::get_block_range(&container) {
                    // Lógica de apertura de Preamble
                    if context.stack.is_empty() && context.in_preamble {
                        context.add_pending_section(AuxiliarOrgKind::Preamble);
                    }

                    context.add_content_range_to_last(range);
                }
            }

            _ => {}
        }
    }

    fn get_block_range(container: &Container) -> Option<Range<usize>> {
        let text_range = match container {
            Container::Paragraph(c) => Some(c.syntax().text_range()),
            Container::List(c) => Some(c.syntax().text_range()),
            Container::OrgTable(c) => Some(c.syntax().text_range()),
            Container::Drawer(c) => Some(c.syntax().text_range()),
            Container::FixedWidth(c) => Some(c.syntax().text_range()),
            Container::QuoteBlock(c) => Some(c.syntax().text_range()),
            Container::CenterBlock(c) => Some(c.syntax().text_range()),
            Container::VerseBlock(c) => Some(c.syntax().text_range()),
            Container::SpecialBlock(c) => Some(c.syntax().text_range()),
            Container::ExampleBlock(c) => Some(c.syntax().text_range()),
            Container::Comment(c) => Some(c.syntax().text_range()),
            Container::DynBlock(c) => Some(c.syntax().text_range()),
            _ => None,
        }?;

        Some(usize::from(text_range.start())..usize::from(text_range.end()))
    }

    pub fn new(content: &str) -> Result<Self, ParserError> {
        let translator = LocationTranslator::new(content);
        let mut context = ParserContext::new(&translator);
        let org = Org::parse(content);
        let (org_id, file_title, keywords) = Self::parse_global_metadata(&org)?;
        let mut handler = from_fn(|event| {
            Self::handle_event(event, &mut context);
        });
        org.traverse(&mut handler);

        while !context.stack.is_empty() {
            let _ = context.pop_and_finish_section();
        }

        let sections = context
            .finished_sections
            .into_iter()
            .map(|s| {
                s.validate(content)
                    .map_err(|(_, err)| ParserError::ValidationError(err))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            sections,
            org_id,
            file_title,
            keywords,
        })
    }

    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn file_title(&self) -> &str {
        &self.file_title
    }

    pub fn keywords(&self) -> &HashMap<String, Vec<String>> {
        &self.keywords
    }

    pub fn get_keyword(&self, key: &str) -> Option<&[String]> {
        self.keywords.get(&key.to_uppercase()).map(|v| v.as_slice())
    }

    pub fn tags(&self) -> Vec<String> {
        self.keywords
            .get("FILETAGS")
            .into_iter()
            .flat_map(|v| v.iter())
            .flat_map(|s| s.split(':'))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
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
            panic!("Should be a SrcBlock");
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
        let content = r#":PROPERTIES:
    :ID: 12345
    :END:
    #+TITLE: My Great Note
    #+FILETAGS: :work:rust:
    #+CATEGORY: systems
    * A headline"#;
        let org = Org::parse(content);
        let (id, title, keywords) =
            Parser::parse_global_metadata(&org).expect("Should parse valid metadata");

        assert_eq!(id, "12345");
        assert_eq!(title, "My Great Note");

        assert_eq!(keywords.get("FILETAGS").unwrap()[0], ":work:rust:");
        assert_eq!(keywords.get("CATEGORY").unwrap()[0], "systems");
    }

    #[test]
    fn test_parse_global_metadata_missing_id_error() {
        let content = "#+TITLE: Note without ID\n* Headline";
        let org = Org::parse(content);
        let result = Parser::parse_global_metadata(&org);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParserError::EmptyOrgId);
    }

    #[test]
    fn test_tags_extraction_and_cleaning() {
        let content = r#":PROPERTIES:
:ID: id-123
:END:
#+FILETAGS: :work:
#+FILETAGS: :rust:coding:"#;
        let parser = Parser::new(content).expect("Should initialize parser");
        let tags = parser.tags();

        assert_eq!(tags, vec!["work", "rust", "coding"]);
    }

    #[test]
    fn test_parser_full_lifecycle_and_getters() {
        let content = r#":PROPERTIES:
:ID: uuid-999
:END:
#+TITLE: Integration Test
#+AUTHOR: Nervo
#+CATEGORY: testing

This is a paragraph in the preamble.

* Section 1
Content of section 1."#;

        let parser = Parser::new(content).expect("Should process a full file");

        assert_eq!(parser.org_id(), "uuid-999");
        assert_eq!(parser.file_title(), "Integration Test");
        assert_eq!(parser.get_keyword("AUTHOR").unwrap()[0], "Nervo");
        assert_eq!(parser.get_keyword("CATEGORY").unwrap()[0], "testing");

        // Preamble + Headline = 2 validated sections
        assert_eq!(parser.sections.len(), 2);
    }

    #[test]
    fn test_block_level_capture_avoids_duplicates() {
        let content = r#":PROPERTIES:
:ID: doc-1
:END:
* Content Node
This is a paragraph with *bold* and /italic/ and a [[link]].

- Item 1
- Item 2

#+begin_src rust
fn main() {}
#+end_src
"#;
        let parser = Parser::new(content).expect("Should parse the content");

        assert_eq!(
            parser.sections.len(),
            2,
            "There should be 1 SrcBlock and 1 Headline"
        );

        let src_section = &parser.sections[0];
        assert!(
            matches!(src_section.kind(), OrgKind::SrcBlock { .. }),
            "The first closed section should be the SrcBlock"
        );

        let hl_section = &parser.sections[1];

        if let OrgKind::Headline { content_ranges, .. } = hl_section.kind() {
            for range in &content_ranges {
                let total_len = content[range.clone()].len();
                let mut result = String::with_capacity(total_len);
                result.push_str(&content[range.clone()]);
                println!("CAPTURED: {:?}", result.trim());
            }

            assert_eq!(
                content_ranges.len(),
                2,
                "Should capture exactly 1 paragraph and 1 list block, avoiding child duplication"
            );
        } else {
            panic!("Section should be a Headline");
        }
    }
    #[test]
    fn test_parser_empty_content_fails() {
        let content = "";
        let result = Parser::new(content);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParserError::EmptyOrgId);
    }
}
