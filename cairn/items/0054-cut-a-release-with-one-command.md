---
id: 54
uid: 64cfa2c0-354e-4ac9-bdd9-0419aa2f6c5c
title: Cut a release with one command
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-27
closed_at: 2026-11-27
related:
- 42
priority: p1
risk: medium
surface: none
effort: m
area: workflow
track:
- workflow
- release
---

## Purpose

A release needs a version bump, a changelog section and a tag. Each is a chance to make a mistake.

## Approach

Add `scripts/agent release <version>`: it moves the Unreleased section of the changelog under the version, bumps the crate versions, opens a pull request, and after the merge the tag is pushed by a workflow.

## Acceptance criteria

- [x] One command and one merged pull request produce a tagged release.
- [x] A pull request that changes a public contract (`surface` is not `none`) without a changelog entry fails CI.

## 2026-11-27

Owner decision: 0054 no longer depends on 0042. The release workflow of 0042 exists and its dry run builds all four archives. The first real tag, v0.1.0, is cut with the command this item builds, and that tag closes the last criterion of 0042 as well.

## 2026-11-27

Criterion 2: on PR #97, commit b1c6662 set surface: cli on this item with no changelog entry. The changelog job (run 45956330483, job 140363537144) failed and printed: 'check-changelog: item 54 changes the cli contract (surface: cli), and the Unreleased section of CHANGELOG.md has no new entry.' The next commit restores surface: none. tests/release.sh also covers: no entry fails, an entry passes, a heading alone fails, a line outside Unreleased fails, surface none passes, a branch with no item passes, a missing item fails.

## 2026-11-27

Criterion 1: proven against a scratch clone with a bare origin, as the item instructions ask; no real tag was pushed. sh tests/release.sh printed 'tests/release.sh: ok'. It runs scripts/agent release 0.1.0-rc.1 and then 0.1.0 (one command each opens one pull request through a stand-in gh), squash-merges each as 'chore(release): v<version> (#N)', and runs scripts/release-tag, the logic of the tag job of release.yml, on a shallow clone of origin. v0.1.0 then points at the merge commit and the job outputs build=true tag=v0.1.0 for the publish job. Refusals checked: no version, '1.0', the current version, an older version, an older pre-release, a dirty tree, a branch other than main, an empty Unreleased section, an existing release branch. Removing each guard made the test fail. scripts/task check exited 0. actionlint 1.7.12 passed. The publish step itself first runs for real on v0.1.0.

## 2026-11-27

After the restore and a rebase, the changelog job on PR #97 passed (run 45958713439, job 140371175616).

## 2026-11-27

Correction to the criterion 1 note: 'Removing each guard made the test fail' was false for two guards in scripts/agent release, the check that main is the same as origin/main and the check that the tag v<version> exists on origin. tests/release.sh now has a refusal case for each, before the 0.1.0 release: an unpushed commit on main (expects 'not the same as origin/main'), and refs/tags/v0.1.0 pushed to the scratch origin (expects 'already exists on origin'). sh tests/release.sh printed 'tests/release.sh: ok'. With 'true ||' in front of the sync guard in a scratch copy, the test printed 'FAIL: a main ahead of origin/main was accepted' and exited 1. With 'true ||' in front of the tag guard, it printed 'FAIL: a version whose tag exists on origin was accepted' and exited 1.
