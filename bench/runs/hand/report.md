# Benchmark report: hand

100 tasks. 14 have a reply. A task with no reply scores as a failure.

| Score | Tasks |
| --- | ---: |
| Valid on the first try | 7 of 100 |
| Valid after one `fix` | 11 of 100 |
| Valid after one `fix`, with every expected fact | 10 of 100 |

## The diagnostics that replies hit most often

Counted on the first try. "Left" counts the replies that still have the code after one `fix`.

| Code | Replies | Times | Left | Meaning |
| --- | ---: | ---: | ---: | --- |
| `N0101` | 2 | 3 | 0 | A line is written the way Mermaid writes it: `-->`, quotes that a name does not need, a space before the colon of a label, or a `;` at the end. The fix writes the canonical form. |
| `E0004` | 2 | 2 | 0 | A name, a value or a label holds a character that only a text in double quotes can hold. The fix adds the quotes. A bare text can hold a double quote, but a text in double quotes cannot. So a text that holds a double quote and also needs quotes gets no fix, and the message says to remove the double quote or the character that needs the quotes. |
| `W0107` | 1 | 2 | 1 | A thing has no link and no rule, so it stands alone in every picture. A link or a rule that names its group, at any depth, counts as its own. A group needs no link of its own. |
| `E0008` | 1 | 1 | 1 | A `show` line is not `show` and a lens, with `of`, `from` or `by` and a name after it. |
| `E0009` | 1 | 1 | 1 | A fact stands outside braces, or a line mixes names and facts. |
| `E0300` | 1 | 1 | 0 | A `show` line names a lens that does not exist. The message lists the lenses, and a lens one edit from a known lens gets a safe fix. |
| `E0412` | 1 | 1 | 0 | A name is written once and is one edit from a name written more often, so it is likely a typo that makes a new thing. The safe fix writes the other name. When more than one such name is one edit away, the message names each and there is no fix. Two names are not a typo when they differ only in a final word that is a number or one letter (`Service A` and `Service B`), or when both are shorter than three characters. |
| `W0010` | 1 | 1 | 1 | A story is parsed, and this version does not order its links. |
| `W0111` | 1 | 1 | 0 | A bare name ends the line and ends in `?` with no space, as in `A -> Archive?`. Only a `?` on its own marks a guess, so the line states a firm link to a thing named `Archive?`. The safe fix adds the space and makes the link a guess. A name that must end in `?` is written in double quotes, as in `"Archive?"`. When the file writes the same name with its `?` where the `?` cannot be a guess, as in `Archive? -> Store` or `"Archive?"`, the name is a name and there is no warning. |
| `N0200` | 0 | 0 | 1 | A link or a rule ends in `?` in every statement, so it is a guess. |

## Each task

"Facts" counts the facts of the checker that the file states after one `fix`.

