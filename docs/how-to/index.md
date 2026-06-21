# How-to guides

Task-focused recipes for common `nub` operations.

## Authenticate against a Nubster instance

```sh
nub auth login --with-token
```

Reads a personal access token from stdin and stores it in the OS keychain. Set
`NUB_NO_KEYCHAIN=1` to fall back to a plain file if the keychain is unavailable.

To remove stored credentials:

```sh
nub auth logout
```

To verify that a credential is available:

```sh
nub auth status
```

## Configure the git credential helper

Register `nub` as the credential helper in your global git configuration:

```sh
nub auth setup-git
```

After this step, git commands (`clone`, `push`, `fetch`) pick up your stored token
automatically without prompting.

## Clone a repository

```sh
nub repo clone <NAME>
```

`nub repo clone` requires the git credential helper to be configured first. Run
`nub auth setup-git` if you have not done so.

Clone over SSH instead of HTTPS:

```sh
nub repo clone <NAME> --ssh
```

## List accessible repositories

```sh
nub repo list
```

Print the list as machine-readable JSON:

```sh
nub repo list --json
```

## Inspect a single repository

```sh
nub repo view <NAME>
```

Displays clone URLs, visibility, and description.

## Check the active configuration

Print the path to the configuration file:

```sh
nub config path
```

Print the resolved configuration and the effective host:

```sh
nub config show
```

## Override the target host for a single command

Every command accepts a global `--host` flag:

```sh
nub repo list --host https://nubster.example.com
```
