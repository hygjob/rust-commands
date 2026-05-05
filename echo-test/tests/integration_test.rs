use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn echo_basic() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    let output = cmd.arg("hello").assert();

    output.success().stdout("hello\n");
}

#[test]
fn echo_multiple_args() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.args(["hello", "world"])
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn echo_no_newline() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.args(["-n", "hello"])
        .assert()
        .success()
        .stdout("hello");
}

#[test]
fn echo_with_escape() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.args(["-e", "hello\\tworld"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello\tworld"));
}

#[test]
fn echo_empty_input() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.assert()
        .success()
        .stdout("\n");
}

#[test]
fn echo_help() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("텍스트를 표준 출력에 표시합니다"));
}
