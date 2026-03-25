use org_fmt_lib::format::format_org;

// ---------------------------------------------------------------------------
// Paragraph wrapping
// ---------------------------------------------------------------------------

#[test]
fn short_paragraph_passes_through() {
  let input = "This is a short paragraph.\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn long_paragraph_is_wrapped() {
  let input = "This is a very long paragraph that definitely exceeds the eighty \
               character column limit and should therefore be wrapped by the formatter.\n";
  let output = format_org(input);
  for line in output.lines() {
    assert!(
      line.len() <= 80,
      "Line exceeds 80 chars: {:?}",
      line
    );
  }
}

#[test]
fn short_lines_within_paragraph_are_joined_and_rewrapped() {
  // Three short lines that together form one logical paragraph.
  let input = "First line of the paragraph.\nSecond line of the paragraph.\nThird line.\n";
  let output = format_org(input);
  // The output must contain all the words and have no blank line in the middle.
  assert!(!output.contains("\n\n"));
  assert!(output.contains("First line"));
  assert!(output.contains("Third line"));
}

#[test]
fn two_paragraphs_are_not_joined() {
  let input = "First paragraph.\n\nSecond paragraph.\n";
  let output = format_org(input);
  assert!(
    output.contains("\n\n"),
    "Blank line between paragraphs must be preserved"
  );
  // Verify order
  let blank_pos = output.find("\n\n").unwrap();
  assert!(output[..blank_pos].contains("First paragraph"));
  assert!(output[blank_pos..].contains("Second paragraph"));
}

#[test]
fn multiple_blank_lines_are_preserved() {
  let input = "Para one.\n\n\nPara two.\n";
  let output = format_org(input);
  assert!(output.contains("\n\n\n"));
}

// ---------------------------------------------------------------------------
// Headings
// ---------------------------------------------------------------------------

#[test]
fn heading_is_not_wrapped() {
  let line = "* This is a very long heading that exceeds the eighty character limit \
              and must not be reflowed by the formatter\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn nested_heading_is_not_wrapped() {
  let line = "*** This is a deeply nested heading that is also very long and must \
              not be wrapped at all by the formatter\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn bold_at_line_start_is_not_a_heading() {
  // *word* at the start of a line is inline bold, not a heading — it should
  // be treated as plain text and may be wrapped with its paragraph.
  let input = "*bold* text that forms a paragraph and could in principle be wrapped.\n";
  // It should be classified as plain and pass through (or be wrapped if long).
  // The key assertion is that it does NOT get treated as a heading and is
  // still present in the output.
  let output = format_org(input);
  assert!(output.contains("*bold*"));
}

// ---------------------------------------------------------------------------
// Keywords and directives (#+ lines)
// ---------------------------------------------------------------------------

