# Project instructions

<!-- cisco-skill-scanner:start -->
## Cisco Skill Scanner

Before installing or first using a new third-party agent skill, use the shared `skill-scanner` skill or run `~/.local/bin/skill-scan /absolute/path/to/skill-or-collection`. Keep the default local analysis, summarize its compact report, and inspect flagged code as needed. Recheck updated third-party skills when relevant. This scanner covers agent skills, not arbitrary projects; a clean scan is not proof of safety.
<!-- cisco-skill-scanner:end -->

## Product and workflow

Build a Rust Discord music bot that receives audio through a separate Bluetooth
profile on a device running a companion service, then plays it in Discord voice.
The user confirmed macOS 14.2+ and Windows 11+ service hosts, Discord server voice
channels, and a private deployment for a trusted group. Built-in Bluetooth on
both hosts is required, with a feasibility prototype before the full build.
Selected-application audio from the host is also selected; the user's notes
emphasize the phone-as-speaker workflow. One local installation runs capture and
the bot, with one active input and one destination at a time.

Preserve answers in `docs/design-questions.md`. Round 3 resolves Bluetooth
identity: the existing host identity is sufficient if required coexistence is
verified. Bluetooth headphones, headset microphone, and other peripherals must
remain usable. The user requests no interference; distinguish ordinary OS
headset-mode behavior from disruption caused by the bot in the acceptance criteria.

The agreed runtime is a native Rust adapter plus a Dockerized Rust Discord core
on the same computer. Provide a macOS menu-bar / Windows tray app and Discord
slash commands. Local monitoring is off by default with an opt-in toggle. Prioritize
stable stereo music; approximately one second or less delay is an exploratory
target to measure. Only the host operator approves new-device pairing, and real
hardware on both platforms is available for validation.

Reuse established components to save time: Serenity, Poise, Songbird (including
DAVE/Opus/PCM support), native Rust OS bindings, and existing tray/menu components.
The source-backed plan is in `docs/research/bluetooth-discord-existing-solutions.md`.
The existing-service check found partial tools but no verified integrated match
for both hosts. Keep custom work focused on integration; native macOS Bluetooth
reception remains a feasibility gate.

Q19–Q26 confirm the operating rules, the two public test seams, and shared
understanding. The approved [spec](docs/spec.md) is published as GitHub issue #2;
implementation tickets #3–#9 form the dependency graph in `docs/WORK.md`.
Proceed through feasibility first; the full app is gated on the recorded
two-platform hardware acceptance. Apply TDD at the approved public session/input
boundary and record real-device evidence separately. Use the project's pinned
Rust toolchain rather than changing the user's global default.

Project skills live in `.agents/skills/`: `grilling`, `tdd`, `to-spec`,
`implement-spec`, and `zoetrope-tracking`. Read the applicable `SKILL.md` before use.
The verified installation and scan summary are in `.agents/skills-manifest.json`.

Follow [the project issue tracker](docs/agents/issue-tracker.md) for the task graph,
triage labels, spec publication, and completion rules. Use `grilling` to settle the
design with the user before invoking `to-spec`. Agree test seams before tests;
implement approved behavior in Rust through the `tdd` workflow.

<!-- zoetrope-tracking:start -->
For coordinator startup/resume in an opted-in project, use `zoetrope-tracking`
and follow the tracking convention in [the issue tracker](docs/agents/issue-tracker.md).
<!-- zoetrope-tracking:end -->
