# Semantic versioning policy

`nub` follows [Semantic Versioning 2.0.0](https://semver.org/).

## 0.x phase (pre-1.0)

During the 0.x phase, the **minor** version carries the same weight as the major version in a
stable release:

- A **patch** bump (`0.1.0` -> `0.1.1`) is for bug fixes that do not change observable
  behaviour from the caller's perspective.
- A **minor** bump (`0.1.0` -> `0.2.0`) is for new commands, new flags, or any change that
  a script relying on `nub` output might need to adapt to.

Breaking changes are allowed in minor bumps during the 0.x phase. They are always documented
in `CHANGELOG.md` under a `### Changed` or `### Removed` heading.

## 1.0 and beyond

From 1.0.0 onward, the standard SemVer contract applies:

- **Patch**: backward-compatible bug fixes.
- **Minor**: backward-compatible new features.
- **Major**: breaking changes.

## Public API definition

The versioned surface for `nub` is the **CLI surface**:

- Command names and sub-command names.
- Flag and argument names (long form, e.g. `--json`, `--host`).
- The JSON schema of `--json` output on all commands that support it.
- Exit codes documented in the manual pages.

The following are **not** part of the public API:

- Internal Rust types exposed by the `nub` library crate. The library surface is
  `publish = false` and has no stability guarantee.
- Human-readable text output (non-JSON): phrasing and formatting may change in any release.
- Short flags (single-character, e.g. `-j`): reserved for future use without a major bump.

## MSRV

An MSRV bump is treated as a **minor** version bump. See [MSRV policy](MSRV_POLICY.md).

## Deprecation cycle

Before removing or renaming a command or flag, it is marked deprecated in the help text for
at least one minor release. The deprecation and the subsequent removal are each documented in
`CHANGELOG.md`.

## Breaking change documentation

Every breaking change must appear in `CHANGELOG.md` under `### Changed` or `### Removed` with
a concise migration note.
