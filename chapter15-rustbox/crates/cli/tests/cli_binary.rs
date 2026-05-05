//! `rustbox` 바이너리 통합 테스트 (`cargo test -p rustbox`).

use std::fs;
use std::io::Write;
use std::process::Command;

fn rustbox_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rustbox"))
}

#[test]
fn echo_prints_joined_text_with_newline() {
    let out = rustbox_cmd()
        .args(["echo", "hello", "world"])
        .output()
        .expect("spawn rustbox echo");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello world\n");
}

#[test]
fn echo_n_skips_trailing_newline() {
    let out = rustbox_cmd()
        .args(["echo", "-n", "x"])
        .output()
        .expect("spawn rustbox echo -n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x");
}

#[test]
fn echo_e_processes_escapes() {
    let out = rustbox_cmd()
        .args(["echo", "-e", r"a\nb"])
        .output()
        .expect("spawn rustbox echo -e");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

#[test]
fn cat_prints_file_contents() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("sample.txt");
    fs::write(&p, "one\ntwo\n").unwrap();

    let out = rustbox_cmd()
        .args(["cat", p.to_str().unwrap()])
        .output()
        .expect("spawn rustbox cat");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "one\ntwo\n");
}

#[test]
fn head_first_lines() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("n.txt");
    let mut f = fs::File::create(&p).unwrap();
    writeln!(f, "a").unwrap();
    writeln!(f, "b").unwrap();
    writeln!(f, "c").unwrap();

    let out = rustbox_cmd()
        .args(["head", "-n", "2", p.to_str().unwrap()])
        .output()
        .expect("spawn rustbox head");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

#[test]
fn sort_sorts_lines() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("s.txt");
    fs::write(&p, "banana\napple\n").unwrap();

    let out = rustbox_cmd()
        .args(["sort", p.to_str().unwrap()])
        .output()
        .expect("spawn rustbox sort");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "apple\nbanana\n");
}

#[test]
fn grep_finds_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("g.txt");
    fs::write(&p, "foo\nbar foo\nbaz\n").unwrap();

    let out = rustbox_cmd()
        .args(["grep", "foo", p.to_str().unwrap()])
        .output()
        .expect("spawn rustbox grep");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("foo"));
    assert!(stdout.contains("bar foo"));
    assert!(!stdout.lines().any(|l| l == "baz"));
}
