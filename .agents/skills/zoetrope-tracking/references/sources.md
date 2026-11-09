# Choose structured sources

Use one existing maintained plan. Recognize both an aggregate task table and Matt
Pocock-style individual ticket files, following the project's own tracker pointer
rather than editing installed upstream skills. Specs and tickets remain the
project's workflow; Zoetrope reads their designated structured area only.

## Adapt the smallest area

Select `relative/file.md#Exact heading` for the first table under one unique
heading, or one explicit `relative/ticket.md@ticket` per ticket. Select only this
goal's scope. Files must resolve inside the canonical project root. No glob,
directory crawl, remote URL, or fenced example becomes an input. At most 16 source
files per binding, 256 KiB each, 512 current tasks total. For current command syntax and supported fields,
use the installed executable's `tracking --help`.

Tables require ID/Ticket, Task/Title and Status. Preserve stable existing IDs;
add only missing required fields. Optional Owner, Depends on/Blocked by/Dependencies,
Purpose, Change, Next and Blocker columns provide useful context when already
maintained or justified by the task. Existing status words are accepted below;
unknown values stay unknown rather than guessing. Preserve prose and links.

```markdown
## Tasks

| ID | Task | Status | Owner | Depends on | Next |
| --- | --- | --- | --- | --- | --- |
| T1 | Implement save | active | /root/save | None | Run save checks |
| T2 | Review save | blocked | Alex | T1 | Await implementation |
```

Ticket metadata is within the first 64 lines, before the first body heading,
rule or prose. A leading title supplies Task. Plain or bold field names work:

```markdown
# Implement save

ID: T1
Status: active
Owner: /root/save
Blocked by: None
Next: Run save checks

## Acceptance criteria

Existing ticket body continues here.
```

IDs use 1–64 ASCII letters/digits/`._-`. Dependencies are comma-separated IDs
within a selected table, or unambiguous IDs across the explicitly selected ticket
set. `None`/`[]` means no prerequisites; blank means unknown. Keep declared cycles
and missing references honest; grouping and file order do not imply dependency.
Tables use leading/trailing pipes, equal cell counts and a dash separator; escape
literal pipes as `\|` and backslashes as `\\`. Duplicate IDs/recognized columns
or metadata fields invalidate that source.

Status mappings: planned/pending/ready-for-agent → planned;
active/in_progress/in-progress → active; blocked → blocked;
done/completed/resolved → reported done; cancelled/canceled → cancelled.
Owner `main`, `/root/path`, or `native:EXACT_THREAD_ID` is a scoped native reference
requiring exact public owner proof before steering. Other text is a nonsteerable
human/unassigned label. Record the actual owner, never invent a native mapping.

## No suitable convention

First reuse an existing native plan or compact work reports. If a useful prose
plan exists, adapt its smallest task area with required fields; keep it as the
single maintained plan. When the project truly lacks one and setup is authorized,
establish one minimal task document (for example `docs/WORK.md#Tasks`) seeded with
the work already known from the request/plan. Use honest planned/active/blocked
statuses and only declared dependencies. Keep native/report fallback if the work
is not yet known; do not replace it with an empty starter table unless the user
intentionally selects an empty effective plan.

## Routine edits and failures

Maintain normal artifacts at meaningful work transitions. Source-managed fields
remain authoritative; a present empty field clears its value. Deleting a managed
column does not revive old supplemental values. A valid snapshot removing a task
retires it from current work; it does not imply success or cancellation. Invalid
or missing sources retain last-good tasks and show stale/error context. Correct
the designated source and let normal refresh recover. Source status and
checkboxes never verify outcomes; evidence and scope revisions stay separate.
