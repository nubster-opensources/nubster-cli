# Explanation

This section covers architectural background and design decisions for `nub`.

## How nub talks to the platform

`nub` is a thin CLI client. It communicates with the Nubster platform over its HTTP API using
the active host configured in `~/.config/nub/config.toml` (or overridden with `--host`).

Authentication tokens are stored in the OS keychain via the `keyring` crate. On Linux, this
requires D-Bus and a keyring daemon (for example, GNOME Keyring or KWallet). On macOS and
Windows the native keychain is used directly. Set `NUB_NO_KEYCHAIN=1` to store the token in a
plain file instead.

## Git credential helper integration

Running `nub auth setup-git` writes a `credential.helper = nub auth git-credential` entry to
the global git configuration. Git then calls `nub auth git-credential get` before every network
operation. `nub` reads the stored token and prints it in the credential protocol format git
expects. This mechanism means you never need to manage tokens inside git itself.

## Command structure

```
nub
 auth
   login         store a personal access token
   logout        remove stored credentials
   status        check whether a credential is available
   setup-git     register nub as the git credential helper
   git-credential  git credential protocol entry point (called by git)
 repo
   create        create a new repository on the platform
   list          list accessible repositories
   view          inspect a single repository
   clone         clone a repository (HTTPS or SSH)
 config
   path          print the configuration file path
   show          print the resolved configuration
```

## Available explanations

- [Roadmap](roadmap.md)
