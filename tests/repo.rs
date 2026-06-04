use std::io::Write as _;
use std::process::{Command, Stdio};

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn run_nub(args: &[&str], envs: &[(&str, String)], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nub"))
        .args(args)
        .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn nub");
    if !stdin.is_empty() {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin)
            .expect("write stdin");
    }
    child.wait_with_output().expect("failed to wait")
}

fn stub_env(api_url: &str) -> (tempfile::TempDir, Vec<(&'static str, String)>) {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::write(
        dir.path().join("config.toml"),
        format!(
            "default_host = \"api.nubster.com\"\n\
             [hosts.\"api.nubster.com\"]\n\
             api_url = \"{api_url}\"\n\
             git_host = \"git.nubster.com\"\n"
        ),
    )
    .expect("write config");
    let envs = vec![
        ("NUB_CONFIG_DIR", dir.path().to_string_lossy().into_owned()),
        ("NUB_NO_KEYCHAIN", "1".to_owned()),
    ];
    (dir, envs)
}

/// Serializable body mirroring the platform Repository resource.
#[derive(serde::Serialize)]
struct RepoBody<'a> {
    id: &'a str,
    name: &'a str,
    full_name: &'a str,
    description: Option<&'a str>,
    visibility: &'a str,
    clone_url: &'a str,
    ssh_url: &'a str,
    created_at: &'a str,
}

fn fake_repo(name: &str) -> RepoBody<'_> {
    RepoBody {
        id: "abc123",
        name,
        full_name: "ns/test-repo",
        description: None,
        visibility: "private",
        clone_url: "https://git.nubster.com/ns/test-repo.git",
        ssh_url: "ssh://git@git.nubster.com/ns/test-repo.git",
        created_at: "2026-06-04T00:00:00Z",
    }
}

#[tokio::test]
async fn repo_create_posts_and_prints_full_name() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/scm/repos"))
        .and(header("authorization", "Bearer test-pat"))
        .respond_with(ResponseTemplate::new(201).set_body_json(fake_repo("test-repo")))
        .expect(1)
        .mount(&server)
        .await;

    let (dir, envs) = stub_env(&server.uri());
    let login = run_nub(&["auth", "login", "--with-token"], &envs, b"test-pat");
    assert!(login.status.success(), "login failed");

    let output = run_nub(&["repo", "create", "--name", "test-repo"], &envs, &[]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    assert!(stdout.contains("ns/test-repo"), "stdout: {stdout}");
    drop(dir);
}

#[tokio::test]
async fn repo_list_prints_all_repos() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/scm/repos"))
        .and(header("authorization", "Bearer test-pat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(&[fake_repo("alpha"), fake_repo("beta")]),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (dir, envs) = stub_env(&server.uri());
    let login = run_nub(&["auth", "login", "--with-token"], &envs, b"test-pat");
    assert!(login.status.success(), "login failed");

    let output = run_nub(&["repo", "list"], &envs, &[]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    assert!(stdout.contains("ns/test-repo"), "stdout: {stdout}");
    drop(dir);
}

#[tokio::test]
async fn repo_view_prints_detail() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/scm/repos/test-repo"))
        .and(header("authorization", "Bearer test-pat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fake_repo("test-repo")))
        .expect(1)
        .mount(&server)
        .await;

    let (dir, envs) = stub_env(&server.uri());
    let login = run_nub(&["auth", "login", "--with-token"], &envs, b"test-pat");
    assert!(login.status.success(), "login failed");

    let output = run_nub(&["repo", "view", "test-repo"], &envs, &[]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    assert!(
        stdout.contains("https://git.nubster.com/ns/test-repo.git"),
        "stdout: {stdout}"
    );
    drop(dir);
}

#[tokio::test]
async fn repo_create_exits_not_authenticated_when_no_token() {
    let (dir, envs) = stub_env("https://api.example.com");
    let output = run_nub(&["repo", "create", "--name", "foo"], &envs, &[]);
    assert_eq!(
        output.status.code(),
        Some(4),
        "expected NotAuthenticated (4)"
    );
    drop(dir);
}
