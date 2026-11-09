# Optional supplemental reporting

Supported source updates need no JSON apply. Use a compact supplement only for
context the source schema does not manage, native-plan fallback, or explicit
outcome evidence. Read the exact goal's bounded context first:

```sh
'/installed/zoe' state --root '/local/state' context codex 'EXACT_ROOT'
```

Use the executable, provider, full native root and local state supplied by the
request/current validated runtime. Context contains source tasks, field provenance,
current goal/revision and pagination (`next_work_offset`); omitted fields are
partial information. For source tasks, use the returned `source:...` stable ID,
not a match by label. Source-owned fields require an artifact edit.

`state --help` documents `apply`, `upsert_work_item`, `set_goal` and evidence
operations. Use actual returned revisions, update changed fields only, and reread
context on conflict/scope changes. Stable work IDs survive ordinary edits. Owners
refer to this exact provider/root and actual agent identity. Batch related reports
at meaningful transitions; routine tool calls need no report.

If completion criteria are missing, propose concise outcomes for the agreed scope,
including relevant tests/final checks, and record them as unverified through
`set_goal`. A work item's done status is an attributed report. Verify outcomes
only with actual evidence and the current criterion ID/goal revision. Keep earlier
evidence history when scope changes. State commands change local Zoetrope records;
steering still requires an explicit native control action.
