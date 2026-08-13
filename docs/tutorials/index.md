# Getting started

This tutorial walks you through installing `nub`, authenticating against a Nubster instance,
and cloning your first repository.

## Prerequisites

- Rust 1.89 or later (for building from source).
- A running Nubster instance and a personal access token issued from its web interface.

## Step 1: build and install nub

Clone the repository and build the release binary:

```sh
git clone https://github.com/nubster-opensources/nubster-cli.git
cd nubster-cli
cargo build --release
```

Copy the binary to a directory on your `PATH`, for example:

```sh
cp target/release/nub ~/.local/bin/
```

Verify the installation:

```sh
nub --version
```

## Step 2: authenticate

Obtain a personal access token from your Nubster instance, then run:

```sh
nub auth login --with-token
```

Paste your token when prompted. `nub` stores it in the OS keychain (or in a file when
`NUB_NO_KEYCHAIN=1` is set).

Check that the credential was stored:

```sh
nub auth status
```

You should see a confirmation that a token is available for the active host.

## Step 3: register the git credential helper

Allow git to use your stored token transparently:

```sh
nub auth setup-git
```

This adds `nub` as the credential helper in your global git configuration.

## Step 4: clone a repository

```sh
nub repo clone my-repo
```

The repository is cloned over HTTPS. Pass `--ssh` to use SSH instead:

```sh
nub repo clone my-repo --ssh
```

You are now set up. See [How-to guides](../how-to/index.md) for more task-focused recipes.
