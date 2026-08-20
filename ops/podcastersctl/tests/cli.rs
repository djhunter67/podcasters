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
