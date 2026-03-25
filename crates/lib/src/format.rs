const WRAP_COLUMN: usize = 80;

/// Format an org-mode document.
///
/// Plain-text paragraphs are reflowed to wrap at 80 columns.  All structured
/// elements — headings, keywords, blocks, drawers, tables, comments, list
/// items, horizontal rules, and fixed-width areas — are passed through
/// unchanged.
///
/// Paragraph boundaries are blank lines.  Paragraphs are never joined across a
/// blank line, and blank lines are preserved as-is.
pub fn format_org(input: &str) -> String {
  let mut output = String::with_capacity(input.len());
  let mut paragraph: Vec<&str> = Vec::new();
  let mut state = State::Normal;

  for line in input.lines() {
    match state {
      State::InBlock => {
        output.push_str(line);
        output.push('\n');
        if line.to_ascii_lowercase().starts_with("#+end_") {
          state = State::Normal;
        }
      }
      State::InDrawer => {
        output.push_str(line);
        output.push('\n');
        if line.trim().eq_ignore_ascii_case(":end:") {
          state = State::Normal;
        }
      }
      State::Normal => match classify_line(line) {
        LineKind::Plain => {
          paragraph.push(line);
        }
        LineKind::Blank => {
          flush_paragraph(&mut paragraph, &mut output);
          output.push('\n');
        }
        LineKind::BlockBegin => {
          flush_paragraph(&mut paragraph, &mut output);
          output.push_str(line);
          output.push('\n');
          state = State::InBlock;
        }
        LineKind::DrawerBegin => {
          flush_paragraph(&mut paragraph, &mut output);
          output.push_str(line);
          output.push('\n');
          state = State::InDrawer;
        }
        _ => {
          flush_paragraph(&mut paragraph, &mut output);
          output.push_str(line);
          output.push('\n');
        }
      },
    }
  }

  flush_paragraph(&mut paragraph, &mut output);
  output
}

fn flush_paragraph(lines: &mut Vec<&str>, output: &mut String) {
  if lines.is_empty() {
    return;
  }
  let text = lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
  let wrapped = textwrap::fill(&text, WRAP_COLUMN);
  output.push_str(&wrapped);
  output.push('\n');
  lines.clear();
}

#[derive(Debug, PartialEq)]
enum State {
  Normal,
  InBlock,
  InDrawer,
}

#[derive(Debug, PartialEq)]
enum LineKind {
  Blank,
  Heading,
  Keyword,
  BlockBegin,
  BlockEnd,
  DrawerBegin,
  DrawerEnd,
  Table,
  Comment,
  HorizRule,
  FixedWidth,
  ListItem,
  Escape,
  Plain,
}

fn classify_line(line: &str) -> LineKind {
  if line.trim().is_empty() {
    return LineKind::Blank;
  }

  // Block begin/end must start at column 0 (org-mode spec)
  let lower = line.to_ascii_lowercase();
  if lower.starts_with("#+begin_") {
    return LineKind::BlockBegin;
  }
  if lower.starts_with("#+end_") {
    return LineKind::BlockEnd;
  }

  // All other #+ keywords (column 0 only)
  if line.starts_with("#+") {
    return LineKind::Keyword;
  }

  // Headings: one or more * followed by space or end-of-line
  if is_heading(line) {
    return LineKind::Heading;
  }

  // Drawer end before begin so :END: doesn't match the drawer-begin pattern
  if line.trim().eq_ignore_ascii_case(":end:") {
    return LineKind::DrawerEnd;
  }
  if is_drawer_begin(line) {
    return LineKind::DrawerBegin;
  }

  // Tables
  if line.starts_with('|') {
    return LineKind::Table;
  }

  // Comments: "# " prefix or bare "#"
  if line == "#" || line.starts_with("# ") {
    return LineKind::Comment;
  }

  // Fixed-width areas: ": " prefix or bare ":"
  if line == ":" || line.starts_with(": ") {
    return LineKind::FixedWidth;
  }

  // Horizontal rules: five or more consecutive dashes
  if is_horiz_rule(line) {
    return LineKind::HorizRule;
  }

  // List items (unordered: "- " / "+ "; ordered: "N. " / "N) ")
  if is_list_item(line) {
    return LineKind::ListItem;
  }

  // Org escape character (prevents "#+..." from being interpreted as keyword)
  if line.starts_with(',') {
    return LineKind::Escape;
  }

  LineKind::Plain
}

fn is_heading(line: &str) -> bool {
  let after_stars = line.trim_start_matches('*');
  // Must have consumed at least one *, and what follows is either nothing or a space
  after_stars.len() < line.len()
    && (after_stars.is_empty() || after_stars.starts_with(' '))
}

fn is_drawer_begin(line: &str) -> bool {
  let trimmed = line.trim();
  if trimmed.len() < 3 || !trimmed.starts_with(':') || !trimmed.ends_with(':') {
    return false;
  }
  let name = &trimmed[1..trimmed.len() - 1];
  !name.is_empty()
    && name
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn is_horiz_rule(line: &str) -> bool {
  let trimmed = line.trim();
  trimmed.len() >= 5 && trimmed.chars().all(|c| c == '-')
}

fn is_list_item(line: &str) -> bool {
  let trimmed = line.trim_start();
  if trimmed.starts_with("- ") || trimmed.starts_with("+ ") {
    return true;
  }
  // Ordered: one or more digits followed by "." or ")" then space
  let after_digits = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
  after_digits.len() < trimmed.len()
    && (after_digits.starts_with(". ") || after_digits.starts_with(") "))
}
