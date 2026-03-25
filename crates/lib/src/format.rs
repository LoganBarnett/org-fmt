const WRAP_COLUMN: usize = 80;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Format an org-mode document.
///
/// Plain-text paragraphs and list items are reflowed to wrap at 80 *visible*
/// columns.  All other structured elements are passed through unchanged.
///
/// "Visible columns" means the display width of the text as rendered by
/// org-mode: bracket links `[[url][desc]]` count as the length of `desc`,
/// and `[[url]]` counts as the length of `url`.
///
/// Paragraph wrapping rules:
/// - Paragraphs are delimited by blank lines; they are never joined across one.
/// - Blank lines are preserved exactly.
///
/// List item wrapping rules:
/// - The continuation indent is aligned to the first text character after the
///   list marker (hanging indent).
/// - Extra spaces between the marker and the text are normalised to one.
/// - Checkboxes (`[ ]`, `[X]`, `[-]`) are kept as part of the marker.
/// - Unordered markers: `-` and `+`.
/// - Ordered markers: any sequence of alphanumeric characters followed by `.`
///   or `)` (covers `1.`, `a.`, `A.`, `iv.`, `1)`, …).
pub fn format_org(input: &str) -> String {
  let mut output = String::with_capacity(input.len());
  let mut paragraph: Vec<&str> = Vec::new();
  let mut list_item: Option<ListItemBuf> = None;
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
          // A plain line following a list item is a continuation of that item.
          if let Some(ref mut item) = list_item {
            item.push(line.trim());
          } else {
            paragraph.push(line);
          }
        }

        LineKind::Blank => {
          flush_paragraph(&mut paragraph, &mut output);
          flush_list_item(&mut list_item, &mut output);
          output.push('\n');
        }

        LineKind::ListItem => {
          flush_paragraph(&mut paragraph, &mut output);
          flush_list_item(&mut list_item, &mut output);
          if let Some(item) = ListItemBuf::from_line(line) {
            list_item = Some(item);
          } else {
            // Fallback: emit as-is (should not happen for well-formed input)
            output.push_str(line);
            output.push('\n');
          }
        }

        LineKind::BlockBegin => {
          flush_paragraph(&mut paragraph, &mut output);
          flush_list_item(&mut list_item, &mut output);
          output.push_str(line);
          output.push('\n');
          state = State::InBlock;
        }

        LineKind::DrawerBegin => {
          flush_paragraph(&mut paragraph, &mut output);
          flush_list_item(&mut list_item, &mut output);
          output.push_str(line);
          output.push('\n');
          state = State::InDrawer;
        }

        _ => {
          flush_paragraph(&mut paragraph, &mut output);
          flush_list_item(&mut list_item, &mut output);
          output.push_str(line);
          output.push('\n');
        }
      },
    }
  }

  flush_paragraph(&mut paragraph, &mut output);
  flush_list_item(&mut list_item, &mut output);
  output
}

// ---------------------------------------------------------------------------
// Flush helpers
// ---------------------------------------------------------------------------

fn flush_paragraph(lines: &mut Vec<&str>, output: &mut String) {
  if lines.is_empty() {
    return;
  }
  let text = lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
  output.push_str(&wrap_text(&text, WRAP_COLUMN));
  output.push('\n');
  lines.clear();
}

fn flush_list_item(slot: &mut Option<ListItemBuf>, output: &mut String) {
  if let Some(item) = slot.take() {
    item.flush(output);
  }
}

// ---------------------------------------------------------------------------
// List item accumulation buffer
// ---------------------------------------------------------------------------

struct ListItemBuf {
  /// Everything before the text (leading whitespace + normalised marker).
  prefix: String,
  /// Continuation indent: same visible length as `prefix`, all spaces.
  hang: String,
  /// Accumulated text words (trimmed lines collected so far).
  text: String,
}

impl ListItemBuf {
  /// Parse the first line of a list item.  Returns `None` if the line is not
  /// a recognisable list item.
  fn from_line(line: &str) -> Option<Self> {
    let (prefix, text) = parse_list_prefix(line)?;
    let hang = " ".repeat(prefix.len());
    Some(Self {
      hang,
      prefix,
      text: text.to_string(),
    })
  }

