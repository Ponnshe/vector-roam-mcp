use rust_vector_engine::parser_mod::{org_kind::OrgKind, parser::Parser};
use std::fs;

#[test]
fn test_parser_generates_correct_payloads_from_fixture() {
    let filepath = "tests/fixtures/sample.org";
    let org_content = fs::read_to_string(filepath)
        .expect("Test fixture sample.org must exist in tests/fixtures/ directory");

    let parser = Parser::new(&org_content).expect("Parser should successfully process the fixture");

    assert_eq!(
        parser.org_id(),
        "e4cc951c-1f37-42ab-932b-e6695cdf0671",
        "Should extract the exact ID from the properties drawer"
    );
    assert_eq!(
        parser.file_title(),
        "OSTEP - Chapter 5",
        "Should extract the correct file title"
    );

    let tags = parser.tags();
    assert_eq!(tags, vec!["study"], "Should extract and clean FILETAGS");

    let category = parser
        .get_keyword("CATEGORY")
        .expect("Should have CATEGORY keyword");
    assert_eq!(category[0], "TOOL", "Should extract CATEGORY correctly");

    let sections = parser.sections();
    assert!(
        !sections.is_empty(),
        "Parser should have generated multiple sections"
    );

    let mut found_rust_src_block = false;
    let mut found_c_src_block = false;
    let mut found_bash_src_block = false;

    for section in sections {
        let text_content = match section.kind() {
            OrgKind::Preamble { .. } | OrgKind::Headline { .. } | OrgKind::SrcBlock { .. } => {
                section.get_text(&org_content)
            }
        };

        if text_content.is_empty() {
            continue;
        }

        let (line_start, line_end) = section.line_range();
        let byte_range = section.byte_range();

        assert!(
            line_end >= line_start,
            "Line range must be logically valid (end >= start)"
        );
        assert!(
            byte_range.end > byte_range.start,
            "Byte range must be logically valid"
        );
        assert!(
            !text_content.is_empty(),
            "Text content cannot be empty if it bypassed the filter"
        );

        if let OrgKind::SrcBlock { language, .. } = section.kind() {
            match language.as_str() {
                "rust" => found_rust_src_block = true,
                "C" => found_c_src_block = true,
                "bash" => found_bash_src_block = true,
                _ => {}
            }
        }
    }

    assert!(
        found_rust_src_block,
        "Parser failed to capture Rust source blocks"
    );
    assert!(
        found_c_src_block,
        "Parser failed to capture C source blocks"
    );
    assert!(
        found_bash_src_block,
        "Parser failed to capture Bash source blocks"
    );
}
