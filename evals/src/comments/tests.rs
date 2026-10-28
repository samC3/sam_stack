use super::*;

const NONE: [usize; 0] = [];

fn lines(lang: Lang, src: &str) -> Vec<usize> {
    comment_lines(lang, src).into_iter().collect()
}

#[test]
fn language_comes_from_the_extension_or_a_bin_folder() {
    assert_eq!(Lang::of(Path::new("src/main.rs")), Some(Lang::Rust));
    assert_eq!(Lang::of(Path::new("checks.sh")), Some(Lang::Shell));
    assert_eq!(Lang::of(Path::new("a/notes.py")), Some(Lang::Python));
    assert_eq!(Lang::of(Path::new("task/bin/herdr")), Some(Lang::Shell));
    assert_eq!(Lang::of(Path::new("task/herdr")), None);
    assert_eq!(Lang::of(Path::new("README.md")), None);
}

#[test]
fn rust_line_doc_and_block_comments() {
    let src = "fn a() {}\n// line\n/// doc\nfn b() { let x = 1; /* block */ }\n//! inner\n";
    assert_eq!(lines(Lang::Rust, src), [2, 3, 4, 5]);
}

#[test]
fn rust_nested_block_comment_counts_its_lines() {
    let src = "/* a /* b */\n still */ fn c() {}\n// after\n";
    assert_eq!(lines(Lang::Rust, src), [1, 3]);
}

#[test]
fn rust_comment_markers_inside_strings_are_code() {
    let src = concat!(
        "let a = \"https://x\";\n",
        "let b = \"esc \\\" // still string\";\n",
        "let c = r#\"raw \"/* \"# ;\n",
        "let d = br\"bytes //\";\n",
        "let e = '/'; let f = '\\''; let g = \"/*\";\n",
        "fn h<'a>(x: &'a str) -> &'a str { x }\n",
        "let multi = \"one\ntwo // in string\";\n",
        "// real\n",
    );
    assert_eq!(lines(Lang::Rust, src), [9]);
}

#[test]
fn rust_line_numbers_survive_multiline_raw_strings_and_blocks() {
    let src = "let a = r\"x\ny\";\n/* p\nq */\n// here\n";
    assert_eq!(lines(Lang::Rust, src), [3, 5]);
}

#[test]
fn rust_unterminated_literals_stop_at_the_end() {
    assert_eq!(lines(Lang::Rust, "let a = \"open"), NONE);
    assert_eq!(lines(Lang::Rust, "let a = r#\"open"), NONE);
    assert_eq!(lines(Lang::Rust, "let a = r#x"), NONE);
    assert_eq!(lines(Lang::Rust, "let a = '\\u{1F600"), NONE);
    assert_eq!(lines(Lang::Rust, "/* open"), [1]);
    assert_eq!(lines(Lang::Rust, "let a = r ; let b = r2;"), NONE);
}

#[test]
fn shell_whole_line_and_trailing_comments() {
    let src = "#!/bin/sh\n# whole\necho hi # trailing\necho hi#not\n  # indented\n";
    assert_eq!(lines(Lang::Shell, src), [2, 3, 5]);
}

#[test]
fn shell_hash_inside_quotes_and_expansions_is_code() {
    let src = "echo '# no'\necho \"# no \\\" # still\"\necho ${#x} $#\necho \\# not\n";
    assert_eq!(lines(Lang::Shell, src), NONE);
}

#[test]
fn shell_quotes_can_span_lines() {
    let src = "echo \"one\n# inside\ntwo\" # after\n";
    assert_eq!(lines(Lang::Shell, src), [3]);
}

#[test]
fn shell_heredoc_bodies_are_data() {
    let src = "cat > a.md <<'MD'\n## P1\n## P2\nMD\n# after\ncat <<-EOF\n\t# body\n\tEOF\ncat <<< \"x\" # here\n";
    assert_eq!(lines(Lang::Shell, src), [5, 9]);
}

#[test]
fn shell_shellcheck_directive_and_late_shebang() {
    let src = "# shellcheck disable=SC2086\necho a\n#!/bin/sh\n";
    assert_eq!(lines(Lang::Shell, src), [3]);
}

#[test]
fn python_comments_and_directives() {
    let src = "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\nimport os  # noqa\nx = 1  # type: int\n# plain\ny = '#no'\nz = \"a\\\"#\"  # yes\n";
    assert_eq!(lines(Lang::Python, src), [5, 7]);
}

