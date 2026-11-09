# Project setup and continuation

## Establish identity and authorization

A UI Enable tracking request names an executable, state root, provider, exact
native root session, project and this installed skill path. Use those values,
preserving shell quoting. The editable request authorizes project setup when the
operator sends it; merely preparing it changes no project instruction.

On later coordinator startup, use the current runtime's exact identity. For Codex,
`--session auto` validates both `CODEX_SESSION_ID` and `CODEX_THREAD_ID` against
public native headers and ancestry. Missing or mismatched identity leaves setup
incomplete; obtain an exact supplied identity rather than choosing by cwd, title
or recency. Claude requires an explicitly supplied exact root identity. Resolve
the installed `zoe` executable and its local state root afresh (`zoe state --help`
shows the default). Runtime paths and identities belong in local state, never in
committed project pointers. Each goal chooses its own explicit source subset.

Check local state before any enable or project edit:

```sh
'/installed/zoe' tracking status --project '/project' --provider codex --session auto --root '/local/state'
```

Replace the example paths/provider/session with established values. An explicit
request to enable permits setup/re-enable. During ordinary resumed/startup work,
`project_opt_in: false` takes precedence over any committed pointer: leave tracking
disabled and continue ordinary artifacts. A committed opt-in is a convention,
not permission to override a local disable. If the project is opted in but this
exact goal has no enabled binding, select sources for this goal and bind them;
never reuse a historical coordinator's identity or bind every project task by
default. An existing enabled binding with the right sources needs no enable.

## Reuse the project convention

Read existing `AGENTS.md`/`CLAUDE.md` and follow their pointer to the project tracker
(e.g. `docs/agents/issue-tracker.md`). Inspect the actual plan/tickets and choose
sources from [structured sources](sources.md). Finish selection with a nonempty
known-work source, or deliberately retain native-plan/compact-report fallback if
no source is suitable. An empty source intentionally replaces the effective plan;
never bind an empty starter table that hides useful existing work.

Before edits, show a compact preview of the selected source subset, any missing
structured fields, and the managed pointer locations. Existing setup authorization
covers those narrow edits; ask only when authorization or actual source choice is
unresolved. Preserve project prose and the existing task plan.

Integrate through the existing tracker document. Add/update only one managed block
bounded by `<!-- zoetrope-tracking:start -->` and
`<!-- zoetrope-tracking:end -->`. Keep portable selectors and these two roles:

- Coordinator: use `zoetrope-tracking` on goal startup/resume, check local opt-in,
  then select/bind only this exact goal's sources using current runtime identity.
- Workers: maintain assigned tasks in the named ordinary artifacts; task status
  does not verify outcomes.

List the project's source convention, not a machine path, native session or
universal binding. Existing instruction-to-tracker routing is sufficient if it
already reaches this guidance. Otherwise add a tiny managed conditional pointer
to the primary existing instruction file. If no tracker convention exists, use
`docs/agents/zoetrope-tracking.md` for the block and point to it from `AGENTS.md`
(or the project's chosen instruction file). Keep this project material short:
setup details live in this installed skill. Repeated setup updates the block in
place and leaves a single maintained task plan.

## Bind and confirm

Use the exact identity/root and one `--source` per selected table or ticket:

```sh
'/installed/zoe' tracking enable --project '/project' --provider codex --session auto --source 'docs/WORK.md#Tasks' --root '/local/state'
```

`enable` opts in the project and replaces only this goal's source subset. Inspect
its bounded JSON: project/identity, enabled state, each selected source's task
count, stale flag and error. A successful process may still report malformed or
missing sources; resolve those diagnostics before calling setup complete. Read
compact effective tasks once with `state --root '/local/state' context codex
'EXACT_ROOT'` if needed to confirm the selected work (use the resolved root ID,
not `auto`, with `state`). Normal console refresh imports ordinary edits on its
existing worker every five seconds. `tracking sync` is optional one-shot recovery,
not a reporting obligation. The importer neither runs network commands nor reads
remote issue status; a remote tracker's local artifacts are attributed working
projections with their existing links/as-of context.

## Disable

For an explicit disable request, run `tracking disable` with this exact project,
provider/session and state root. It opts out the project and stops every existing
binding without erasing history/evidence. Remove only managed tracking blocks
and any now-unused managed pointer; retain task artifacts, original instructions
and upstream skills. A dedicated managed tracker file may be removed only if it
contains solely the managed block. A retained committed pointer cannot undo the
local disable. Re-enable requires an explicit enable request and selects only the
newly authorized exact goal binding.