#[test]
fn keyword_line_is_not_wrapped() {
  let line = "#+TITLE: This is a very long title that exceeds eighty characters \
              and must not be reflowed\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn begin_end_block_contents_are_not_wrapped() {
  let input = "#+begin_src rust\n\
               fn main() { println!(\"Hello, world! This is a long line that clearly exceeds 80 chars in length\"); }\n\
               #+end_src\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn begin_end_block_is_case_insensitive() {
  let input = "#+BEGIN_EXAMPLE\nsome long content that should not be wrapped at all because it is inside a block\n#+END_EXAMPLE\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn quote_block_is_not_wrapped() {
  let input = "#+begin_quote\nThis is a long quotation that should not be reformatted because it is inside a quote block.\n#+end_quote\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn text_after_block_is_wrapped_normally() {
  let long_para = "This paragraph comes after a block and is long enough that it \
                   should definitely be wrapped at the eighty column boundary by the formatter.";
  let input = format!(
    "#+begin_example\nsome content\n#+end_example\n\n{long_para}\n"
  );
  let output = format_org(&input);
  // Block contents unchanged
  assert!(output.contains("#+begin_example\nsome content\n#+end_example"));
  // Paragraph after block is wrapped
  for line in output.lines().skip_while(|l| !l.starts_with("This")) {
    assert!(line.len() <= 80, "Line too long: {:?}", line);
  }
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

#[test]
fn table_row_is_not_wrapped() {
  let line =
    "| column 1 | column 2 | this is a very long cell value that exceeds eighty characters |\n";
  assert_eq!(format_org(line), line);
}

// ---------------------------------------------------------------------------
// Drawers
// ---------------------------------------------------------------------------

#[test]
fn property_drawer_is_not_wrapped() {
  let input = ":PROPERTIES:\n\
               :ID: some-very-long-identifier-that-exceeds-eighty-chars-but-must-not-be-wrapped\n\
               :END:\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn custom_drawer_is_not_wrapped() {
  let input = ":LOGBOOK:\nCLOCK: [2024-01-01 Mon 09:00]--[2024-01-01 Mon 17:00] => 8:00\n:END:\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn text_after_drawer_is_wrapped_normally() {
  let long_para = "This paragraph follows a drawer and is long enough that it should \
                   be wrapped at the eighty column boundary by the formatter.";
  let input = format!(":PROPERTIES:\n:ID: abc\n:END:\n\n{long_para}\n");
  let output = format_org(&input);
  assert!(output.contains(":PROPERTIES:\n:ID: abc\n:END:"));
  for line in output.lines().skip_while(|l| !l.starts_with("This")) {
    assert!(line.len() <= 80, "Line too long: {:?}", line);
  }
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn comment_line_is_not_wrapped() {
  let line = "# This is a very long comment that exceeds the eighty character column \
              limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn bare_hash_is_not_wrapped() {
  let input = "#\n";
  assert_eq!(format_org(input), input);
}

// ---------------------------------------------------------------------------
// List items
// ---------------------------------------------------------------------------

#[test]
fn unordered_dash_list_item_is_not_wrapped() {
  let line = "- This is a very long list item that exceeds the eighty character \
              column limit and must not be wrapped by the formatter\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn unordered_plus_list_item_is_not_wrapped() {
  let line = "+ This is a very long list item that exceeds the eighty character \
              column limit and must not be wrapped by the formatter\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn ordered_period_list_item_is_not_wrapped() {
  let line = "1. This is a very long ordered list item that exceeds the eighty \
              character column limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn ordered_paren_list_item_is_not_wrapped() {
  let line = "1) This is a very long ordered list item that exceeds the eighty \
              character column limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn indented_list_item_is_not_wrapped() {
  let line = "  - This is an indented list item that exceeds the eighty character \
              column limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

// ---------------------------------------------------------------------------
// Escape lines
// ---------------------------------------------------------------------------

#[test]
fn escape_line_is_not_wrapped() {
  let line =
    ",#+this-is-an-escaped-keyword-line-that-is-quite-long-and-must-not-be-wrapped\n";
  assert_eq!(format_org(line), line);
}

// ---------------------------------------------------------------------------
// Fixed-width areas
// ---------------------------------------------------------------------------

#[test]
fn fixed_width_line_is_not_wrapped() {
  let line = ":  This is a fixed-width verbatim line that exceeds the eighty character \
              limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

// ---------------------------------------------------------------------------
// Horizontal rules
// ---------------------------------------------------------------------------

#[test]
fn horizontal_rule_is_not_wrapped() {
  let input = "-----\n";
  assert_eq!(format_org(input), input);
}

// ---------------------------------------------------------------------------
// Mixed content
// ---------------------------------------------------------------------------

#[test]
fn typical_org_document() {
  let input = "\
#+title: My Document

* Introduction

This is the introductory paragraph.  It talks about the project and gives some \
context for the reader.

** Details

Here are some details.

- First item
- Second item

#+begin_src rust
let x = 42; // this line is long and inside a src block so it must not be wrapped
#+end_src

:PROPERTIES:
:ID: abc-123
:END:

Final paragraph after everything.
";
  let output = format_org(input);

  // Structured elements unchanged
  assert!(output.contains("#+title: My Document"));
  assert!(output.contains("* Introduction"));
  assert!(output.contains("** Details"));
  assert!(output.contains("- First item"));
  assert!(output.contains("#+begin_src rust\nlet x = 42;"));
  assert!(output.contains("#+end_src"));
  assert!(output.contains(":PROPERTIES:\n:ID: abc-123\n:END:"));
  assert!(output.contains("Final paragraph after everything."));

  // All lines at most 80 chars (excluding lines we know are intentionally long
  // and are inside blocks)
  for line in output.lines() {
    if !line.contains("long and inside a src block") {
      assert!(line.len() <= 80, "Line too long: {:?}", line);
    }
  }
}
