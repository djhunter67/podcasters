#![allow(clippy::unwrap_used)]
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_command_succeeds() {
    let mut command = Command::cargo_bin("podcastersctl").unwrap();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("mongo"))
        .stdout(predicate::str::contains("redis"));
}

#[test]
fn version_command_succeeds() {
    let mut command = Command::cargo_bin("podcastersctl").unwrap();

    command
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn doctor_dispatches_correctly() {
    let mut command = Command::cargo_bin("podcastersctl").unwrap();

    command
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("Toolchain:"))
        .stdout(predicate::str::is_match("Rustfmt:").unwrap())
        .stdout(predicate::str::is_match("Clippy:").unwrap())
        .stdout(predicate::str::is_match("Compiler:").unwrap());
}