#[test]
fn python_docstrings_open_a_block_or_the_module() {
    let src = concat!(
        "\"\"\"Module doc.\"\"\"\n",
        "def f():\n",
        "    '''Function doc.'''\n",
        "    return 1\n",
        "class C:\n",
        "    \"\"\"Starts here\n",
        "    and goes on.\n",
        "    \"\"\"\n",
        "    x = \"\"\"not a docstring\"\"\"\n",
        "    y = 2\n",
        "\"\"\"open\n",
        "# inside\n",
        "\"\"\"; z = 1 # after\n",
    );
    assert_eq!(lines(Lang::Python, src), [1, 3, 6, 13]);
}

#[test]
fn python_a_string_after_ordinary_code_is_not_a_docstring() {
    let src = "x = 1\n'not doc'\nif x:  # header\n    'doc after a commented header'\n";
    assert_eq!(lines(Lang::Python, src), [3, 4]);
}

#[test]
fn python_unclosed_string_at_end() {
    assert_eq!(lines(Lang::Python, "x = 1\ny = \"\"\"never closed\n"), NONE);
    assert_eq!(lines(Lang::Python, "x = 'never closed"), NONE);
}

fn git(repo: &Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git {args:?}");
}

#[test]
fn added_since_reports_comments_on_added_lines_only() {
    let dir = crate::testing::TempDir::new();
    let repo = dir.path();
    dir.write("app.py", "# old comment\nx = 1\n");
    dir.write("lib.rs", "fn a() {}\n");
    dir.write("notes.md", "# heading\n");
    git(repo, &["init", "-q"]);
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", "base"]);
    git(repo, &["tag", "base"]);
    dir.write("app.py", "# old comment\nx = 1\ny = 2  # new\nz = 3\n");
    dir.write("lib.rs", "// top\nfn a() {}\n");
    dir.write("notes.md", "# heading\n# another\n");
    dir.write("new.sh", "echo a\n# fresh\n");
    dir.write("gone.py", "");
    git(repo, &["rm", "-q", "--cached", "notes.md"]);
    assert_eq!(
        added_since(repo, "base").unwrap(),
        ["app.py:3 y = 2  # new", "lib.rs:1 // top", "new.sh:2 # fresh"]
    );
}

#[test]
fn added_since_clean_and_bad_rev() {
    let dir = crate::testing::TempDir::new();
    dir.write("a.py", "x = 1\n");
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-qm", "base"]);
    assert_eq!(added_since(dir.path(), "HEAD").unwrap(), Vec::<String>::new());
    assert!(added_since(dir.path(), "nope").unwrap_err().contains("git diff"));
}

#[test]
fn added_lines_reads_hunk_headers() {
    let diff = "+++ b/a.py\n@@ -1 +2 @@\n@@ -5,0 +7,3 @@ def f\n+++ /dev/null\n@@ -1,2 +0,0 @@\n";
    let added = added_lines(diff);
    assert_eq!(added.len(), 1);
    assert_eq!(added["a.py"].iter().copied().collect::<Vec<_>>(), [2, 7, 8, 9]);
}

#[test]
fn rust_edge_cases_around_comment_markers() {
    assert_eq!(lines(Lang::Rust, "fn a() {} // end"), [1]);
    assert_eq!(lines(Lang::Rust, "let x = a / b * c;\n"), NONE);
    assert_eq!(lines(Lang::Rust, "/* a *x\n // here */\n"), [1]);
    assert_eq!(lines(Lang::Rust, "/* \"*/\n// later\n"), [1, 2]);
    assert_eq!(lines(Lang::Rust, "let a = b\"q\\\" // not\";\n"), NONE);
}

#[test]
fn rust_char_literals_that_hold_quotes() {
    assert_eq!(lines(Lang::Rust, "let q = '\"'; // c\n"), [1]);
    assert_eq!(lines(Lang::Rust, "let q = '\\\"'; // c\n"), [1]);
    assert_eq!(lines(Lang::Rust, "let v = ['\\u{41}','\"']; // c\n"), [1]);
}

#[test]
fn shell_single_quotes_hold_hashes() {
    assert_eq!(lines(Lang::Shell, "echo 'a # b'\n"), NONE);
}

#[test]
fn python_strings_end_where_they_close() {
    assert_eq!(lines(Lang::Python, "x = 1\ny = \"\"\"a # not\"\"\" + z\n"), NONE);
    assert_eq!(lines(Lang::Python, "x = 1\ny = \"\"\"ab\"\"\" # c\n"), [2]);
    assert_eq!(lines(Lang::Python, "x = 1\ny = \"\"\"a\"\"\"# c\n"), [2]);
    assert_eq!(lines(Lang::Python, "x = 1\ny = 'a#b'\n"), NONE);
}
