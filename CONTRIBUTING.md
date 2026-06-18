# Contributing to nub

Thank you for considering a contribution to nub. This document explains how to get involved.

## Conventions

nub follows the Nubster general coding standards documented in [nubster-docs](https://github.com/nubster-opensources/nubster-docs/tree/main/docs/reference/coding-standards). In short:

- **Trunk-Based Development** - feature branches `feature/<issue>-<slug>` from `main`, never commit directly on `main`.
- **Conventional Commits** - all commit messages follow the `type(scope): description` format, enforced by `cog verify` in the commit-msg hook.
- **Rust style** - lints `clippy::all` and `clippy::pedantic` set to `deny`, MSRV pinned in `rust-toolchain.toml` and `Cargo.toml`.
- **No competitor mentions** - the source code, commit messages, pull requests and documentation never name competing tools.
- **English on the public API** - rustdoc comments and public-facing documentation are written in English.

## Local setup

1. Install Rust via [rustup](https://rustup.rs/). The `rust-toolchain.toml` file at the root pins the exact version automatically.
2. Install [lefthook](https://github.com/evilmartians/lefthook) and run `lefthook install` to wire up the pre-commit hooks.
3. Verify your setup: `cargo test --all-features`.

## Discussion before code

Open a discussion or issue before writing code for any non-trivial change. This avoids wasted effort and keeps the design aligned with the project roadmap.

## Contributor License Agreement

Contributions to this project are governed by the Nubster Contributor License Agreement, hosted at [github.com/nubster-opensources/cla](https://github.com/nubster-opensources/cla).

On your first pull request, the CLA Assistant bot will automatically prompt you to sign the CLA. Once signed, your signature applies to all current and future contributions to any `nubster-opensources` project.

The CLA is a license grant (not a copyright assignment): you keep the copyright on your contributions and grant Nubster a broad license to use, sub-license, and re-license them.

## License

By contributing to nub, you agree that your contributions will be licensed under the terms described in the [License](#license) section of the README and the `LICENSE-MIT` / `LICENSE-APACHE` files at the root of this repository.
