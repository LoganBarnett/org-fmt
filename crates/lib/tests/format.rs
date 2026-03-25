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
    assert!(line.len() <= 80, "Line exceeds 80 chars: {:?}", line);
  }
}

#[test]
fn short_lines_within_paragraph_are_joined_and_rewrapped() {
  let input = "First line of the paragraph.\nSecond line of the paragraph.\nThird line.\n";
  let output = format_org(input);
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
  let blank = output.find("\n\n").unwrap();
  assert!(output[..blank].contains("First paragraph"));
  assert!(output[blank..].contains("Second paragraph"));
}

#[test]
fn multiple_blank_lines_are_preserved() {
  let input = "Para one.\n\n\nPara two.\n";
  assert!(format_org(input).contains("\n\n\n"));
}

// ---------------------------------------------------------------------------
// Headings
// ---------------------------------------------------------------------------

#[test]
fn heading_is_not_wrapped() {
  let line =
    "* This is a very long heading that exceeds the eighty character limit \
              and must not be reflowed by the formatter\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn nested_heading_is_not_wrapped() {
  let line =
    "*** This is a deeply nested heading that is also very long and must \
              not be wrapped at all by the formatter\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn bold_at_line_start_is_not_a_heading() {
  let input = "*bold* text that forms a paragraph.\n";
  let output = format_org(input);
  assert!(output.contains("*bold*"));
}

// ---------------------------------------------------------------------------
// Keywords and blocks
// ---------------------------------------------------------------------------

