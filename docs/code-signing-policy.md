# Code signing policy

The programs and the installer are not code-signed yet, and Windows SmartScreen may warn before
the installer runs (**More info > Run anyway**). Phai Studio applied for free code signing for
open-source projects from [SignPath Foundation](https://signpath.org) in November 2026; the
Foundation asks for wider public adoption first, and Phai Studio will apply again once it has it.
Until then every installer is built from source by GitHub Actions, and anyone can check the build.

<!-- Shown once SignPath Foundation accepts the project, as its terms ask:
Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org)
-->

## What is signed

Only programs built by this project from this repository:

| File | What it is |
|---|---|
| `phai-studio.exe` | Phai Studio itself |
| `phai-setup.exe` | Phai Studio's setup and uninstall program |
| `PhaiStudio-<version>-setup.exe` | The installer handed out on the Releases page |

Each is built from source by GitHub Actions ([`.github/workflows/release.yml`](../.github/workflows/release.yml))
from a tagged commit of [kirati-ai/phai-studio](https://github.com/kirati-ai/phai-studio). Nothing
built on a person's own computer is signed. Libraries from crates.io are compiled into the programs
from their published source; no third-party binary is signed as ours.

Every signed file carries the product name **Phai Studio** and the version in `Cargo.toml`.

## Who does what

| Role | Who |
|---|---|
| Committers and reviewers | [kirati-ai](https://github.com/kirati-ai) - the project's maintainer |
| Approvers | [kirati-ai](https://github.com/kirati-ai) |

Changes from anyone else arrive as pull requests and are reviewed by a committer before they are
merged. Every signing request is approved by hand by an approver. Everyone in these roles uses
multi-factor authentication for GitHub and for SignPath.

## Privacy

This program will not transfer any information to other networked systems unless specifically
requested by the user or the person installing or operating it.

What Phai Studio sends, and only when the person asks for it:

- **Manage Crates** searches crates.io with the words typed in its search box.
- **Build** runs Cargo, which downloads the crates a project names from crates.io.
- **Git** pushes to and pulls from the remotes the person has set up.
- **Server Explorer** connects to the database servers the person adds.
- **Code Coverage** installs Rust's `llvm-tools` with rustup, after asking.
- **Help > Check for Updates** (and the button in About) asks GitHub for the newest release, and
  downloads its setup from GitHub Releases when the person says yes.
- **GitHub Copilot** is off until the person turns it on (Tools > Options > GitHub Copilot). Then
  Phai Studio downloads GitHub's Copilot Language Server from GitHub Releases once, after asking,
  checked against the SHA-256 GitHub gives for it, and signs in with GitHub's device flow (the
  sign-in is kept by GitHub's server, not in a file of Phai Studio's). While it is on, that server
  sends the open files to GitHub's Copilot service for suggestions as the person types, under
  their own GitHub account and GitHub's terms. GitHub's server may send its own usage data to
  GitHub under those terms; Phai Studio adds none.
- Links in the guide, the start window and About open in the browser when clicked.

Phai Studio collects no usage data and has no telemetry. rust-analyzer, which Phai Studio uses for
code completion, runs on the same computer.

## Installing and uninstalling

The installer says where it installs and which shortcuts it makes before it changes anything.
Phai Studio is removed with **Settings > Apps** (or **Control Panel > Programs**), which runs its
uninstaller.
