---
id: 20
uid: 25df5bc6-8825-49cd-bf99-3ded6adaa625
title: Is the name free on the package registries?
type: spike
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p1
risk: low
surface: none
effort: s
area: release
track:
- release
---

## Question

Is the name horsegirl free on crates.io, npm and Homebrew, and as a GitHub organisation?

## Why it matters

Package names appear in every install command and cannot change after people use them.

## Time box

Half a day.

## Answer

Conclusion: use `horsegirl` on crates.io, npm and Homebrew. Do not use `horsegirl` as the GitHub organisation name. A GitHub user holds it. Keep the repository under `oddurs`, or create an organisation named `horsegirl-lang`.

All checks were read-only HTTP requests on 2026-11-25. Nothing was registered.

| Registry | Name checked | Result | Free? | Fallback |
| --- | --- | --- | --- | --- |
| crates.io | `horsegirl` | `GET /api/v1/crates/horsegirl` returned 404 | yes | `horsegirl-cli` (404) |
| npm | `horsegirl` | `GET registry.npmjs.org/horsegirl` returned 404 | yes | `horsegirl-cli` (404) |
| npm scope | `@horsegirl/cli` | registry returned 404 for the package. The scope itself could not be read (the npm site returned 403), so it is not confirmed | not confirmed | `horsegirl-cli` |
| Homebrew core | `horsegirl` | `Formula/h/horsegirl.rb` in Homebrew/homebrew-core returned 404 | yes | `oddurs/tap` (a tap needs no free name) |
| Homebrew cask | `horsegirl` | `Casks/h/horsegirl.rb` in Homebrew/homebrew-cask returned 404 | yes | none needed |
| GitHub organisation | `horsegirl` | `/orgs/horsegirl` returned 404, but `/users/horsegirl` returned 200 | no | `horsegirl-lang` (404 as user and as org) |
| PyPI (extra) | `horsegirl` | 404 | yes | none needed |

Notes.

- GitHub shares one namespace between users and organisations. The account `horsegirl` is a user, created 2014-10-08, with 0 public repositories and 0 followers. It looks unused, but GitHub does not release a name on request without a trademark claim. Do not plan on it.
- The Homebrew API site (formulae.brew.sh) returned 503 on each try. The check used the homebrew-core and homebrew-cask repositories instead. A 404 there means no formula or cask with that file name. It does not cover an alias or a renamed formula.
- A free name today does not hold. Item 0115 reserves the crate and the npm package.
- The install commands in item 0096 can keep the name `horsegirl` for `cargo install`, `npm install` and `brew install`.

## Acceptance criteria

- [x] The Answer section states a conclusion and the evidence for it.
- [x] Follow-up items exist for the work the answer implies.
- [x] The answer lists each registry, whether the name is free, and the fallback name where it is not.

## 2026-11-25

Ran curl GET on crates.io API, registry.npmjs.org, api.github.com /orgs and /users, homebrew-core and homebrew-cask contents API. crates.io 404, npm 404, core 404, cask 404, /orgs/horsegirl 404, /users/horsegirl 200 (type User). formulae.brew.sh returned 503. Fallback names horsegirl-cli and horsegirl-lang returned 404. Filed 0115 for the reservation.