| Task | Title | First try | After `fix` | Facts | Diagnostics on the first try | First missing fact |
| --- | --- | --- | --- | ---: | --- | --- |
| 001 | A web shop's checkout | valid | valid | 8/8 |  |  |
| 002 | Signing in | valid | valid | 6/6 |  |  |
| 003 | A static blog | no reply | no reply | | | |
| 004 | Two apps and one cache | valid | valid | 5/5 | N0101 |  |
| 005 | Logs from three services | no reply | no reply | | | |
| 006 | Sending an email | no reply | no reply | | | |
| 007 | A search box | no reply | no reply | | | |
| 008 | Sensors on a farm | no reply | no reply | | | |
| 009 | One service, three providers | no reply | no reply | | | |
| 010 | A build pipeline | no reply | no reply | | | |
| 011 | A chat app | no reply | no reply | | | |
| 012 | A weather station | no reply | no reply | | | |
| 013 | Resolving a name | no reply | no reply | | | |
| 014 | Uploading a video | no reply | no reply | | | |
| 015 | A library loan | no reply | no reply | | | |
| 016 | A restaurant kitchen | no reply | no reply | | | |
| 017 | Receiving a webhook | no reply | no reply | | | |
| 018 | Offline sync | no reply | no reply | | | |
| 019 | A multiplayer game | no reply | no reply | | | |
| 020 | A nightly data load | no reply | no reply | | | |
| 021 | Printing a document | no reply | no reply | | | |
| 022 | Booking a ride | no reply | no reply | | | |
| 023 | Feature flags | no reply | no reply | | | |
| 024 | Recommendations | no reply | no reply | | | |
| 025 | Backups | no reply | no reply | | | |
| 026 | An online poll | no reply | no reply | | | |
| 027 | A hospital admission | no reply | no reply | | | |
| 028 | A web crawler | no reply | no reply | | | |
| 029 | Monitoring | no reply | no reply | | | |
| 030 | Services in a group | invalid | valid | 8/8 | E0412 |  |
| 031 | A private network | no reply | no reply | | | |
| 032 | Two regions | no reply | no reply | | | |
| 033 | Frontend and backend | no reply | no reply | | | |
| 034 | A group inside a group | no reply | no reply | | | |
| 035 | A cluster | no reply | no reply | | | |
| 036 | A smart home | no reply | no reply | | | |
| 037 | Four services | no reply | no reply | | | |
| 038 | Office network | no reply | no reply | | | |
| 039 | A compiler | no reply | no reply | | | |
| 040 | Who owns what | invalid | invalid | 6/8 | E0009 | `Orders { owner: Commerce }` |
| 041 | Seams between teams | no reply | no reply | | | |
| 042 | Service tiers | no reply | no reply | | | |
| 043 | Languages of services | no reply | no reply | | | |
| 044 | A group with an owner | no reply | no reply | | | |
| 045 | Versions | no reply | no reply | | | |
| 046 | Where things run | no reply | no reply | | | |
| 047 | Five services and three teams | no reply | no reply | | | |
| 048 | Parts with facts | no reply | no reply | | | |
| 049 | What may fail | no reply | no reply | | | |
| 050 | Who to call | no reply | no reply | | | |
| 051 | No direct database access | no reply | no reply | | | |
| 052 | A rule with a reason | no reply | no reply | | | |
| 053 | Two apps, one rule | no reply | no reply | | | |
| 054 | One link is a guess | valid | valid | 5/5 | W0111 |  |
| 055 | A guessed link with a label | no reply | no reply | | | |
| 056 | A guess and a rule | no reply | no reply | | | |
| 057 | Legacy calls | no reply | no reply | | | |
| 058 | Layered architecture | no reply | no reply | | | |
| 059 | Who reads the topic? | no reply | no reply | | | |
| 060 | Personal data rules | no reply | no reply | | | |
| 061 | The checkout story | invalid | invalid | 7/8 | W0010 E0008 | `show sequence of checkout` |
| 062 | The login story | no reply | no reply | | | |
| 063 | A refund in lanes | no reply | no reply | | | |
| 064 | A deploy | no reply | no reply | | | |
| 065 | A story and a flow | no reply | no reply | | | |
| 066 | Placing an order at a cafe | no reply | no reply | | | |
| 067 | Two stories | no reply | no reply | | | |
| 068 | OAuth | no reply | no reply | | | |
| 069 | An incident | no reply | no reply | | | |
| 070 | A story with a guess | no reply | no reply | | | |
| 071 | A story across groups | no reply | no reply | | | |
| 072 | A story and a rule | no reply | no reply | | | |
| 073 | A name with a colon | invalid | valid | 3/3 | E0004 |  |
| 074 | A name with a comma | no reply | no reply | | | |
| 075 | A port in a fact | invalid | valid | 4/4 | E0004 |  |
| 076 | A list in a fact | no reply | no reply | | | |
| 077 | An arrow in a name | no reply | no reply | | | |
| 078 | An at sign in a name | no reply | no reply | | | |
| 079 | Braces in a name | no reply | no reply | | | |
| 080 | Some names need quotes | no reply | no reply | | | |
| 081 | A time in a fact | no reply | no reply | | | |
| 082 | A group whose name has a colon | no reply | no reply | | | |
| 083 | No quotes needed | valid | valid | 5/5 | N0101 |  |
| 084 | A URL in a fact | no reply | no reply | | | |
| 085 | An email in a fact | no reply | no reply | | | |
| 086 | One link with evidence | valid | valid | 3/3 |  |  |
| 087 | Two links with evidence | no reply | no reply | | | |
| 088 | Evidence and a guess | no reply | no reply | | | |
| 089 | A cited rule | no reply | no reply | | | |
| 090 | A traced call path | no reply | no reply | | | |
| 091 | A matrix | invalid | valid | 7/7 | E0300 |  |
| 092 | Rings around a thing | no reply | no reply | | | |
| 093 | Two lenses | no reply | no reply | | | |
| 094 | A whole shop | valid | valid | 21/23 |  | `Web app -x Postgres: never directly` |
| 095 | A data platform | no reply | no reply | | | |
| 096 | A bank transfer | no reply | no reply | | | |
| 097 | Quotes and owners | no reply | no reply | | | |
| 098 | A game backend | no reply | no reply | | | |
| 099 | Hospital teams | no reply | no reply | | | |
| 100 | A model in production | invalid | invalid | 0/13 | W0107 | `thing Feature store` |
