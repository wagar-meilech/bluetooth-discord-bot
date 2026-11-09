# Design questions

Mark one option per question by changing `[ ]` to `[x]`. Add details under
**Notes** when needed. Recommendations are suggestions; no answers are preselected.

**Status:** all four rounds answered; Q25 confirms the test seams and Q26 confirms
shared understanding and feasibility-first implementation. The approved design is
in [the spec](spec.md); implementation is tracked in [WORK.md](WORK.md).
Earlier answers are retained below. Q11 resolves Q5; Q12 requires headset-microphone
use as well as headphone output and other peripherals.

Related work: [GitHub design ticket #1](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/1).

## Round 1 — answered

### Q1 — Audio flow

Should a phone or computer see the service host as a Bluetooth speaker, send audio
to it, and have the bot relay that audio into Discord?

- [X] Yes: source device → Bluetooth → service host → Discord voice. **Recommended.**
- [ ] I want a different audio flow; describe it below.
- [ ] I'm unsure and want to discuss the options.

**Notes:**
Should also support playing directly from service host

<!-- Describe any different flow or clarify which devices you have in mind. -->

### Q2 — Discord scope

Can the first version target server voice channels?

Discord explicitly excludes bots from group DMs, so group DM calls cannot use
the normal bot approach. [Discord documentation](https://docs.discord.com/developers/topics/oauth2#bot-vs-user-accounts).

- [X] Yes: support server voice channels initially. **Recommended.**
- [ ] Private or group DM calls are essential; discuss feasibility before proceeding.
- [ ] I need another Discord session type; describe it below.

**Notes:**

<!-- Add any requirements for the kinds of Discord sessions you use. -->

### Q3 — Service host

Can the receiving service run on Linux initially, or do you require another
operating system?

Linux has documented Bluetooth audio receiver and headless service support.
[WirePlumber documentation](https://pipewire.pages.freedesktop.org/wireplumber/daemon/configuration/bluetooth.html).
Other platform requirements will need their own feasibility checks.

- [ ] Linux first. **Recommended.**
- [ ] macOS is required for the first version.
- [ ] Windows is required for the first version.
- [X] Multiple operating systems are required initially; list them below.
- [ ] Another platform, or undecided; explain below.

**Notes:**
macOs and Windows
<!-- Mention required hardware, operating systems, or deployment constraints. -->

### Q4 — Audience

Should the first release serve you and a few trusted friends?

- [X] Yes: a private deployment for a trusted group. **Recommended.**
- [ ] It must support several independently managed groups or servers initially.
- [ ] It must be a public bot that other people can install initially.
- [ ] Another audience, or undecided; explain below.

**Notes:**

<!-- Describe who will run and use it. -->

### Additional context

<!-- Add requirements, examples, or concerns that do not fit the questions above. -->

## Round 2 — platform feasibility and audio routing

Your settled requirements from round 1:

- Rust implementation.
- Bluetooth source device → service host → Discord server voice channel.
- Also support audio played directly from the service host.
- Both macOS and Windows service hosts in the initial release.
- A private deployment for you and trusted friends.

Relevant findings before choosing the next branches:

- Windows has a documented A2DP receiver API. It plays received audio locally;
  extracting and isolating that audio for Discord needs a prototype.
  [Microsoft guide](https://learn.microsoft.com/en-us/windows/apps/develop/media-playback/enable-remote-audio-playback).
- We have not established a ready public macOS A2DP receiver API. Apple's archived
  Bluetooth documentation describes APIs for implementing profiles, so a custom
  receiver is a research candidate whose current macOS compatibility is unproven.
  This is not a conclusion that macOS reception is impossible.
  [Apple Bluetooth guide](https://developer.apple.com/library/archive/documentation/DeviceDrivers/Conceptual/Bluetooth/BT_Develop_BT_Apps/BT_Develop_BT_Apps.html).
- Receiving an audio profile on the host's Bluetooth identity and advertising a
  second independently named Bluetooth device are different requirements. The
  documented receiver APIs do not establish the latter capability.
- Direct host audio has documented capture paths: macOS Core Audio taps and
  Windows WASAPI loopback/process capture. Whole-system capture can include
  Discord voices and notifications; selected-app capture can narrow the scope.
  [Apple audio taps](https://developer.apple.com/documentation/coreaudio/capturing-system-audio-with-core-audio-taps),
  [Microsoft process capture](https://learn.microsoft.com/en-us/samples/microsoft/windows-classic-samples/applicationloopbackaudio-sample/).

### Q5 — What does “separate Bluetooth profile” mean?

Is selecting the host computer's existing Bluetooth entry sufficient, provided
it accepts music audio for the bot?

➡️ Recommended: yes, use the existing host identity with an audio receiver role.
Do not require a second independently named Bluetooth device unless that is
essential to your use case.

- [X] Yes: pairing to the host computer's existing Bluetooth entry is sufficient. **Recommended.**
- [X] It must appear as a separate Bluetooth speaker/device entry, distinct from the host computer.
- [ ] I mean a selectable audio output on the service host itself; explain below.
- [ ] Another meaning, or undecided; explain below.

**Notes:**
The reason I think it should be different from the host computer is because the host computer might itself be connected to other bluetooth devices, so am not sure how would that work

<!-- Describe what you expect to see in the source device's Bluetooth or audio-output menu. -->

### Q6 — macOS Bluetooth feasibility

Is built-in Bluetooth reception on both macOS and Windows a hard requirement,
with a feasibility prototype before committing to the full implementation?

➡️ Recommended: yes, validate the requested software-only Bluetooth path first.
If macOS reception cannot be made reliable, return with evidence and discuss a
fallback before changing the product scope.

- [X] Yes: built-in Bluetooth on both hosts is required; prototype first and discuss any failure. **Recommended.**
- [ ] An external Bluetooth audio receiver plus a host audio-capture input is acceptable if needed.
- [ ] Bluetooth reception on Windows and direct host audio on macOS would be sufficient initially.
- [ ] Another constraint or fallback; describe below.

**Notes:**

<!-- Note any willingness to add hardware or change input transport. No hardware purchase is assumed. -->

### Q7 — Audio played directly from the service host

Should this mean selecting a running application and streaming only its audio?

➡️ Recommended: yes, selected-application audio. This keeps unrelated desktop
sounds outside the intended stream. We will separately validate exclusion of
Discord playback and the bot's own output.

- [X] Yes: select a running application, such as a browser or music player. **Recommended.**
- [ ] Choose a dedicated virtual audio output such as “Discord Music” in app/system sound settings; driver requirements need investigation.
- [ ] Capture all audio from a selected system output device.
- [ ] Play files or URLs supplied to the service instead of capturing application audio.
- [ ] I need a combination or another behavior; describe below.

**Notes:**
I just want to connect my phone to it, as I would to a speaker and play music

<!-- Give an example of how you would start music on the host and send it to Discord. -->

### Q8 — Where the bot runs

Should one installation on the service host include both audio reception/capture
and the Discord bot connection?

➡️ Recommended: yes, run both on the macOS or Windows host as one local
installation, configured with your Discord bot credentials.

- [X] Yes: audio service and bot run together on the host. **Recommended.**
- [ ] Audio is captured locally and streamed to a separately hosted Discord bot.
- [ ] Either topology must be supported initially.
- [ ] Another deployment arrangement, or undecided; explain below.

**Notes:**
Unless discord requires a permanent running server to register a bot

<!-- Mention any existing server, always-on requirement, or deployment constraints. Do not paste bot tokens. -->

### Q9 — Simultaneous streams

Can each installation send one active audio input to one Discord voice channel
at a time, with switching between Bluetooth and host audio?

➡️ Recommended: yes, one input and one destination per installation initially.

- [X] Yes: one active input and one destination, with source switching. **Recommended.**
- [ ] Bluetooth and host audio must be mixed simultaneously into one destination.
- [ ] Multiple simultaneous sources or Discord destinations are required; describe below.
- [ ] Undecided; explain the use case below.

**Notes:**

<!-- Mention whether multiple people, host installations, or servers must play music at once. -->

### Q10 — Minimum operating-system versions

Can the first release require macOS 14.2 or newer and Windows 11 or newer?

➡️ Recommended: yes, use these baselines while evaluating modern application
audio-capture APIs. Apple's Core Audio taps sample requires macOS 14.2+, and
Microsoft's process-capture sample requires Windows build 20348+. These version
baselines do not establish Bluetooth receiver feasibility or hardware support.

- [X] Yes: macOS 14.2+ and Windows 11+. **Recommended.**
- [ ] Older macOS versions must be supported; specify the oldest version below.
- [ ] Windows 10 must be supported; specify the version/build below if known.
- [ ] Older versions of both must be supported; describe below.
- [ ] Undecided; list the machines you intend to use below.

**Notes:**

<!-- Optionally list your host machines/OS versions and the phones/computers that will send Bluetooth audio. -->

### Additional context for round 2

Also let's dockerize it, and use existing rust frameworks to improve maintenability. Also, Ig do a quick check if such a service already exists before we commit to building it (specifically bluetooth streaming into a discord VC, not just playing music or sound that obviously exists)

<!-- Add requirements or examples that affect these decisions. Later rounds will cover controls, pairing, permissions, monitoring, reconnect behavior, performance targets, and test seams once these branches are settled. -->

## Round 3 — Bluetooth coexistence and everyday use

Recorded from round 2:

- Built-in Bluetooth is required on both hosts; prototype it before the full build.
- Selected-application host capture remains selected. Your note emphasizes that
  the main experience should be pairing a phone as if to a speaker.
- Bot and audio service run together on your computer, one input and one voice
  channel at a time.
- Minimum versions: macOS 14.2+ and Windows 11+.
- Docker packaging and established Rust frameworks are requested.
- You requested an existing-service check before committing to a build.

### Answers to the factual questions in your notes

**Other Bluetooth connections:** one host radio can maintain several connections.
A second device name does not create extra radio capacity. The requirement to
test is coexistence with your peripherals, especially receiving phone audio while
sending audio to Bluetooth headphones. Headset-microphone use is a further case.
[Bluetooth specification](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Core_v7.0/out/en/br-edr-controller/baseband--bb--specification.html),
[Windows audio modes](https://learn.microsoft.com/en-us/windows-hardware/drivers/bluetooth/bluetooth-classic-audio).

**Bot registration:** a permanent cloud server is unnecessary. The bot can connect
to Discord from your computer while you use it; the app needs to stay running and
online while it plays. This follows from Discord's outbound Gateway and voice
connections. [Discord Gateway](https://docs.discord.com/developers/events/gateway),
[voice connections](https://docs.discord.com/developers/topics/voice-connections).

**Existing service:** the bounded search found no verified integrated solution
meeting the native macOS + Windows requirement. Existing receiver and live-input
Discord tools could be composed on Windows, but none of the reviewed projects
establishes the native Mac receiver. Read the
[candidate comparison and sources](research/bluetooth-discord-existing-solutions.md).
No candidate was installed or tested end to end.

**Docker:** native macOS/Windows Bluetooth and audio APIs belong on the host.
Docker Desktop's Linux VM cannot directly call those APIs. The proposed boundary
is a native Rust host adapter plus a local container for the Rust Discord core;
it does not resolve the Mac receiver feasibility question.
[Docker architecture](https://docs.docker.com/desktop/features/vmm/).

**Rust reuse:** Serenity, Poise, and Songbird are current candidates for the
Discord portion; Songbird includes DAVE voice encryption support. We can evaluate
these rather than write the Discord protocols ourselves. Platform Bluetooth work
still requires its own prototype. See the research note for project sources.

### Q11 — Resolve the two Bluetooth-identity choices

Given the explanation above, is the existing host Bluetooth identity sufficient
if receiving music preserves the other Bluetooth uses you require?

➡️ Recommended: yes. Make coexistence a prototype acceptance criterion; do not
require another independently named Bluetooth device to solve that concern.

- [X] Yes: the host identity is sufficient if the required coexistence is verified. **Recommended.**
- [ ] A separate discoverable speaker entry is still essential, independently of coexistence; explain why below.
- [ ] I need another distinction in the source device's menu; describe it below.

**Notes:**

<!-- This resolves the two checked options in Q5 without changing your original answer. -->

### Q12 — Which simultaneous Bluetooth audio use is required?

Must receiving phone music work while you use Bluetooth headphones on the same
computer for Discord?

➡️ Recommended: yes if this is your everyday setup. Validate simultaneous incoming
music and headphone output explicitly; validate headset-microphone mode separately
if you need it. Mouse/keyboard coexistence should also be checked.

- [X] Yes: Bluetooth stereo headphones plus mouse/keyboard must keep working; I can use another microphone. **Recommended if this matches your setup.**
- [X] Yes, and the Bluetooth headset microphone must work for my Discord conversation too.
- [ ] Mouse/keyboard coexistence is required, but Bluetooth headphone output is not; I can use wired/USB audio.
- [ ] Another required combination; describe below.

**Notes:**
It shouldn't interfere with anything

<!-- List your actual headphones/headset and any other Bluetooth devices that must remain usable. -->

### Q13 — Docker packaging boundary

Is a native Rust host adapter feeding a Dockerized Rust Discord core on the same
computer an acceptable way to satisfy the Docker requirement?

➡️ Recommended: yes. Keep Bluetooth reception, OS permissions, and application
audio capture native; containerize the Discord connection and audio processing.
The feasibility work must still prove both host paths and the local audio link.

- [X] Yes: native host adapter plus a local Dockerized bot core. **Recommended.**
- [ ] Prefer a fully native runtime; Docker is needed only for build/test tooling.
- [ ] The complete runtime must be inside Docker; revisit the host/platform requirements before designing further.
- [ ] Another packaging goal; explain below.

**Notes:**

<!-- Both components can remain local; no remote server is implied. -->

### Q14 — Everyday controls

Should everyday use have a small macOS menu-bar / Windows tray app for host
settings and Discord slash commands for voice-channel controls?

➡️ Recommended: yes. Use the local app for pairing/capture status and source
selection, and Discord commands for joining/leaving a channel, stream status,
and volume. This keeps phone-to-speaker playback as the main workflow.

- [X] Yes: menu-bar/tray app plus Discord slash commands. **Recommended.**
- [ ] A command-line/background service plus Discord slash commands is sufficient initially.
- [ ] All controls should be in a local graphical app; no Discord commands are needed.
- [ ] Another interface or workflow; describe below.

**Notes:**

<!-- Describe the shortest sequence you want from launching the app to hearing phone music in Discord. -->

### Q15 — Local monitoring

Should received music be sent only to Discord by default, with an explicit option
to monitor it locally through selected headphones/speakers?

➡️ Recommended: yes, local monitoring off by default with an opt-in toggle.
Prototype silent forwarding on both platforms; a receiver that merely plays into
the desktop mix is not sufficient evidence for this behavior.

- [X] Yes: Discord output by default; optional local monitoring. **Recommended.**
- [ ] Always play through the host's selected headphones/speakers as well as Discord.
- [ ] Never play locally; a monitoring option is unnecessary.
- [ ] Another behavior; describe below.

**Notes:**

<!-- Your normal Discord client may already play the bot's audio; consider whether an extra local copy is useful. -->

### Q16 — Playback priority and delay

Is smooth stereo music playback the priority, with a short buffering delay
acceptable?

➡️ Recommended: yes, music-first playback. Start with a target of roughly one
second or less from source to listener, measure it during feasibility work, and
discuss the result before treating it as a release requirement. This is a design
target, not a measured claim or guarantee.

- [X] Yes: prioritize stable stereo music; the proposed delay target is acceptable. **Recommended.**
- [ ] Tight synchronization with videos/games is essential; describe the use case and acceptable delay below.
- [ ] A longer delay is acceptable if it improves reliability or simplicity.
- [ ] I want to choose a numeric limit; specify it below.

**Notes:**

<!-- Distinguish casual music listening from synchronized video, gaming, or live performance. -->

### Q17 — Real-device validation

Can at least one Mac, one Windows PC, and your intended Bluetooth source phone
be made available for manual validation before the full build?

➡️ Recommended: yes. OS documentation and automated tests cannot by themselves
prove the native Bluetooth path, audio isolation, or coexistence on real hardware.
Record the devices/OS versions below so the feasibility criteria can be concrete.

- [X] Yes: both host platforms and the intended source device(s) can be tested. **Recommended.**
- [ ] Only macOS hardware is available initially.
- [ ] Only Windows hardware is available initially.
- [ ] I need help arranging hardware validation; describe what is available below.

**Notes:**

<!-- List intended source devices (iPhone/Android/computer, models/OS if known) and host test machines available for future validation. -->

### Q18 — Pairing new devices

Should the host operator explicitly enable/approve pairing for new source devices?

➡️ Recommended: yes. Use normal OS Bluetooth pairing where the platform permits,
and expose receiver/pairing status locally. Do not let a Discord command authorize
an unknown nearby Bluetooth device.

- [X] Yes: only the host operator enables/approves new-device pairing. **Recommended.**
- [ ] Trusted Discord users should also be able to enable a pairing window; define that access later.
- [ ] I need unattended/open pairing; explain the setting below.
- [ ] Another pairing flow; describe below.

**Notes:**

<!-- Reconnection of already paired devices and Discord command permissions will be settled after the control interface is chosen. -->

### Additional context for round 3

<!-- Remaining branches after this round include command permissions, source switching, startup/reconnect/stop behavior, prototype acceptance criteria, agreed test seams, and final design confirmation. -->

## Round 4 — operating rules, test seams, and confirmation

Your round 3 choices settle the following:

- The existing host Bluetooth identity is sufficient if coexistence is verified.
- Bluetooth headphones, the headset microphone, and other peripherals must remain
  usable. Your note is: “It shouldn't interfere with anything.”
- Native Rust host adapter plus a Dockerized Rust core on the same computer.
- Menu-bar/tray controls plus Discord slash commands.
- Local monitoring off by default, with an opt-in toggle.
- Smooth stereo music is the priority; roughly one second or less is an
  exploratory end-to-end delay target, to measure before making it binding.
- Real Mac/Windows/source-device validation is available.
- Only the host operator approves new-device pairing.

### Concrete reuse plan

Use Serenity and Poise for Discord/commands, Songbird for voice/DAVE/Opus/live PCM,
Microsoft's `windows` bindings for Windows APIs, `objc2` framework bindings for
macOS APIs, and `tray-icon` with its Muda menu re-export for the native shell.
Use Docker's existing lifecycle tooling for the container. Keep custom code
focused on receiver feasibility, capture/routing, the local audio link, and product
behavior. The [reuse plan and sources](research/bluetooth-discord-existing-solutions.md#concrete-reuse-plan-after-round-3)
distinguish verified library capabilities from integration that still needs proof.
macOS bindings do not themselves implement an A2DP receiver.

### Q19 — Make “no interference” testable

Can we define this as no additional bot-caused disconnections, device resets,
audio stealing, or headset-mode changes compared with your normal Discord call?

➡️ Recommended: yes. Preserve the normal host setup and compare before/after on
the same devices. Windows can already change Classic Bluetooth headset modes
when a microphone/communications session opens; a second Bluetooth identity does
not remove that OS behavior. If the bot needs disruptive changes or cannot meet
the required coexistence, fail the feasibility gate and report it.
[Windows audio modes](https://learn.microsoft.com/en-us/windows-hardware/drivers/bluetooth/bluetooth-classic-audio).

- [X] Yes: no extra interference relative to my normal call, including headset-microphone use. **Recommended.**
- [ ] Full-quality stereo headset output plus its microphone must work even if my normal OS setup cannot provide it; treat this as an additional feasibility requirement.
- [ ] Another measurable definition; describe below.

**Notes:**
No interference means, the host device should still be able to connect to other bluetooth speakers/microphones/devices and the bot/devices connected to the host device to play music should be unaffected

<!-- No disabling/replacing the host Bluetooth stack, resetting the radio, or unpairing other devices is assumed as an acceptable workaround. -->

### Q20 — Trusted command users and voice destination

Should the bot owner and an explicitly configured user/server allowlist control
streaming, with `/join` targeting the caller's current voice channel?

➡️ Recommended: yes. Restrict playback/move commands to approved users in the
allowed server and relevant voice channel. Keep pairing, application selection,
and credentials under local host control. Discord commands can choose Bluetooth
or the already approved host application; they cannot select arbitrary host apps.

- [X] Yes: explicit trusted users/server, caller-channel join, and local-only sensitive settings. **Recommended.**
- [ ] Any member in the configured server/voice channel may control playback.
- [ ] Only the bot owner may control playback.
- [ ] Another permission or channel-selection policy; describe below.

**Notes:**

<!-- IDs and tokens can be entered during setup; do not paste credentials here. Local emergency stop always remains available to the host operator. -->

### Q21 — Starting a session

Should launching the local app prepare its receiver and container, but require an
explicit `/join` before audio goes into a Discord channel?

➡️ Recommended: yes. Start manually by default; offer opt-in OS auto-start later
in settings. Check Docker availability and show actionable status. Previously
approved sources can reconnect while the app is running. Do not auto-join a voice
channel after a fresh launch. Store configuration/credentials outside the image
and repository, and use the OS credential store for secrets where practical.

- [X] Yes: app launch prepares the service; explicit join starts the voice session. **Recommended.**
- [ ] Automatically join a saved voice channel when the app starts; describe the intended trigger below.
- [ ] The user should start/stop the container manually outside the app.
- [ ] Another startup behavior; describe below.

**Notes:**

<!-- The app manages only this project's container resources and does not stop unrelated Docker workloads. -->

### Q22 — Live playback, switching, and recovery

Should this behave as a live stream: switch sources explicitly, discard stale
audio, and let pause/volume affect the bot stream while the phone/player stays
under your control?

➡️ Recommended: yes. Provide join/leave/status/source/volume and stream
pause/resume controls. Flush old frames on switching, pause, or disconnect; do not
replay a backlog. Reconnect only the previously selected source/session, without
allowing another phone to take over. For transient Discord/network loss, retry
the same active destination for up to 30 seconds, then show failure and require
an explicit retry. Leaving, stopping, or quitting cancels recovery.

- [X] Yes: live-stream controls, explicit switching, no stale replay, and bounded recovery. **Recommended.**
- [ ] Also require play/pause/skip controls that operate the phone/player itself; investigate transport support first.
- [ ] Prefer automatic source switching or different recovery rules; describe below.

**Notes:**

<!-- There is no track queue, seeking, downloaded-media playback, or recording implied by these stream controls. -->

### Q23 — Stop and empty-channel behavior

Should `/leave`, local Stop, and app Quit immediately stop forwarding, and should
the bot leave after 60 seconds alone in its voice channel?

➡️ Recommended: yes. Stop capture/forwarding and clear queued frames on leave or
Stop; Quit also closes the receiver session and this app's managed bot process.
Keep saved pairings/configuration for next time. Do not automatically rejoin after
an empty-channel leave. Diagnostic logs contain status/counters, not audio, tokens,
or recordings. Local monitoring remains an explicit session choice.

- [X] Yes: immediate explicit stop, plus leave after 60 seconds alone. **Recommended.**
- [ ] Explicit stop only; remain in the channel when alone.
- [ ] Another idle/stop policy; describe below.

**Notes:**

<!-- Change the idle timeout here if needed. Phone playback itself remains under the phone's control. -->

### Q24 — Feasibility gate before the full build

Can the prototype be judged against the following concrete gate on both host OSes?

1. A real intended source pairs to the built-in receiver and supplies live audio;
   no external receiver or alternate transport substitutes for that proof.
2. The audio reaches the local container and a real Discord voice listener using
   current voice encryption; joining/rejoining and member changes are exercised.
3. Default forwarding is silent locally, optional monitoring works, and the chosen
   host-app capture excludes Discord playback and unrelated audio. Reject a source
   that cannot be isolated instead of silently capturing the whole desktop.
4. The Bluetooth devices/headset microphone remain usable under Q19's definition.
5. Sustain at least 30 minutes of playback per host and exercise source switching,
   source loss, network recovery, and explicit stop. Record actual hardware/OSes,
   dropouts, delay, and resource behavior. Report the approximate one-second delay
   goal as measured or unmet; do not silently relabel it as achieved.

➡️ Recommended: yes. Investigate the uncertain native Mac receiver first, and use
existing components/reference apps where useful to isolate problems. If a required
gate fails, stop the full build and return with evidence and options. Build the
polished app after both platform paths pass; do not spend time on it before the
Bluetooth requirement is proven.

- [X] Yes: use this gate before committing to the full application. **Recommended.**
- [ ] Change the gate/duration or add a required setup; specify below.
- [ ] Permit one platform to ship before the other; explain the scope change below.

**Notes:**

<!-- Record the concrete phone/host/headset models when running the hardware validation. Same-browser music and Discord may need rejection if app-level capture cannot isolate them. -->

### Q25 — Agree the test seams

Can tests concentrate on these two public boundaries instead of internal modules
or the behavior of third-party frameworks?

| Seam | What it catches | What it misses |
| --- | --- | --- |
| Public bridge-session control and audio-input interface, tested automatically with external OS/Discord endpoints substituted | Permissions, channel/source selection, one active route, stale-frame handling, stop/pause/recovery, and externally observable forwarding | Real Bluetooth roles, OS consent/routing, actual Discord encryption/networking, and physical audio quality |
| Installed system, from real source through native adapter/container to a Discord listener, tested on actual hardware | Receiver feasibility, audio isolation, monitoring, Bluetooth coexistence, real voice lifecycle, latency, and packaging | Exhaustive failure combinations and hardware not included in the recorded matrix |

➡️ Recommended: yes. Use behavior-focused TDD at the first seam, one failing test
and minimal implementation per slice. Use the second seam for the feasibility
gate and hardware acceptance evidence. Add another seam only if its need emerges
and we agree it, rather than testing thin wrappers or vendor internals.

- [X] Yes: these seams and their stated limitations match my expectations. **Recommended.**
- [ ] Change/add a seam or acceptance boundary; describe below before writing tests.

**Notes:**

<!-- The tdd skill requires pre-agreed seams; approving this question approves the boundaries, not an unmeasured hardware outcome. -->

### Q26 — Shared understanding and next action

Once your selections/notes above are settled, do the recorded rounds capture our
shared understanding well enough to publish the spec and dependent GitHub tickets,
then start feasibility-first Rust work using the reusable components?

➡️ Recommended: yes if the choices and notes capture what you want. Publish the
agreed design, use the agreed seams, and keep the full build behind Q24's gate.
Any answer that opens an unresolved branch will be clarified before dependent work.

- [X] Yes: the settled choices/notes capture shared understanding; publish the spec/tickets and begin the agreed feasibility work. **Recommended.**
- [ ] I want to review/correct the design before that next action; explain below.

**Notes:**

<!-- If you have corrections, put them here or under the relevant question. -->

This confirmation follows the installed [grilling skill](../.agents/skills/grilling/SKILL.md):
“Do not act on it until the user confirms you have reached a shared understanding.”
The test-boundary agreement follows the [tdd skill](../.agents/skills/tdd/SKILL.md):
“Test only at pre-agreed seams.” The concrete design and proposed boundaries are
above so these confirmations apply to reviewable choices.
