use std::{
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
};
use tempfile::NamedTempFile;

fn get_binary_path() -> PathBuf {
  let mut path =
    std::env::current_exe().expect("Failed to get current executable path");

  path.pop(); // remove test executable name
  path.pop(); // remove deps dir
  path.push("org-fmt");

  if !path.exists() {
    path.pop();
    path.pop();
    path.push("debug");
    path.push("org-fmt");
  }

  path
}

fn run(args: &[&str], stdin: Option<&str>) -> std::process::Output {
  let mut cmd = Command::new(get_binary_path());
  cmd.args(args);
  if stdin.is_some() {
    cmd.stdin(Stdio::piped());
  }
  cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

  if let Some(input) = stdin {
    let mut child = cmd.spawn().expect("Failed to spawn process");
    child
      .stdin
      .as_mut()
      .unwrap()
      .write_all(input.as_bytes())
      .expect("Failed to write stdin");
    child
      .wait_with_output()
      .expect("Failed to wait for process")
  } else {
    cmd.output().expect("Failed to execute binary")
  }
}

#[test]
fn test_help_flag() {
  let output = run(&["--help"], None);
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Usage:"));
}

#[test]
fn test_version_flag() {
  let output = run(&["--version"], None);
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("org-fmt"));
}

#[test]
fn stdin_short_paragraph_passes_through() {
  let input = "This is a short paragraph.\n";
  let output = run(&[], Some(input));
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert_eq!(String::from_utf8_lossy(&output.stdout), input);
}

#[test]
fn stdin_long_paragraph_is_wrapped() {
  let input = "This is a very long paragraph that definitely exceeds the eighty \
               character column limit and should therefore be wrapped by the formatter.\n";
  let output = run(&[], Some(input));
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  for line in stdout.lines() {
    assert!(line.len() <= 80, "Line exceeds 80 chars: {:?}", line);
  }
}

#[test]
fn file_argument_formats_file() {
  let mut tmp = NamedTempFile::new().unwrap();
  let input = "This is a very long paragraph that definitely exceeds the eighty \
               character column limit and should therefore be wrapped by the formatter.\n";
  tmp.write_all(input.as_bytes()).unwrap();

  let output = run(&[tmp.path().to_str().unwrap()], None);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  for line in stdout.lines() {
    assert!(line.len() <= 80, "Line exceeds 80 chars: {:?}", line);
  }
}

#[test]
fn in_place_modifies_file() {
  let mut tmp = NamedTempFile::new().unwrap();
  let input = "This is a very long paragraph that definitely exceeds the eighty \
               character column limit and should therefore be wrapped by the formatter.\n";
  tmp.write_all(input.as_bytes()).unwrap();

  let output = run(&["--in-place", tmp.path().to_str().unwrap()], None);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );

  let contents = std::fs::read_to_string(tmp.path()).unwrap();
  for line in contents.lines() {
    assert!(line.len() <= 80, "Line exceeds 80 chars: {:?}", line);
  }
}

#[test]
fn headings_not_wrapped_via_cli() {
  let input =
    "* This is a very long heading that exceeds the eighty character limit \
               and must not be reflowed by the formatter\n";
  let output = run(&[], Some(input));
  assert!(output.status.success());
  assert_eq!(String::from_utf8_lossy(&output.stdout), input);
}

#[test]
fn block_contents_not_wrapped_via_cli() {
  let input = "#+begin_src rust\nlet x = 42; // this is a long line inside a block that must not be wrapped at all\n#+end_src\n";
  let output = run(&[], Some(input));
  assert!(output.status.success());
  assert_eq!(String::from_utf8_lossy(&output.stdout), input);
}
