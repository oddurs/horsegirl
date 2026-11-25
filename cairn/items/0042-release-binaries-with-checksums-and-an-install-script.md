---
id: 42
uid: cbd0deff-6359-4de2-9677-1c61f2cf0db4
title: Release binaries with checksums and an install script
type: chore
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
depends_on:
- 20
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: release
track:
- release
- workflow
---

## Purpose

There is no way to install horsegirl without a Rust toolchain.

## Approach

On a version tag, build static binaries for Linux (x86_64, aarch64) and macOS (x86_64, aarch64), publish them with `SHA256SUMS` and a build attestation, and provide an `install.sh` that verifies the checksum.

## Acceptance criteria

- [ ] A tag on a test repository or a pre-release tag produces all four archives.
- [x] `install.sh` installs to `~/.local/bin` and fails on a checksum mismatch.
- [x] The README install section uses the script.

## 2026-11-25

Criterion 2: sh tests/install.sh exits 0 (run by scripts/task test, which passed). It installs a fake release from a file:// directory to HOME/.local/bin, honours HORSEGIRL_INSTALL_DIR, and with every SHA256SUMS hash set to zeros exits 1 with 'checksum mismatch' and installs nothing; with no line for the archive it exits 1 with 'no checksum'. With the comparison in install.sh disabled, the test failed: 'the install passed with a checksum mismatch'. End to end: a real x86_64-unknown-linux-musl archive packed as the workflow packs it installed through install.sh, and the installed binary printed 'horsegirl 0.0.0'. file reported 'static-pie linked'.

## 2026-11-25

Criterion 3: README.md Install section now leads with curl -fsSL https://raw.githubusercontent.com/oddurs/horsegirl/main/install.sh | sh and keeps cargo install as the build-from-source path. scripts/task check exited 0. actionlint 1.7.12 on release.yml exited 0.

## 2026-11-25

Criterion 1 stays unticked: it needs a real tag, and this work pushes no tag. Dry-run evidence: gh workflow run release.yml --ref chore/0042-release-binaries-with-checksums-and-an-i -f dry-run=true gave run 45626307500, conclusion success. check, the four build jobs and sums passed; publish was skipped. The release artifact held the four archives, SHA256SUMS and install.sh; sha256sum -c SHA256SUMS printed OK for each archive; install.sh installed the x86_64 Linux archive from that artifact and the binary printed 'horsegirl 0.0.0'. The first dry run (45625754373) failed because file printed 'statically linked' for aarch64 musl, which the old pattern did not match; the step now checks for an INTERP program header. The publish job (attestation, gh release create) is untested until the first pre-release tag.