  /// Append a continuation line (already trimmed by the caller).
  fn push(&mut self, trimmed: &str) {
    if !self.text.is_empty() && !trimmed.is_empty() {
      self.text.push(' ');
    }
    self.text.push_str(trimmed);
  }

  /// Wrap and emit the item into `output`.
  fn flush(self, output: &mut String) {
    if self.text.is_empty() {
      output.push_str(self.prefix.trim_end());
      output.push('\n');
      return;
    }
    let wrapped =
      wrap_with_indent(&self.text, WRAP_COLUMN, &self.prefix, &self.hang);
    output.push_str(&wrapped);
    output.push('\n');
  }
}

/// Parse a list item line into (prefix, text).
///
/// `prefix` is the normalised first-line leader: leading whitespace + marker
/// + one space + optional checkbox + one space.  `text` is the content after
/// the prefix, with leading spaces stripped.
fn parse_list_prefix(line: &str) -> Option<(String, &str)> {
  let leading_len = line.len() - line.trim_start().len();
  let leading = &line[..leading_len];
  let rest = &line[leading_len..];

  let (marker, after) = parse_list_marker(rest)?;

  // Normalise: exactly one space between marker and content
  let content = after.trim_start();

  // Optional checkbox
  let (cb, text) = parse_checkbox(content);

  let prefix = match cb {
    Some(c) => format!("{}{} {} ", leading, marker, c),
    None => format!("{}{} ", leading, marker),
  };

  Some((prefix, text))
}

/// Detect the list marker at the start of `s` (no leading whitespace).
/// Returns `(marker_str, rest_after_one_mandatory_space)`.
fn parse_list_marker(s: &str) -> Option<(&str, &str)> {
  // Unordered
  if s.starts_with("- ") || s.starts_with("+ ") {
    return Some((&s[..1], &s[2..]));
  }
  // Ordered: one or more alphanumeric chars + "." or ")"
  let n = s.chars().take_while(|c| c.is_ascii_alphanumeric()).count();
  if n == 0 {
    return None;
  }
  let after_counter = &s[n..];
  if after_counter.starts_with(". ") || after_counter.starts_with(") ") {
    // marker = counter + delimiter (n + 1 chars), then skip the space
    Some((&s[..n + 1], &s[n + 2..]))
  } else {
    None
  }
}

/// Detect an optional checkbox at the start of `s`.
/// Returns `(Some(checkbox_str), remaining_text)` or `(None, s)`.
fn parse_checkbox(s: &str) -> (Option<&str>, &str) {
  for cb in ["[ ]", "[X]", "[x]", "[-]"] {
    if s.starts_with(cb) {
      let rest = s[cb.len()..].trim_start();
      return (Some(cb), rest);
    }
  }
  (None, s)
}

// ---------------------------------------------------------------------------
// Visible-column-aware text wrapping
// ---------------------------------------------------------------------------

/// Wrap `text` at `width` visible columns (no indent).
fn wrap_text(text: &str, width: usize) -> String {
  wrap_with_indent(text, width, "", "")
}

/// Wrap `text` at `width` visible columns with `first_indent` on the first
/// line and `rest_indent` on all subsequent lines.
fn wrap_with_indent(
  text: &str,
  width: usize,
  first_indent: &str,
  rest_indent: &str,
) -> String {
  let tokens = tokenize(text);
  let mut lines: Vec<String> = Vec::new();
  let mut current_raws: Vec<&str> = Vec::new();
  let mut current_vis: usize = 0;
  let mut is_first = true;

  for token in &tokens {
    let indent = if is_first { first_indent } else { rest_indent };
    let indent_vis = indent.chars().count();
    let tok_vis = token.display_width();

    if current_raws.is_empty() {
      current_raws.push(token.raw());
      current_vis = indent_vis + tok_vis;
    } else if current_vis + 1 + tok_vis <= width {
      current_raws.push(token.raw());
      current_vis += 1 + tok_vis;
    } else {
      lines.push(format!("{}{}", indent, current_raws.join(" ")));
      is_first = false;
      let indent = rest_indent;
      let indent_vis = indent.chars().count();
      current_raws = vec![token.raw()];
      current_vis = indent_vis + tok_vis;
    }
  }

  if !current_raws.is_empty() {
    let indent = if is_first { first_indent } else { rest_indent };
    lines.push(format!("{}{}", indent, current_raws.join(" ")));
  }

  lines.join("\n")
}

