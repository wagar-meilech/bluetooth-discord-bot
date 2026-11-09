# Windows receiver diagnostic

This is a Windows 11 x86_64 receiver baseline for BDB-006 / [GitHub issue #4](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/4).
It uses Microsoft's released `windows` bindings for `AudioPlaybackConnection`.
It does not capture audio frames, forward to Discord, mute playback, change
routing, pair devices, reset the radio, or pass the full feasibility gate.
The explicit receive mode **plays the incoming phone audio locally through
Windows**. Silent forwarding and isolation remain untested.

## Download and run

Download `windows-receiver-probe-x86_64` from the feature PR's
**Windows receiver probe** GitHub Actions run, and extract the ZIP. It contains
`windows-receiver-probe.exe`, these instructions, `SHA256SUMS.txt`, and build
metadata. Rust, Docker, an administrator shell, and a bot token are not required.
The executable is built for x86_64 Windows with the MSVC CRT linked statically.
An artifact/build success is not a Windows Bluetooth hardware result.

Open PowerShell in the extracted directory. Optionally compare the executable's
hash with `SHA256SUMS.txt`:

```powershell
Get-FileHash .\windows-receiver-probe.exe -Algorithm SHA256
```

Run the nonmutating default check first:

```powershell
.\windows-receiver-probe.exe --preflight | Set-Content -Encoding utf8 preflight.jsonl
$LASTEXITCODE
```

Preflight initializes only this process's MTA WinRT apartment and inspects its
apartment, Windows version, and receiver type availability. It performs no
radio/device enumeration, inquiry, pairing, or receiver activation. The program
requires Windows 11 build 22000 or newer and the receiver API. Exit code `0`
means the requested API inspection or baseline finished; `2` means an argument,
platform, API, timeout, or activation failure. Neither exit code passes audio
acceptance. A preflight report can safely be shared as diagnostic evidence.

## Pair and select your own source

1. Record your current Windows version and intended host/source models in your
   private hardware notes. Confirm the built-in Bluetooth radio is in use.
2. Establish a baseline with your usual Bluetooth headphones, headset microphone,
   and other peripherals, including an ordinary Discord call. Keep the devices
   paired; this probe does not disconnect or reconfigure them.
3. Manually pair your intended phone in **Windows Settings → Bluetooth & devices**.
   Only the local operator approves the normal Windows pairing prompts.
4. Close any other Bluetooth receiver application to avoid conflating which
   application owns the receiver. Leave unrelated Bluetooth devices connected.
5. List eligible paired sources:

```powershell
.\windows-receiver-probe.exe --list-sources
```

The app queries `AudioPlaybackConnection.GetDeviceSelector()` with Windows device
enumeration, then accepts only entries whose `Pairing().IsPaired()` is true. It
skips unpaired/unverified entries and reports their count (and any HRESULT).
It does not start active Bluetooth inquiry or pair anything. An empty filtered
list alone does not prove the radio lacks support: device-information pairing
properties may need further diagnosis on the actual Windows machine. Friendly
names appear on the local console (stderr), with numeric indices; stdout contains
only a sanitized JSON summary. Device IDs stay in process memory. Indices are
sorted within the current paired set; rerun listing after any pairing changes.
The receive command enumerates the set again and displays the selected name
locally, so check that the indicated source is your intended phone.

## Bounded receiver baseline

For source index `0`, run a 60-second baseline, then play music on that phone and
select the Windows host as its Bluetooth audio output when offered:

```powershell
.\windows-receiver-probe.exe --receive-index 0 --duration-seconds 60 | Set-Content -Encoding utf8 receiver-baseline.jsonl
$LASTEXITCODE
```

Replace `0` with the index from your source list. Duration must be 1–1800 seconds;
each asynchronous activation/enumeration operation has a separate 30-second
limit. The app creates exactly one receiver for the selected existing source,
requests `StartAsync` and `OpenAsync`, reports their real result/HRESULT and
connection state, then waits for the bounded interval. `open_status` values are
`0` success, `1` request timeout, `2` denied by system, `3` unknown failure.
Connection `state` is `1` opened or `0` closed. An early closed state is reported
as a failure. A successful open is not a measurement of received PCM frames.

**Expect local sound in this mode.** This is the OS receiver playback path,
not the product's required silent forwarding mode. Note whether sound actually
plays, whether the phone keeps streaming, and whether your existing peripherals
continue working relative to the baseline. Do not treat a good result as proof
of isolated capture, silence, latency, or Discord delivery.

Press **Ctrl+C** in the same console to finish early. On normal timeout, Ctrl+C,
or an API error, the app calls `Close()` on only the receiver it created and
reports the close result. It then releases its WinRT objects/apartment. OS
termination, power loss, or force-killing a process cannot promise an app cleanup
report; use Ctrl+C for a measured cleanup check. If Close reports an error,
record it rather than claiming cleanup succeeded. Other applications may still
hold their own receiver references; this probe does not close those.

## Read-only capture endpoint lead

While the receiver baseline is active, open a second PowerShell window in the
same directory and run:

```powershell
.\windows-receiver-probe.exe --list-capture-endpoints
```

This reads audio-capture endpoint names and marks names containing `A2DP` as
candidates locally. It does not open a capture stream or record frames. One
existing receiver project reports a `Line (A2DP SNK)` recording input, but its
presence and suitability must be verified on your actual Windows setup. A name
match alone proves no isolation. Missing candidates are a result to investigate;
there is no desktop-loopback fallback.

## Share evidence

Share only `preflight.jsonl` and `receiver-baseline.jsonl`, plus your observations
of music, duration, cleanup, and peripheral behavior. Stdout is JSON Lines with
status, counts, enums, and HRESULTs; it omits friendly names, IDs, Bluetooth
addresses, error-message text, and audio. The program saves no files itself.
Do not merge/capture stderr (`2>&1`, `*>`, or a transcript) into a shared report;
stderr deliberately shows friendly names for local selection. Hardware model/OS
notes can be supplied separately without serial numbers or Bluetooth addresses.

Both native host paths, isolated/silent forwarding, optional monitoring, real
Discord voice, selected-app input, and the common sustained coexistence tests
remain required before the full app gate passes.

## Maintainer validation and sources

Use the pinned Rust 1.94.0 toolchain:

```text
cargo check --locked -p windows-receiver-probe --target x86_64-pc-windows-msvc
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```

A Mac can type-check the Windows target after installing its Rust standard
library, but cannot link/run an MSVC executable without the Windows toolchain.
The pull-request workflow builds the actual executable on Windows, including
for draft PRs; it can also be manually dispatched once registered on the default
branch. Hosted runners perform compilation/linting, not Bluetooth hardware acceptance.
It uses read-only repository permissions and no custom secrets. The checkout
and artifact actions are pinned to the official v7.0.1 and v7.0.2 commits.

- [Microsoft receiver guide](https://learn.microsoft.com/en-us/windows/apps/develop/media-playback/enable-remote-audio-playback).
- [AudioPlaybackConnection API](https://learn.microsoft.com/en-us/uwp/api/windows.media.audio.audioplaybackconnection).
- [Microsoft Rust bindings](https://microsoft.github.io/windows-docs-rs/doc/windows/Media/Audio/struct.AudioPlaybackConnection.html).
- [First-party Line (A2DP SNK) lead](https://infiartt.com/projects/bt-audio-sink?lang=en), not a Windows API guarantee.
- [Checkout v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1).
- [Upload Artifact v7.0.2](https://github.com/actions/upload-artifact/releases/tag/v7.0.2).
