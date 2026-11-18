# Code signing policy

Phai Studio has applied for free code signing for open-source projects from
[SignPath Foundation](https://signpath.org). Until the application is accepted, the programs and
the installer are not signed, and Windows SmartScreen may warn before the installer runs
(**More info > Run anyway**).

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
- Links in the guide and the start window open in the browser when clicked.

Phai Studio collects no usage data and has no telemetry. rust-analyzer, which Phai Studio uses for
code completion, runs on the same computer.

## Installing and uninstalling

The installer says where it installs and which shortcuts it makes before it changes anything.
Phai Studio is removed with **Settings > Apps** (or **Control Panel > Programs**), which runs its
uninstaller.
