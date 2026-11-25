# Changelog

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `horsegirl_core::text::measure` gives the width and height of a label in
  pixels, in Arial metrics, from a table bundled in the core. It gives the same
  result on Linux, macOS and in WebAssembly. A character outside the table
  measures as one em.
