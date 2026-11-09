# Bluetooth Discord bot work

Rust implementation is required. Rounds 1–3 confirm Bluetooth audio received by a
companion service and relayed into Discord server voice channels, plus selected
host-application audio. Both macOS 14.2+ and Windows 11+ hosts are required for a
private trusted group. Built-in Bluetooth must be prototyped before the full
build; bot and capture run locally, one input and voice destination at a time.
The existing host identity is sufficient, with Bluetooth headphones, headset mic,
and other peripherals remaining usable. The native Rust adapter feeds a local
Dockerized Rust core; controls use a menu-bar/tray app and Discord slash commands.
Monitoring is off by default with an opt-in toggle, pairing is operator-approved,
and stable stereo music takes priority with an exploratory delay target near one
second or less. Real-device validation on both hosts is available.

The requested bounded [existing-service and platform research](research/bluetooth-discord-existing-solutions.md)
found partial tools but no verified integrated solution satisfying both native
host requirements. Native macOS Bluetooth remains a feasibility gate. Reuse the
documented Discord/voice frameworks, OS bindings, and tray/menu components; custom
work should focus on integration. Q19–Q26 approve the operating rules, baseline
coexistence criteria, test seams, and shared design. The approved
[spec](spec.md) is published as [GitHub #2](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/2).

Published specs and implementation tickets live in
[GitHub Issues](https://github.com/wagar-meilech/bluetooth-discord-bot/issues).
This table is the local plan and Zoetrope projection, refreshed from GitHub at
meaningful transitions. GitHub issue links were checked on 2026-11-09.

## Tasks

| ID | Task | Status | Owner | Depends on | Next |
| --- | --- | --- | --- | --- | --- |
| BDB-001 | Install project skills and configure GitHub Issues and Zoetrope tracking | done | main | None | Setup verified; see evidence below |
| BDB-002 | [Agree the product behavior, architecture constraints, and test seams through grilling (#1)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/1) | done | main | BDB-001 | Q25/Q26 confirmed; approved spec and tickets published |
| BDB-003 | Publish the agreed spec and create implementation tickets | done | main | BDB-002 | [Spec #2](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/2) and tickets #3–#9 published |
| BDB-004 | [Implement and validate the approved Rust bridge (spec #2)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/2) | blocked | main | BDB-003 | Native capability gate remains unpassed; BDB-005 is testing sink-initiated iPhone signaling |
| BDB-005 | [Prove native macOS Bluetooth receiver feasibility (#3)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/3) | active | main | None | Reviewed pairing and modern Async DISCOVER helpers merged; latest phone test awaits operator selection, then a 25-second native deadline; no live audio evidence |
| BDB-006 | [Prove Windows Bluetooth input, isolation, and coexistence (#4)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/4) | blocked | main | None | Windows CI passed; verified executable ready, awaiting real phone/operator results in windows-hardware-results.md |
| BDB-007 | [Build minimal local audio-to-Discord feasibility harness (#5)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/5) | planned | unassigned | BDB-006 | Reuse a proven native input for the common harness; full gate still requires Mac and Windows |
| BDB-008 | [Record complete two-platform hardware feasibility gate (#6)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/6) | planned | unassigned | BDB-005, BDB-006, BDB-007 | Record both real source-to-listener paths; no full app until passed |
| BDB-009 | [Implement native tray controls and scoped container lifecycle (#7)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/7) | planned | unassigned | BDB-008 | Await the complete hardware gate |
| BDB-010 | [Implement trusted live-stream controls, recovery, and idle policy (#8)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/8) | planned | unassigned | BDB-008 | Await the complete hardware gate; TDD at the approved public seam |
| BDB-011 | [Package, document, and validate the complete private bridge (#9)](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/9) | planned | unassigned | BDB-009, BDB-010 | Await product implementation, then review and record acceptance |

The setup/design/publication rows are local administrative milestones. BDB-004
projects the still-open feature spec; BDB-005 onward project implementation issues.
Task details are maintained in `docs/issues/` and their linked GitHub issues;
status/ownership/dependencies remain in this table and GitHub. A source status is
not hardware verification.

The Mac candidate was investigated first and its PSM-dependent failure isolated.
Mac work is active on sink-initiated signaling to the user's iPhone. Windows
feasibility and the common harness can progress independently of that unresolved
backend. BDB-008 still requires BDB-005, BDB-006 and BDB-007 before any full
product implementation.

## Setup evidence

Five skill packages were scanned with the default local Cisco Skill Scanner and
copied with their supporting files. Every source had one informational finding,
zero medium/high/critical findings, and zero scanner errors. All copied files
were byte-checked against their sources; hashes and the compact scan summary are
recorded in `.agents/skills-manifest.json`. This scan covers the skill packages
and does not establish that they are safe.

GitHub Issues is enabled. Workflow labels `needs-discussion`, `ready-for-agent`,
`in-progress`, and `blocked` were created, and design issue #1 was published with
the confirmed requirements and open design areas.

Zoetrope setup returned `enabled: true` and `project_opt_in: true` for this exact
Codex session. Its `docs/WORK.md#Tasks` source imported four tasks with
`stale: false` and `error: null`. Runtime identity and state paths stay in local
Zoetrope state rather than this repository.

## Feasibility evidence

[Draft PR #10](https://github.com/wagar-meilech/bluetooth-discord-bot/pull/10) contains
the first native Mac prerequisite probe. Locked build, formatting and Clippy passed;
read-only API/controller preflight and local SDP attribute parsing passed. The
single service-publication attempt returned nil/exit 5 with no owned record.
[Observed results and next options](feasibility/macos-receiver.md) keep that
rejection distinct from untested live audio and Bluetooth coexistence. The Mac
ticket is active and the feature remains blocked on capability evidence. Further
[controlled diagnosis](feasibility/macos-sdp-diagnosis.md) isolated requested-PSM
dependence. Windows feasibility is being advanced independently; full product
work remains behind the unchanged two-platform gate.

Windows API code type-checks and the Windows CI release build/lint/artifact upload
passed in run `42543599395`. The downloaded x86_64 executable matches its supplied
SHA256. [Operator instructions and result fields](feasibility/windows-hardware-results.md)
are ready. These checks do not observe Windows hardware, PCM, or coexistence.

The [Mac/iPhone continuation](feasibility/macos-iphone-connection.md) adds the
public OS pairing panel, exact selected-device correlation, duplicate-record
grouping, and a modern Async DISCOVER probe. The operator confirmed phone-side
stored pairing. A legacy open call blocked before returning a channel and was
stopped; it sent no DISCOVER. The replacement has an independent 25-second
own-process deadline, with cleanup explicitly unconfirmed on forced exit.
Host checks, Windows-target Clippy and 12 offline packet cases passed. The latest
native test is waiting for operator selection; neither signaling reception nor
live audio/coexistence has passed.
