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
    fake_repo_with_urls(
        name,
        "https://git.nubster.com/ns/test-repo.git",
        "ssh://git@git.nubster.com/ns/test-repo.git",
    )
}

fn fake_repo_with_urls<'a>(name: &'a str, clone_url: &'a str, ssh_url: &'a str) -> RepoBody<'a> {
    RepoBody {
        id: "abc123",
        name,
        full_name: "ns/test-repo",
        description: None,
        visibility: "private",
        clone_url,
        ssh_url,
        created_at: "2026-06-04T00:00:00Z",
    }
}

/// Environment that points git at a throwaway global config so tests never
/// read or pollute the developer's `~/.gitconfig`.
fn git_isolation_env(dir: &std::path::Path) -> Vec<(&'static str, String)> {
    vec![
        (
            "GIT_CONFIG_GLOBAL",
            dir.join("gitconfig").to_string_lossy().into_owned(),
        ),
        ("GIT_CONFIG_NOSYSTEM", "1".to_owned()),
    ]
}

/// Creates a bare repository under `path` and returns its `file://` URL,
/// letting tests exercise a real `git clone` without network or credentials.
fn init_bare_repo(path: &std::path::Path) -> String {
    let status = Command::new("git")
        .args(["init", "--bare", "--quiet"])
        .arg(path)
        .status()
        .expect("run git init");
    assert!(status.success(), "git init --bare failed");
    file_url(path)
}

fn file_url(path: &std::path::Path) -> String {
    let p = path.to_string_lossy().replace('\\', "/");
    if p.starts_with('/') {
        format!("file://{p}")
    } else {
        format!("file:///{p}")
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

/// Mounts a `view` mock answering with the given clone and ssh URLs, and
/// returns a logged-in environment with git isolation applied.
async fn clone_fixture(
    server: &MockServer,
    clone_url: &str,
    ssh_url: &str,
) -> (tempfile::TempDir, Vec<(&'static str, String)>) {
    Mock::given(method("GET"))
        .and(path("/scm/repos/test-repo"))
        .and(header("authorization", "Bearer test-pat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fake_repo_with_urls(
                "test-repo",
                clone_url,
                ssh_url,
            )),
        )
        .expect(1)
        .mount(server)
        .await;

    let (dir, mut envs) = stub_env(&server.uri());
    envs.extend(git_isolation_env(dir.path()));
    let login = run_nub(&["auth", "login", "--with-token"], &envs, b"test-pat");
    assert!(login.status.success(), "login failed");
    (dir, envs)
}

#[tokio::test]
async fn repo_clone_clones_into_positional_dest() {
    let server = MockServer::start().await;
    let work = tempfile::TempDir::new().expect("temp dir");
    let clone_url = init_bare_repo(&work.path().join("origin.git"));
    let (dir, envs) = clone_fixture(&server, &clone_url, "ssh://unused.invalid/x.git").await;

    let dest = work.path().join("checkout");
    let dest_arg = dest.to_string_lossy().into_owned();
    let output = run_nub(&["repo", "clone", "test-repo", &dest_arg], &envs, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dest.join(".git").is_dir(), "clone destination missing .git");
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    assert!(stdout.contains("Cloned ns/test-repo"), "stdout: {stdout}");
    drop(dir);
}

#[tokio::test]
async fn repo_clone_accepts_dest_flag() {
    let server = MockServer::start().await;
    let work = tempfile::TempDir::new().expect("temp dir");
    let clone_url = init_bare_repo(&work.path().join("origin.git"));
    let (dir, envs) = clone_fixture(&server, &clone_url, "ssh://unused.invalid/x.git").await;

    let dest = work.path().join("checkout-flag");
    let dest_arg = dest.to_string_lossy().into_owned();
    let output = run_nub(
        &["repo", "clone", "test-repo", "--dest", &dest_arg],
        &envs,
        &[],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dest.join(".git").is_dir(), "clone destination missing .git");
    drop(dir);
}

#[tokio::test]
async fn repo_clone_uses_ssh_url_with_ssh_flag() {
    let server = MockServer::start().await;
    let work = tempfile::TempDir::new().expect("temp dir");
    let ssh_url = init_bare_repo(&work.path().join("origin.git"));
    let (dir, envs) = clone_fixture(
        &server,
        "https://git.nubster.com/ns/test-repo.git",
        &ssh_url,
    )
    .await;

    let dest = work.path().join("checkout-ssh");
    let dest_arg = dest.to_string_lossy().into_owned();
    let output = run_nub(
        &["repo", "clone", "test-repo", "--ssh", &dest_arg],
        &envs,
        &[],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dest.join(".git").is_dir(), "clone destination missing .git");
    drop(dir);
}

#[tokio::test]
async fn repo_clone_guides_when_helper_missing() {
    let server = MockServer::start().await;
    let (dir, envs) = clone_fixture(
        &server,
        "https://git.nubster.com/ns/test-repo.git",
        "ssh://git@git.nubster.com/ns/test-repo.git",
    )
    .await;

    let output = run_nub(&["repo", "clone", "test-repo"], &envs, &[]);
    assert_eq!(output.status.code(), Some(1), "expected guided error (1)");
    let stderr = String::from_utf8(output.stderr).expect("non-utf8 stderr");
    assert!(
        stderr.contains("nub auth setup-git"),
        "stderr should guide towards setup-git: {stderr}"
    );
    drop(dir);
}

#[tokio::test]
async fn repo_clone_fails_with_git_exit_code() {
    let server = MockServer::start().await;
    let work = tempfile::TempDir::new().expect("temp dir");
    let missing = file_url(&work.path().join("missing.git"));
    let (dir, envs) = clone_fixture(&server, &missing, "ssh://unused.invalid/x.git").await;

    let dest_arg = work
        .path()
        .join("never-created")
        .to_string_lossy()
        .into_owned();
    let output = run_nub(&["repo", "clone", "test-repo", &dest_arg], &envs, &[]);
    assert_eq!(output.status.code(), Some(7), "expected GitCommand (7)");
    drop(dir);
}

#[tokio::test]
async fn repo_clone_exits_not_authenticated_when_no_token() {
    let (dir, envs) = stub_env("https://api.example.com");
    let output = run_nub(&["repo", "clone", "foo"], &envs, &[]);
    assert_eq!(
        output.status.code(),
        Some(4),
        "expected NotAuthenticated (4)"
    );
    drop(dir);
}
