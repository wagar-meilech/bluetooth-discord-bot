## Problem Statement

A small trusted group wants to hear music in a Discord server voice channel by
pairing a phone to a computer as if it were a Bluetooth speaker. Existing music
bots do not provide the requested input flow. The computer must also remain
usable for its normal Bluetooth speakers, headphones, headset microphone, and
other peripherals, while the phone-to-bot stream remains unaffected. Both macOS
and Windows are required, and host-application audio is a second input mode.

## Solution

A native Rust host adapter receives Bluetooth audio or captures one locally
approved application. It feeds a Rust Discord core in a Docker container on the
same computer. A menu-bar/tray app handles local setup and device status; trusted
users control the voice session through Discord slash commands. The phone remains
the music player. Native reception and coexistence must be proven on both host
platforms before building the full application.

## User Stories

1. As a listener, I want to pair my phone with the host like a Bluetooth speaker, so that I can play music from my usual phone apps.
2. As a listener, I want that incoming audio forwarded into a Discord server voice channel, so that my friends can hear it together.
3. As a host operator, I want built-in Bluetooth on macOS 14.2 or newer, so that I do not need an external receiver or another transport.
4. As a host operator, I want built-in Bluetooth on Windows 11 or newer, so that the same product supports my Windows computer.
5. As a host operator, I want the existing host Bluetooth identity used, so that another independently named device is unnecessary.
6. As a host operator, I want my other Bluetooth speakers, headphones, microphones, and peripherals to remain usable, so that receiving music does not disrupt my normal setup.
7. As a source user, I want my incoming music connection to remain unaffected by those other host connections, so that coexistence works in both directions.
8. As a Discord participant, I want my headset microphone and headphone output to keep working relative to a normal call, so that I can talk while sharing music.
9. As a host operator, I want to approve new-device pairing locally, so that Discord users cannot authorize unknown nearby devices.
10. As a host operator, I want previously approved sources to reconnect while the app runs, so that ordinary reconnection is convenient.
11. As a host operator, I want selected-application capture as an alternate source, so that I can also play audio directly from the host.
12. As a host operator, I want application selection and capture permissions controlled locally, so that remote commands cannot choose arbitrary applications.
13. As a participant, I want Discord playback and unrelated desktop audio excluded, so that conversations and notifications are not rebroadcast.
14. As a host operator, I want unsafe or unisolatable sources rejected with a reason, so that capture does not silently broaden to the whole desktop.
15. As a host operator, I want a small macOS menu-bar or Windows tray interface, so that I can see receiver, source, container, and connection status.
16. As a host operator, I want bot and capture components installed and run on the same computer, so that a permanently hosted cloud server is unnecessary.
17. As a host operator, I want the core Dockerized and its lifecycle managed through existing tooling, so that maintenance and deployment remain repeatable.
18. As a host operator, I want Docker or permission failures explained, so that I can correct setup without guessing.
19. As a host operator, I want configuration and credentials kept outside images and source control, so that normal setup does not publish secrets.
20. As a host operator, I want manual startup by default with an opt-in auto-start setting, so that the service runs when intended.
21. As a participant, I want a fresh launch to require an explicit join, so that starting the app does not unexpectedly transmit audio into Discord.
22. As a trusted user, I want join to target my current voice channel within the approved server policy, so that destination selection is simple.
23. As a host operator, I want an explicit user/server allowlist and owner access, so that only trusted people control the stream.
24. As a trusted user, I want join, leave, status, source, volume, pause, and resume controls, so that I can manage the live voice stream.
25. As a trusted user, I want only one active source and one voice destination per installation, so that switching is predictable and sources are not mixed.
26. As a source user, I want pause and volume to affect the bot stream while I control my phone/player directly, so that the bot does not become a remote media library.
27. As a listener, I want stale frames discarded on switching, pause, or disconnect, so that resumed playback is live rather than a delayed backlog.
28. As a host operator, I want Bluetooth forwarding silent locally by default with an explicit monitoring option, so that I do not hear an unwanted extra copy.
29. As a listener, I want stable stereo music with measured delay, so that quality and responsiveness are understood rather than assumed.
30. As a trusted user, I want bounded recovery of the selected source and active voice destination, so that transient losses recover without an unapproved source taking over.
31. As a host operator, I want failed recovery shown after 30 seconds and explicit retry required, so that the application does not retry indefinitely.
32. As a participant, I want leave, local Stop, and Quit to cancel forwarding and recovery immediately, so that stopping has a clear effect.
33. As a host operator, I want Quit to close this application's receiver and managed bot resources while preserving unrelated workloads and saved settings, so that cleanup is scoped.
34. As a participant, I want the bot to leave after 60 seconds alone and require a new join, so that an abandoned channel does not retain a live session.
35. As a host operator, I want diagnostics limited to status and counters, so that tokens and audio recordings are not retained in logs.
36. As a maintainer, I want established Discord, voice, OS-binding, and tray components reused, so that custom code stays focused on product integration.
37. As a project owner, I want a recorded real-hardware feasibility gate on both platforms before the polished app, so that an unsupported receiver path does not lead to wasted implementation work.
38. As a project owner, I want a failed required gate reported with evidence and options, so that hardware, transport, or platform requirements are not silently changed.

## Implementation Decisions

- **Runtime boundary:** a native host adapter owns Bluetooth reception, OS consent,
  selected-application capture, local monitoring, and device lifecycle. A local
  container owns the Discord connection and voice stream. The bridge is local,
  authenticated, and bounded; it is not a remote public ingestion service.
