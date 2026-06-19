# nub

`nub` is the official command-line client for the Nubster platform. It lets you authenticate,
manage repositories, and configure the tool from your terminal.

## What you can do with nub

- **Authenticate** against a Nubster instance and store credentials securely in the OS keychain.
- **Clone, create, list, and inspect** repositories hosted on the platform.
- **Configure** the active host and inspect the resolved configuration.
- **Drive git** transparently: `nub` can register itself as a git credential helper so that
  every `git clone`, `git push`, and `git fetch` picks up your stored token automatically.

## Sections

| Section | Purpose |
|---------|---------|
| [Tutorials](tutorials/index.md) | Step-by-step guides for first-time users |
| [How-to guides](how-to/index.md) | Task-focused recipes for common operations |
| [Reference](reference/index.md) | Full CLI flag and sub-command reference |
| [Explanation](explanation/index.md) | Architecture notes and roadmap |

## Current status

`nub` is in active development (version 0.1.x). The binary is not yet distributed via package
managers. See [Roadmap](explanation/roadmap.md) for planned distribution work and
[Release process](RELEASE_PROCESS.md) for how releases are versioned and tagged.
