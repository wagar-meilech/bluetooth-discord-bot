# Existing Bluetooth-to-Discord solutions

Research date: 2026-11-09. Scope: a phone pairs with a computer like a Bluetooth
speaker, then a normal bot forwards that incoming audio into a Discord server
voice channel. Required hosts are macOS and Windows; Rust is the requested
implementation language.

## Finding

**No integrated service meeting that complete requirement was found in this
bounded search.** Existing tools cover Bluetooth reception or live audio input
to Discord, and some could be composed on Windows. That is a candidate to test,
not a verified working Bluetooth-to-Discord solution. None of the reviewed
projects establishes native macOS Bluetooth reception plus Discord forwarding.
This is a search result, not proof that no such service exists.

## Candidates

| Project | What the primary source establishes | Fit and limitation |
| --- | --- | --- |
| [AudioPlaybackConnector2](https://github.com/N0ahTM/AudioPlaybackConnector2) | A per-user Windows tray application that receives paired Bluetooth-source audio through `AudioPlaybackConnection`. | A native Windows Bluetooth receiver building block. It does not document Discord streaming or a macOS receiver. |
| [Discord Audio Stream Bot, maintained continuation](https://github.com/vincentberry/Discord-Audio-Stream-Bot) | Java desktop application with Windows/macOS/Linux builds, input-device selection, channel controls and slash commands. Application audio is routed through a virtual audio cable. Its [build manifest](https://github.com/vincentberry/Discord-Audio-Stream-Bot/blob/master/build.gradle) includes JDA 6.3.1 and libdave adapter/JNI/native dependencies. | Strong candidate for evaluating the existing live-input-to-Discord half, including explicit DAVE-related dependencies. No Bluetooth receiver integration was established. Dependencies and advertised builds do not prove current end-to-end operation. |
| [RadzuPL/discord-stream-bot](https://github.com/RadzuPL/discord-stream-bot) | A containerized Python bot that sends a configured `STREAM_URL` to one ordinary voice channel. Its README explicitly describes discord.py 2.7+ and `davey` for DAVE, plus reconnect handling observed in October 2026. | Candidate existing container for the Discord output half if a receiver exposes a suitable live stream. It does not capture Bluetooth or native host application audio. Not Rust. |
| [ToothSpin](https://github.com/eknoes/toothspin) | Actual phone-as-Bluetooth-speaker input, distributed through Music Assistant or Sendspin. Supports Docker/Podman on a Linux host with BlueZ; its documentation requires no competing host Bluetooth audio stack. | Useful reference for the Bluetooth receiver behavior and pairing UX. No Discord output is documented; the supported Linux/BlueZ setup does not satisfy native macOS/Windows built-in Bluetooth requirements. |
| [Aux Cable](https://tangled.org/notjustanna.tngl.sh/aux-cable) | Self-hosted application streaming computer audio inputs into Discord, including another device through line-in; can run headless. | Another live-input bot, not a Bluetooth receiver. Current DAVE compatibility was not established in the material inspected. |
| [discord_stream_bot](https://github.com/ygorpontelo/discord_stream_bot/) and [discord-radio-bot](https://github.com/amishshah/discord-radio-bot) | PyAudio input-device streaming in the former; an FFmpeg hardware-audio proof of concept for Linux/Windows in the latter. | Historical examples of the capture-to-voice pattern. Neither reviewed README establishes an integrated Bluetooth receiver or present DAVE compatibility. Do not assume current operation or call them broken without testing. |

## Why current voice support matters

Discord's official voice documentation says E2EE became required for voice
conversations on March 1, 2026. A project that streamed audio successfully before
then may need substantial updates. The reviewed Java continuation has explicit
DAVE dependencies, and the container bot documents DAVE support, but neither was
connected to a real Discord channel during this research.
[Discord voice documentation](https://docs.discord.com/developers/topics/voice-connections#end-to-end-encryption-dave-protocol).

## What could be reused

1. **Evaluate a composed Windows path first as a reference:** an existing Windows
   Bluetooth receiver, an isolated capture/routing path, and a DAVE-capable
   live-input Discord bot. Microsoft's receiver API plays incoming Bluetooth
   audio through Windows endpoints; that does not by itself establish an isolated
   recording source for the bot. Capture must be tested without rebroadcasting
   Discord voices or unrelated desktop sounds.
   [Microsoft's receiver guide](https://learn.microsoft.com/en-us/windows/apps/develop/media-playback/enable-remote-audio-playback).
2. **Keep native macOS Bluetooth as a separate feasibility gate.** Adopting an
   existing Discord bot only replaces the output half; it does not demonstrate
   the required receiver behavior on the Mac. A receiver marketed for multiple
   platforms may use a different transport: for example,
   [Noxy229/BluetoothAudioReceiver](https://github.com/Noxy229/BluetoothAudioReceiver)
   explicitly labels its Mac edition an AirPlay setup companion, not a Bluetooth
   receiver.
3. **Treat these as references or prototype components, not an automatic language
   change.** The user requested Rust. Adopting a Java/Python application, or
   relaxing the Bluetooth/host requirements to use a Linux receiver, needs an
   explicit product decision. No such change has been made.

## Search and verification limits

Searches combined Bluetooth/A2DP sink, Discord bot/voice channel, audio streaming,
and GitHub; additional searches inspected native Windows receiver projects.
Only project-owned READMEs, available source manifests, and official platform
documentation support the findings. Several raw/dependency-file fetches failed;
absence of inspected DAVE evidence is therefore recorded as unknown rather than
non-support. No projects were installed, no hardware was paired, no containers
were run, and no Discord voice connection was tested. Repository metadata or
recent documentation is not a maintenance guarantee.

## Runtime and Bluetooth findings for this design

**Other Bluetooth devices do not by themselves require another host identity.**
Bluetooth's controller specification supports multiple simultaneous connections;
the connections share radio time. A second advertised name would not add radio
capacity. Receiving phone music while sending audio to Bluetooth headphones is
a specific simultaneous A2DP sink/source case, which the reviewed OS sources do
not guarantee. Include it in the prototype if required. Windows also documents
headset mode changes when a microphone or communications playback uses HFP, so
stereo-headphone use and headset-microphone use need separate validation.
[Bluetooth controller specification](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Core_v7.0/out/en/br-edr-controller/baseband--bb--specification.html),
[Windows Classic Bluetooth audio](https://learn.microsoft.com/en-us/windows-hardware/drivers/bluetooth/bluetooth-classic-audio).

**A permanent cloud server is unnecessary for the local-bot design.** This is an
architecture inference: Discord's Gateway uses an outbound WebSocket, interactions
can arrive through that connection, and voice uses its own WebSocket/UDP path.
The local process must remain running and connected while serving commands and
playing audio; bot registration does not require an always-on cloud runtime.
[Discord Gateway](https://docs.discord.com/developers/events/gateway),
[interaction delivery](https://docs.discord.com/developers/interactions/receiving-and-responding#receiving-an-interaction),
[voice connections](https://docs.discord.com/developers/topics/voice-connections).

**Docker has a native-host boundary.** Docker Desktop runs Linux containers in a
Linux VM on these hosts; the container cannot directly call macOS IOBluetooth/Core
Audio or Windows WinRT/WASAPI. Docker's USB/IP workaround does not demonstrate
built-in-radio or Bluetooth-audio access and is not guaranteed for every device.
An architecture to evaluate is a native Rust capture/Bluetooth adapter feeding a
Rust Discord core in a local container. This is an inference and a proposed split,
not an approved design or a fix for native macOS receiver feasibility.
[Docker Desktop VMM](https://docs.docker.com/desktop/features/vmm/),
[Docker USB/IP](https://docs.docker.com/desktop/features/usbip/),
[host/container networking](https://docs.docker.com/desktop/features/networking/).

## Rust frameworks to evaluate

Use established libraries for Discord protocol and command handling, keeping
platform-specific receiver work separate. These are candidates to evaluate,
not installed or pinned dependencies:

- [Serenity](https://docs.rs/serenity/latest/serenity/) for Discord Gateway/REST.
- [Poise](https://docs.rs/poise/latest/poise/) for commands on Serenity.
- [Songbird](https://docs.rs/songbird/latest/songbird/) for voice and custom audio
  inputs. Its 0.6.0 release introduced DAVE support; the reviewed 0.6.1 manifest
  includes its compatible `davey` dependency. Let the voice library select that
  dependency instead of independently forcing the newest version.
  [Releases](https://github.com/serenity-rs/songbird/releases),
  [reviewed manifest](https://github.com/serenity-rs/songbird/blob/v0.6.1/Cargo.toml).

No Rust platform adapter, container networking path, or Discord connection was
executed during these documentation checks. Framework reuse does not establish
the Bluetooth receiver or audio-isolation behavior on either host.

## Concrete reuse plan after round 3

The user explicitly requested reusable components to save time. Start with the
following building blocks and validate compatibility during the feasibility work:

| Area | Reuse | Custom integration still needed |
| --- | --- | --- |
| Discord connection and commands | Serenity and Poise | Trusted-user policy, channel selection, and application command behavior |
| Voice, DAVE, Opus, and PCM input | Songbird, including its `RawAdapter` | Feed bounded live audio, source lifecycle, gain/monitoring policy, and session recovery |
| Windows capture and receiver APIs | Microsoft's `windows` Rust bindings | Receiver setup, discovering the usable input, isolation, and monitoring control |
| macOS capture and Bluetooth APIs | `objc2-core-audio` and `objc2-io-bluetooth` | Process-tap setup, receiver/profile feasibility, pairing lifecycle, and isolation |
| Native tray/menu shell | `tray-icon` and its `tray_icon::menu` re-export of Muda | Product status, source/settings controls, and lifecycle |

Poise 0.7.1's reviewed manifest requires Serenity `^0.12.5`; Songbird 0.6.1 accepts
Serenity `^0.12.0`. Serenity 0.12.5 therefore satisfies both at the manifest level.
Compile and resolve a lockfile before calling this combination verified. Songbird's
released `RawAdapter` accepts unknown-length interleaved `f32` PCM with declared
sample rate/channels, so the bridge does not need its own Discord codec/input stack.
[Poise manifest](https://github.com/serenity-rs/poise/blob/current/Cargo.toml),
[Songbird manifest](https://github.com/serenity-rs/songbird/blob/v0.6.1/Cargo.toml),
[PCM adapter source](https://github.com/serenity-rs/songbird/blob/v0.6.1/src/input/adapters/raw_adapter.rs).

The Windows bindings expose both the receiver API and capture interfaces. Use
those projections rather than handwritten FFI. The macOS bindings include Core
Audio process-tap APIs and IOBluetooth interfaces; **bindings are not an A2DP sink
implementation**. Preserve the native Mac feasibility gate.
[Windows receiver binding](https://microsoft.github.io/windows-docs-rs/doc/windows/Media/Audio/struct.AudioPlaybackConnection.html),
[Windows capture binding](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Media/Audio/struct.IAudioCaptureClient.html),
[Core Audio tap binding](https://docs.rs/objc2-core-audio/latest/objc2_core_audio/fn.AudioHardwareCreateProcessTap.html),
[IOBluetooth bindings](https://docs.rs/objc2-io-bluetooth/latest/objc2_io_bluetooth/).

Use the tray library's menu re-export to avoid mixing incompatible menu versions;
macOS tray/event handling needs the main thread. The tray framework already
supports both target platforms, so a new GUI toolkit is unnecessary for the basic
tray/menu behavior. Rich settings UI can be selected during implementation if
needed. [Tray documentation](https://docs.rs/tray-icon/latest/tray_icon/).

A first-party Windows application describes using a `Line (A2DP SNK)` recording
endpoint and muting original playback for its audio effects. This is a useful
prototype lead to check before creating a virtual driver. It is an application
author's report, not a Microsoft guarantee or a verified path on the user's PCs.
[Bluetooth Audio Sink author](https://infiartt.com/projects/bt-audio-sink?lang=en).

Existing Java/Python bots and the Windows receiver application remain useful
references or diagnostic comparison tools. Adopting them as the shipped runtime
would change the Rust implementation requirement. No such language change is
assumed, and no third-party source code was copied during this research.