- **Reuse:** start with Serenity/Poise for Discord and commands, Songbird for voice,
  DAVE, Opus, and live PCM input, Microsoft's Windows Rust bindings, macOS objc2
  framework bindings, and tray-icon/Muda for the native shell. Resolve and compile
  compatible released dependencies; use existing Docker lifecycle tooling.
- **Feasibility first:** investigate native Mac reception first, then Windows
  receiver isolation, then the local-container/real-voice path. Bindings or a
  successful compile do not prove an A2DP receiver. Do not build the polished app
  until both complete hardware paths pass.
- **Coexistence:** preserve other host Bluetooth connections and the incoming
  source stream. Compare headset behavior against the same normal Discord-call
  baseline. Ordinary OS headset-mode changes are distinguished from additional
  bot-caused disruption. Resetting radios, unpairing peripherals, or replacing
  the host Bluetooth stack are not accepted workarounds.
- **Control:** owner and explicit trusted user/server policy govern playback and
  moves in the relevant voice context. Join targets the caller's current channel.
  New pairing, application selection, and credentials remain local. Discord source
  commands may reference only Bluetooth or the already approved host application.
- **Startup:** manual launch prepares the native receiver and managed container,
  but a fresh launch does not join a voice channel. Previously approved sources
  may reconnect. Auto-start is opt-in. Configuration/secrets stay outside source
  and images, using OS credential storage where practical.
- **Stream:** one input and one destination per installation. Switching is explicit;
  stale queued frames are cleared on source changes, pause, stop, and disconnect.
  Pause/resume and gain operate on the stream, not the phone's player. There is
  no backlog replay or automatic takeover by a different source.
- **Monitoring/isolation:** default forwarding is silent on the host, with opt-in
  monitoring. Host-app capture must exclude Discord and unrelated output; reject
  an unisolatable source rather than capture the desktop mix.
- **Recovery:** reconnect only the previously selected source and active approved
  destination. Retry transient Discord/network loss for at most 30 seconds, then
  show failure and require explicit retry. Stop/leave/quit cancel recovery.
- **Stop/idle:** explicit stop clears capture/forwarding immediately. Quit closes
  this application's receiver and managed core, preserving other Docker work and
  saved pairings/configuration. Leave after 60 seconds alone; do not auto-rejoin.
- **Quality:** prioritize stable stereo music. Approximately one second or less
  is an exploratory source-to-listener delay target, to measure and discuss before
  treating it as a binding release limit.
- **Data:** diagnostic status/counters only; no raw audio, tokens, or recordings in
  logs. Any hardware evidence records conditions, measurements, and outcomes.

## Testing Decisions

The user approved two public seams in Q25. Test externally observable behavior
against independent expected outcomes, not private methods, wrapper structure,
framework internals, or assertions that restate the implementation.

1. **Automated bridge-session control and audio-input boundary.** Substitute only
   external OS/Discord endpoints. Cover authorization, destinations/sources, one
   active route, stale-frame handling, stop/pause/recovery, and observable audio
   forwarding. This does not prove real Bluetooth, OS consent/routing, actual
   Discord encryption/networking, or physical quality. Use one failing test and
   minimal implementation per vertical TDD slice.
2. **Installed source-to-Discord-listener system.** Real hardware covers receiver
   feasibility, isolation, monitoring, peripheral/headset coexistence, current
   encrypted voice lifecycle, latency, packaging, and the native/container link.
   It does not exhaust every failure combination or certify untested hardware.

The feasibility gate requires built-in Bluetooth and an actual source on each
host, live audio through the local container to Discord, join/rejoin/member
changes, silent default forwarding, optional monitoring, isolated host-app input,
and the agreed coexistence baseline. Sustain at least 30 minutes per host and
exercise switching, source loss, network recovery, and explicit stop. Record
hardware/OSes, delay, dropouts, and resource behavior; distinguish measured, unmet,
and untested outcomes. A required failure stops the full build and triggers an
evidence-backed report rather than a silent fallback.

There is no prior project test suite. New tests establish the approved seams;
third-party library suites are not copied or duplicated. Add a new seam only when
its need emerges and the user agrees it.

## Out of Scope

- Private/group DM calls, public multi-tenant hosting, simultaneous mixed sources,
  and multiple active voice destinations per installation.
- A second independently named Bluetooth device, external Bluetooth receivers,
  Linux-only substitutes, or AirPlay/network input replacing the required native
  Bluetooth path without a new product decision.
- Track queues, searching/downloading media, seeking, remote phone play/skip
  control, broad desktop capture, and recording audio.
- Universal hardware compatibility or a claim that normal OS headset behavior
  disappears. Actual prototype evidence is required for the recorded matrix.
- Polished GUI/product work before the required hardware gate passes.

## Further Notes

The user confirmed shared understanding and authorized spec publication, dependent
GitHub tickets, and feasibility-first implementation in Q26. The Q19 note explicitly
requires that the host's other Bluetooth devices and the music-source connection
remain unaffected. Earlier answers and the exact agreement are preserved in the
[questionnaire](design-questions.md).

The bounded [existing-service/reuse research](research/bluetooth-discord-existing-solutions.md)
found useful building blocks but no verified integrated match for both host
requirements. Native macOS Bluetooth reception remains unproven. No hardware,
voice, or performance gate is passed by publishing this spec.
