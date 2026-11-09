# Project issue tracker

Use GitHub Issues in `wagar-meilech/bluetooth-discord-bot` through `gh` as the
authoritative tracker for published specs and implementation tickets:
https://github.com/wagar-meilech/bluetooth-discord-bot/issues.

`docs/WORK.md#Tasks` is the maintained local plan and working projection for
Zoetrope. Keep issue links, status, actual owner, declared dependencies, and next
actions current after meaningful GitHub changes. Local setup/planning rows may
exist before a GitHub issue is warranted; they are not published feature tickets.
Read relevant GitHub issues on startup before refreshing their local projection.

## Specs and tickets

Publish specs and implementation tickets with `gh issue create --repo
wagar-meilech/bluetooth-discord-bot --title ... --body-file ...`; use a temporary
UTF-8 file to preserve multiline bodies. Link each issue from its task row. Keep
stable local `BDB-###` IDs and exact GitHub issue numbers; do not assume they match.
A spec uses the template in `to-spec`; an implementation ticket records its parent
spec, purpose, agreed behavior, acceptance criteria, agreed test seam, explicit
blocking issue links, and validation evidence. Mirror declared dependencies using
local task IDs. Split an approved spec into concrete tickets before
`implement-spec` begins; never infer dependencies merely from issue order.

GitHub triage labels and their local Status projection:

- `needs-discussion`: unresolved product or testing decisions; mirror as `planned`.
- `ready-for-agent`: approved scope and acceptance criteria; ready once dependencies finish.
- `in-progress`: currently being worked on; mirror as `active`.
- `blocked`: unable to proceed; record the actual blocker in Next or the issue.
- Closed as completed: mirror as `done`, with validation evidence in the issue or PR.
- Closed as not planned: mirror as `cancelled`.

`planned` is also valid for known future work. Unknown dependencies must stay
unknown; use `None` only when there are no prerequisites. A ready implementation
ticket has only completed dependencies. Finishing a spec means publishing agreed
requirements and test seams; it does not mean the feature has been implemented.

`to-spec` publishes a GitHub issue with `ready-for-agent` after its required user
agreement. Replace triage labels as work changes state rather than accumulating
contradictory labels. `implement-spec` uses an integration branch and its task
graph. Open a draft PR once there is a substantive change, link its spec and
tickets, and use closing references for implemented work. After validation, mark
the PR ready for review. Tickets remain open until completion is recorded on
GitHub; a ready PR is not merged work. Setup/design issues can be closed through
`gh issue close` once their completion criteria have evidence. Never commit
credentials, bot tokens, or local state.

## Zoetrope tracking

<!-- zoetrope-tracking:start -->
Coordinator: on goal startup/resume, use `zoetrope-tracking`, check local opt-in
with the current runtime's exact identity, and select/bind only the current goal's
sources. Respect a local disable. The project convention is the maintained table
at `docs/WORK.md#Tasks`; use it only when its task scope matches the current goal.
For a narrower goal, select an appropriate explicit source subset instead of
binding unrelated project work. Keep runtime identities and state paths local.

Workers: maintain assigned GitHub issues and their `docs/WORK.md` projections,
updating status, actual owner, dependencies, and next action at meaningful
transitions. These Markdown rows are attributed working projections of GitHub,
not a live remote-status feed. Source status reports work; verify outcomes with
separate evidence.
<!-- zoetrope-tracking:end -->
