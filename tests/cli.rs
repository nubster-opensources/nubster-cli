use std::process::Command;

fn nub() -> Command {
    Command::new(env!("CARGO_BIN_EXE_nub"))
}

#[test]
fn auth_help_lists_all_subcommands() {
    let output = nub()
        .args(["auth", "--help"])
        .output()
        .expect("failed to run nub");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    for sub in ["login", "logout", "status", "setup-git", "git-credential"] {
        assert!(stdout.contains(sub), "auth help should list `{sub}`");
    }
}

#[test]
fn repo_help_lists_all_subcommands() {
    let output = nub()
        .args(["repo", "--help"])
        .output()
        .expect("failed to run nub");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    for sub in ["create", "clone", "list", "view"] {
        assert!(stdout.contains(sub), "repo help should list `{sub}`");
    }
}

#[test]
fn global_flags_are_accepted_at_leaf_level() {
    // Global flags must parse at any depth of the command tree; a parse
    // failure would exit with clap's usage code 2 before printing help.
    let output = nub()
        .args(["repo", "clone", "--json", "--no-color", "--help"])
        .output()
        .expect("failed to run nub");
    assert!(output.status.success(), "global flags rejected at leaf");
}

#[test]
fn clone_requires_a_repository_name() {
    let output = nub()
        .args(["repo", "clone"])
        .output()
        .expect("failed to run nub");
    assert_eq!(output.status.code(), Some(2), "expected clap usage error");
}

#[test]
fn clone_rejects_both_dest_forms() {
    let output = nub()
        .args(["repo", "clone", "ns/repo", "dir-a", "--dest", "dir-b"])
        .output()
        .expect("failed to run nub");
    assert_eq!(output.status.code(), Some(2), "expected clap usage error");
}