// ---------------------------------------------------------------------------
// Org-aware tokeniser
// ---------------------------------------------------------------------------

/// A word-level token that tracks both raw bytes and visible display width.
enum Token<'a> {
  Word(&'a str),
  /// A bracket link `[[url][desc]]` or `[[url]]`.
  /// `raw` is the original text; `display` is what org-mode renders.
  Link {
    raw: &'a str,
    display: &'a str,
  },
}

impl<'a> Token<'a> {
  fn raw(&self) -> &'a str {
    match self {
      Token::Word(s) => s,
      Token::Link { raw, .. } => raw,
    }
  }

  fn display_width(&self) -> usize {
    match self {
      Token::Word(s) => s.chars().count(),
      Token::Link { display, .. } => display.chars().count(),
    }
  }
}

/// Split `text` into tokens, treating `[[...]]` links as atomic units whose
/// display width is the visible description (or URL for bare links).
fn tokenize(text: &str) -> Vec<Token<'_>> {
  let mut tokens = Vec::new();
  let mut rest = text;

  while !rest.is_empty() {
    // Skip spaces between tokens
    if rest.starts_with(' ') {
      rest = rest.trim_start_matches(' ');
      continue;
    }

    // Bracket link: scan forward for matching ]]
    if rest.starts_with("[[") {
      if let Some(close_offset) = rest[2..].find("]]") {
        let end = 2 + close_offset + 2;
        let raw = &rest[..end];
        let display = link_display_text(raw);
        tokens.push(Token::Link { raw, display });
        rest = &rest[end..];
        continue;
      }
    }

    // Regular word: up to the next space or "[[", whichever comes first
    let space_pos = rest.find(' ').unwrap_or(rest.len());
    let link_pos = rest.find("[[").unwrap_or(rest.len());
    let word_end = space_pos.min(link_pos);

    if word_end == 0 {
      // Lone '[' or edge case — consume one char to avoid infinite loop
      let ch_len = rest.chars().next().map_or(1, |c| c.len_utf8());
      tokens.push(Token::Word(&rest[..ch_len]));
      rest = &rest[ch_len..];
    } else {
      tokens.push(Token::Word(&rest[..word_end]));
      rest = &rest[word_end..];
    }
  }

  tokens
}

/// Extract the visible text from a bracket link.
///
/// `[[url][description]]` → `description`
/// `[[url]]`              → `url`
fn link_display_text(raw: &str) -> &str {
  // raw is guaranteed to start with [[ and end with ]]
  let inner = &raw[2..raw.len() - 2];
  if let Some(sep) = inner.find("][") {
    &inner[sep + 2..]
  } else {
    inner
  }
}

// ---------------------------------------------------------------------------
// Line classifier
// ---------------------------------------------------------------------------

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

  // List items: unordered (-/+) or ordered (counter + ./))
  if is_list_item(line) {
    return LineKind::ListItem;
  }

  // Org escape character
  if line.starts_with(',') {
    return LineKind::Escape;
  }

  LineKind::Plain
}

fn is_heading(line: &str) -> bool {
  let after_stars = line.trim_start_matches('*');
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
  // Unordered
  if trimmed.starts_with("- ") || trimmed.starts_with("+ ") {
    return true;
  }
  // Ordered: alphanumeric counter + "." or ")" + space
  let n = trimmed
    .chars()
    .take_while(|c| c.is_ascii_alphanumeric())
    .count();
  if n > 0 {
    let rest = &trimmed[n..];
    if rest.starts_with(". ") || rest.starts_with(") ") {
      return true;
    }
  }
  false
}
