# Release process

`nub` is a binary tool. It is **not** published to crates.io (`publish = false` in `Cargo.toml`).
The release process covers versioning, tagging, and changelog maintenance only.
Distribution via package managers is a separate effort tracked in issue #33.

## Versioning

`nub` follows [Semantic Versioning](SEMVER_POLICY.md). The version is declared once in the
workspace root `Cargo.toml` and propagated to all workspace members.

## Release steps

1. **Create a release branch** from `main`:
   ```sh
   git fetch origin main:main
   git checkout -b chore/release-x.y.z main
   ```

2. **Bump the version** in the workspace root `Cargo.toml` (`[workspace.package] version`).

3. **Update `CHANGELOG.md`**: move entries from `[Unreleased]` to a new section headed
   `[x.y.z] - YYYY-MM-DD`. Add a comparison link at the bottom.

4. **Commit**:
   ```sh
   git commit -m "chore: release x.y.z"
   ```

5. **Open a pull request** targeting `main` and merge it once CI is green.

6. **Tag** the merge commit:
   ```sh
   git tag -a vx.y.z -m "Release x.y.z"
   git push origin vx.y.z
   ```

7. **Create a GitHub Release** from the tag, copying the relevant CHANGELOG section into the
   release notes.

## What this process does NOT do

- There is no `cargo publish` step: `nub` is not on crates.io.
- There is no automated bump script: the version is edited by hand.
- Distribution artifacts (installers, package manager packages) are not produced yet.
  See issue #33.

## Milestones

Each release corresponds to a milestone on the issue tracker. Close the milestone once the
tag is pushed and the GitHub Release is created.
