# Changelog

All notable changes to nub will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-18

### Added

- `nub auth login --with-token`: authenticate by reading a personal access token from stdin and storing it in the OS keychain (or a file fallback via `NUB_NO_KEYCHAIN`).
- `nub auth logout`: remove stored credentials for the active host.
- `nub auth status`: report whether a credential is available for the active host.
- `nub auth setup-git`: register `nub` as a git credential helper in the global git configuration.
- `nub auth git-credential`: git credential protocol entry point (invoked by git, not by users directly).
- `nub repo create --name <NAME>`: create a new repository on the platform.
- `nub repo list`: list repositories accessible to the authenticated principal.
- `nub repo view <NAME>`: show details (clone URLs, visibility, description) of a single repository.
- `nub repo clone <NAME>`: clone a repository over HTTPS or SSH (`--ssh`), with a built-in guard that requires the git credential helper to be configured first.
- `nub config path`: print the configuration file path.
- `nub config show`: print the current configuration and the effective host.
- Global flags (`--host`, `--json`, `--no-color`) accepted at every command level.
- Machine-readable JSON output mode (`--json`) on all commands that produce structured output.
- Online documentation built with mdBook and published to GitHub Pages, covering tutorials,
  how-to guides, reference, and explanation (Diataxis structure).
- Generated CLI reference (markdown) and man pages produced by `cargo xtask doc-gen`.
- Release process, semantic versioning policy, and MSRV policy documented in `docs/`.

[0.1.0]: https://github.com/nubster-opensources/nubster-cli/releases/tag/v0.1.0