#[test]
fn keyword_line_is_not_wrapped() {
  let line =
    "#+TITLE: This is a very long title that exceeds eighty characters \
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
fn text_after_block_is_wrapped_normally() {
  let long_para =
    "This paragraph comes after a block and is long enough that it \
                   should definitely be wrapped at the eighty column boundary.";
  let input =
    format!("#+begin_example\nsome content\n#+end_example\n\n{long_para}\n");
  let output = format_org(&input);
  assert!(output.contains("#+begin_example\nsome content\n#+end_example"));
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
  let input =
    ":LOGBOOK:\nCLOCK: [2024-01-01 Mon 09:00]--[2024-01-01 Mon 17:00] => 8:00\n:END:\n";
  assert_eq!(format_org(input), input);
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn comment_line_is_not_wrapped() {
  let line =
    "# This is a very long comment that exceeds the eighty character column \
              limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn bare_hash_is_not_wrapped() {
  assert_eq!(format_org("#\n"), "#\n");
}

// ---------------------------------------------------------------------------
// List items — unordered
// ---------------------------------------------------------------------------

#[test]
fn short_unordered_list_item_passes_through() {
  let input = "- Short item.\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn long_unordered_dash_list_item_is_hang_wrapped() {
  // 80 visible cols: "- " (2) + text.  Continuation should align to col 2.
  let input = "- This is a very long list item that definitely exceeds the eighty \
               character column limit and needs to be wrapped by the formatter.\n";
  let output = format_org(input);
  let lines: Vec<&str> = output.lines().collect();
  assert!(lines[0].starts_with("- "), "First line must keep marker");
  for cont in lines.iter().skip(1) {
    assert!(
      cont.starts_with("  "),
      "Continuation must be indented 2 spaces, got: {:?}",
      cont
    );
    assert!(!cont.starts_with("   "), "Indent must be exactly 2 spaces");
  }
  for line in &lines {
    assert!(line.len() <= 80, "Line exceeds 80 chars: {:?}", line);
  }
}

#[test]
fn long_unordered_plus_list_item_is_hang_wrapped() {
  let input =
    "+ This is a very long list item using a plus marker that exceeds the \
               eighty character column limit and needs wrapping.\n";
  let output = format_org(input);
  assert!(output.lines().next().unwrap().starts_with("+ "));
  for line in output.lines() {
    assert!(line.len() <= 80);
  }
}

#[test]
fn extra_spaces_after_unordered_marker_are_normalised() {
  // "- " with extra spaces should be treated as "- " (one space)
  let input =
    "-   Three spaces then text that is long enough to require wrapping at \
               the eighty column boundary eventually.\n";
  let output = format_org(input);
  // First line starts with "- " (exactly one space after dash)
  assert!(
    output.lines().next().unwrap().starts_with("- "),
    "Marker should be normalised to '- '"
  );
}

// ---------------------------------------------------------------------------
// List items — ordered (numeric)
// ---------------------------------------------------------------------------

#[test]
fn long_ordered_period_list_item_is_hang_wrapped() {
  let input =
    "1. This is a very long ordered list item that definitely exceeds the \
               eighty character column limit and needs hang-wrapping.\n";
  let output = format_org(input);
  let lines: Vec<&str> = output.lines().collect();
  assert!(lines[0].starts_with("1. "), "First line must keep numeric marker");
  for cont in lines.iter().skip(1) {
    assert!(
      cont.starts_with("   "),
      "Continuation must be indented 3 spaces (len of '1. '), got: {:?}",
      cont
    );
    assert!(!cont.starts_with("    "), "Indent must be exactly 3 spaces");
  }
}

#[test]
fn long_ordered_paren_list_item_is_hang_wrapped() {
  let input =
    "1) This is a very long ordered list item using parens that exceeds the \
               eighty character column limit and needs hang-wrapping.\n";
  let output = format_org(input);
  assert!(output.lines().next().unwrap().starts_with("1) "));
  for line in output.lines() {
    assert!(line.len() <= 80);
  }
}

#[test]
fn two_digit_ordered_marker_indents_four_spaces() {
  // "10. " = 4 chars, so continuation must be 4 spaces
  let input =
    "10. This is a list item with a two-digit marker that is long enough \
               that it should be wrapped by the formatter correctly.\n";
  let output = format_org(input);
  let lines: Vec<&str> = output.lines().collect();
  assert!(lines[0].starts_with("10. "));
  for cont in lines.iter().skip(1) {
    assert!(
      cont.starts_with("    "),
      "Continuation must be 4 spaces for '10. ', got: {:?}",
      cont
    );
    assert!(!cont.starts_with("     "), "Indent must be exactly 4 spaces");
  }
}

// ---------------------------------------------------------------------------
// List items — ordered (lettered)
// ---------------------------------------------------------------------------

#[test]
fn lettered_period_list_item_is_hang_wrapped() {
  let input = "a. This is a lettered list item that is long enough to require \
               wrapping at the eighty character column boundary by the formatter.\n";
  let output = format_org(input);
  let lines: Vec<&str> = output.lines().collect();
  assert!(lines[0].starts_with("a. "), "First line must keep lettered marker");
  for cont in lines.iter().skip(1) {
    assert!(
      cont.starts_with("   "),
      "Continuation must be 3 spaces for 'a. ', got: {:?}",
      cont
    );
  }
}

#[test]
fn uppercase_lettered_list_item_is_hang_wrapped() {
  let input =
    "A. This is an uppercase lettered list item that is long enough to \
               require wrapping at the eighty character column boundary.\n";
  let output = format_org(input);
  assert!(output.lines().next().unwrap().starts_with("A. "));
  for line in output.lines() {
    assert!(line.len() <= 80);
  }
}

#[test]
fn lettered_paren_list_item_is_hang_wrapped() {
  let input = "b) This is a lettered list item with paren delimiter that is long \
               enough to require wrapping at the eighty character column boundary.\n";
  let output = format_org(input);
  assert!(output.lines().next().unwrap().starts_with("b) "));
  for line in output.lines() {
    assert!(line.len() <= 80);
  }
}

// ---------------------------------------------------------------------------
// List items — checkboxes
// ---------------------------------------------------------------------------

#[test]
fn unchecked_checkbox_item_is_hang_wrapped() {
  let input = "- [ ] This is an unchecked task item that is long enough to require \
               wrapping at the eighty character column boundary by the formatter.\n";
  let output = format_org(input);
  let first = output.lines().next().unwrap();
  assert!(
    first.starts_with("- [ ] "),
    "First line must keep '- [ ] ', got: {:?}",
    first
  );
  // Continuation indent = 6 spaces (length of "- [ ] ")
  for cont in output.lines().skip(1) {
    assert!(
      cont.starts_with("      "),
      "Continuation must be 6 spaces, got: {:?}",
      cont
    );
  }
}

#[test]
fn checked_checkbox_item_is_hang_wrapped() {
  let input = "- [X] This is a completed task item that is long enough to require \
               wrapping at the eighty character column boundary by the formatter.\n";
  let output = format_org(input);
  assert!(output.lines().next().unwrap().starts_with("- [X] "));
  for line in output.lines() {
    assert!(line.len() <= 80);
  }
}

#[test]
fn ordered_checkbox_item_is_hang_wrapped() {
  let input = "1. [ ] This is an ordered task item that is long enough to require \
               wrapping at the eighty character column boundary by the formatter.\n";
  let output = format_org(input);
  let first = output.lines().next().unwrap();
  assert!(
    first.starts_with("1. [ ] "),
    "First line must keep '1. [ ] ', got: {:?}",
    first
  );
}

// ---------------------------------------------------------------------------
// List items — nested and indented
// ---------------------------------------------------------------------------

#[test]
fn indented_list_item_preserves_leading_whitespace() {
  let input =
    "  - This is an indented list item that is long enough to require \
               wrapping at the eighty character column boundary.\n";
  let output = format_org(input);
  let first = output.lines().next().unwrap();
  assert!(
    first.starts_with("  - "),
    "Leading indent must be preserved, got: {:?}",
    first
  );
  // Continuation: 2 leading + 2 for "- " = 4 spaces
  for cont in output.lines().skip(1) {
    assert!(
      cont.starts_with("    "),
      "Continuation must be 4 spaces, got: {:?}",
      cont
    );
  }
}

#[test]
fn list_item_continuation_lines_are_rewrapped() {
  // Two short continuation lines should be joined and rewrapped with the item.
  let input = "- First sentence of a list item.\n  Second sentence that continues the same item.\n";
  let output = format_org(input);
  // Everything should be in one wrapped block (no blank line in the middle)
  assert!(
    !output.contains("\n\n"),
    "Continuation should not be separated from the item"
  );
  assert!(output.contains("First sentence"));
  assert!(output.contains("Second sentence"));
}

// ---------------------------------------------------------------------------
// Other non-wrappable elements
// ---------------------------------------------------------------------------

#[test]
fn escape_line_is_not_wrapped() {
  let line =
    ",#+this-is-an-escaped-keyword-line-that-is-quite-long-and-must-not-be-wrapped\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn fixed_width_line_is_not_wrapped() {
  let line =
    ":  This is a fixed-width verbatim line that exceeds the eighty character \
              limit and must not be wrapped\n";
  assert_eq!(format_org(line), line);
}

#[test]
fn horizontal_rule_is_not_wrapped() {
  assert_eq!(format_org("-----\n"), "-----\n");
}

// ---------------------------------------------------------------------------
// Visible-column wrapping (links)
// ---------------------------------------------------------------------------

#[test]
fn short_paragraph_with_link_passes_through() {
  // Short enough in visible columns that no wrapping is needed.
  let input = "See [[https://example.com][the docs]] for details.\n";
  assert_eq!(format_org(input), input);
}

#[test]
fn link_counts_as_visible_description_width_not_raw_width() {
  // Raw line is well over 80 chars due to the URL, but visible width is short.
  // The formatter must not wrap it.
  let url = "https://a-very-long-domain-name.example.com/with/a/very/long/path/to/some/resource";
  let input = format!("See [[{url}][the docs]] for details.\n");
  // raw length > 80, visible length (counting "the docs" = 8) is short
  assert!(input.trim_end().len() > 80, "precondition: raw line is long");
  let output = format_org(&input);
  assert_eq!(
    output, input,
    "Short visible width should not be wrapped even if raw bytes exceed 80"
  );
}

#[test]
fn long_visible_paragraph_with_link_wraps_at_visible_80() {
  // Visible text is over 80 columns; the link description contributes to that.
  let input = "This paragraph has a [[https://example.com][link]] and then enough \
               additional words after it to push the visible line length past eighty columns.\n";
  let output = format_org(input);
  // Every line's *visible* width (treating [[...][desc]] as len(desc)) must be ≤ 80.
  for line in output.lines() {
    let vis = visible_line_width(line);
    assert!(vis <= 80, "Visible width {} > 80 for line: {:?}", vis, line);
  }
}

#[test]
fn bare_link_counts_as_url_width() {
  // [[url]] — visible text is the url itself.
  let input = "See [[https://example.com/short]] for details.\n";
  assert_eq!(format_org(input), input);
}

// ---------------------------------------------------------------------------
// Orgize round-trip tests
// ---------------------------------------------------------------------------

/// Collect all bracket links from `text` as `(path, description)` pairs,
/// with internal whitespace normalised so that a path spanning lines in the
/// input compares equal to the same path on one line in the output.
fn collect_links_normalized(text: &str) -> Vec<(String, Option<String>)> {
  use orgize::{Element, Event, Org};

  let org = Org::parse(text);
  let mut links = Vec::new();

  for event in org.iter() {
    if let Event::Start(Element::Link(link)) = event {
      let path = normalize_ws(&link.path);
      let desc = link.desc.as_ref().map(|d| normalize_ws(d));
      links.push((path, desc));
    }
  }

  links
}

fn normalize_ws(s: &str) -> String {
  s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Compute the visible width of a line the same way our formatter does:
/// bracket links count as their description (or URL for bare links).
fn visible_line_width(line: &str) -> usize {
  let mut width = 0usize;
  let mut rest = line;
  while !rest.is_empty() {
    if rest.starts_with("[[") {
      if let Some(close) = rest[2..].find("]]") {
        let end = 2 + close + 2;
        let raw = &rest[..end];
        let inner = &raw[2..raw.len() - 2];
        let display = if let Some(sep) = inner.find("][") {
          &inner[sep + 2..]
        } else {
          inner
        };
        width += display.chars().count();
        rest = &rest[end..];
        continue;
      }
    }
    width += rest.chars().next().map_or(1, |c| c.len_utf8());
    rest = &rest[rest.chars().next().map_or(1, |c| c.len_utf8())..];
  }
  width
}

#[test]
fn orgize_link_count_preserved_after_formatting_short_paragraph() {
  let input = "See [[https://example.com/][the docs]] for details.\n";
  let output = format_org(input);
  assert_eq!(
    collect_links_normalized(input),
    collect_links_normalized(&output),
    "Links must be identical before and after formatting"
  );
}

#[test]
fn orgize_link_count_preserved_after_wrapping_long_paragraph() {
  let input = "This is a long paragraph containing [[https://example.com/page][a link \
               with a description]] plus extra words to push the line past eighty columns.\n";
  let output = format_org(input);
  let in_links = collect_links_normalized(input);
  let out_links = collect_links_normalized(&output);
  assert_eq!(
    in_links.len(),
    out_links.len(),
    "Same number of links: before={:?} after={:?}",
    in_links,
    out_links
  );
  assert_eq!(
    in_links, out_links,
    "Link paths and descriptions must match (whitespace-normalised)"
  );
}

#[test]
fn orgize_multiline_link_in_input_is_preserved() {
  // Emacs org-mode handles links whose description spans a line break, as the
  // user verified empirically.  Orgize 0.9, however, does NOT parse such links
  // — it fails to recognise the `[[` … `]]` pair when a newline appears inside.
  //
  // Our formatter joins continuation lines before wrapping, so a multi-line
  // link in the input becomes a single-line link in the output.  This test
  // documents that behaviour: the output is *more* compatible with strict
  // parsers than the input was.
  let input =
    "Test a long line that also has a long link.  Here is a [[really really long link\nthat just keeps going]].\n";
  let output = format_org(input);

  let in_links = collect_links_normalized(input);
  let out_links = collect_links_normalized(&output);

  // orgize 0.9 does not see the multi-line link in the raw input.
  assert_eq!(
    in_links.len(),
    0,
    "Orgize 0.9 should not parse a link that spans lines: {:?}",
    in_links
  );

  // After formatting the link is on one line and orgize can parse it.
  assert_eq!(
    out_links.len(),
    1,
    "Formatted output should contain exactly one link: {:?}",
    out_links
  );
  assert_eq!(
    out_links[0].0, "really really long link that just keeps going",
    "Link path should match the original content (whitespace-normalised)"
  );
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

  assert!(output.contains("#+title: My Document"));
  assert!(output.contains("* Introduction"));
  assert!(output.contains("** Details"));
  assert!(output.contains("- First item"));
  assert!(output.contains("#+begin_src rust\nlet x = 42;"));
  assert!(output.contains("#+end_src"));
  assert!(output.contains(":PROPERTIES:\n:ID: abc-123\n:END:"));
  assert!(output.contains("Final paragraph after everything."));

  for line in output.lines() {
    if !line.contains("long and inside a src block") {
      let vis = visible_line_width(line);
      assert!(vis <= 80, "Visible line too long ({vis}): {:?}", line);
    }
  }
}
