---
id: 162
uid: 5dfc0bd2-aa69-48e1-86ac-1d56e8c117e8
title: E0412 leaves version names such as Billing v1 and Billing v2 alone
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-26
priority: p1
risk: low
surface: diagnostics
area: check
effort: s
related:
- 153
---

## Problem

`Billing v1` written once beside `Billing v2` written twice gives E0412 with a
safe fix that rewrites `v1` to `v2`. The sibling rule covers a final word that
is a number or one letter (`Service A`, `Deploy #2`), not a version such as
`v1`. A benchmark task (item 41) had to rename its things to avoid it, and a
model that writes versioned names will have its file changed by `fix`.

## Proposal

Treat a final word of the form `v` and digits as a sibling, as a number is.

## Acceptance criteria

- [ ] `New API -> Billing v2`, `Old API -> Billing v1` and a second mention of `Billing v2` give no E0412.
- [ ] `Billing` and `Biling` still give E0412.

## 2026-11-26

Triage: moved to v0.1 at p1. The card tells an agent to apply every fix that check returns. A safe fix that rewrites Billing v1 to Billing v2 changes a correct file, so the first release must not ship it.
