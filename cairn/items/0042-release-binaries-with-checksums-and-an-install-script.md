---
id: 42
uid: cbd0deff-6359-4de2-9677-1c61f2cf0db4
title: Release binaries with checksums and an install script
type: chore
status: planned
milestone: v0.1
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
- [ ] `install.sh` installs to `~/.local/bin` and fails on a checksum mismatch.
- [ ] The README install section uses the script.
