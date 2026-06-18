# nub

> Unified command-line client for the Nubster developer platform, starting with repository management.

[![CI](https://github.com/nubster-opensources/nubster-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/nubster-opensources/nubster-cli/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](./docs/MSRV_POLICY.md)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![Status](https://img.shields.io/badge/status-alpha-orange)](#status)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)

`nub` is the official CLI for the Nubster platform. It provides authentication,
repository operations, and a git credential helper, all through a single binary
with both human-readable and machine-readable (`--json`) output.

nub is sponsored by [Nubster](https://nubster.com).

## Status

**Alpha.** The command set and output format are not yet stable. Breaking changes
may occur on minor version bumps until v1.0.

## Quick start

```sh
# Authenticate with your personal access token
echo "<your-token>" | nub auth login --with-token

# Create a repository
nub repo create --name my-project

# List repositories
nub repo list

# Clone a repository
nub repo clone my-project

# Register nub as a git credential helper
nub auth setup-git
```

Global flags available on every command:

| Flag | Effect |
|------|--------|
| `--host <HOST>` | Override the platform host for this invocation |
| `--json` | Emit machine-readable JSON instead of human output |
| `--no-color` | Disable ANSI color output |

## Why nub

Managing repositories, authentication, and git integration through three separate
tools is friction. `nub` collapses the common workflows for the Nubster platform
into a single binary: log in once, clone, create, and list repositories, and let
the built-in git credential helper handle token refresh transparently.

## What nub is not

- A general-purpose git wrapper. `nub` calls `git` under the hood for clone
  operations but does not replicate git commands.
- A server-side tool. `nub` is a client that talks to the Nubster control-plane
  REST API.
- Stable for scripting yet. Until v1.0, the output format and command structure
  may change on minor releases.

## Documentation

- [Tutorials](./docs/tutorials/)
- [How-to guides](./docs/how-to/)
- [Reference](./docs/reference/)
- [Explanation](./docs/explanation/)
- [Release process](./docs/RELEASE_PROCESS.md)
- [Semantic versioning policy](./docs/SEMVER_POLICY.md)
- [MSRV policy](./docs/MSRV_POLICY.md)

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](./CONTRIBUTING.md) before opening a pull request.

All contributors must sign the [Nubster CLA](https://github.com/nubster-opensources/cla). The CLA Assistant bot will prompt you automatically on your first PR.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in nub by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
