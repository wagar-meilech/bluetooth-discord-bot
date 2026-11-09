# Windows receiver baseline results

Status: awaiting an operator run on real Windows hardware. This is the first
receiver baseline; it does not pass isolated capture, silent forwarding, Discord
delivery, or the full two-platform gate.

Download and extract the [verified x86_64 artifact](https://github.com/wagar-meilech/bluetooth-discord-bot/actions/runs/42543599395/artifacts/13642297887).
The [full instructions](windows-receiver-probe.md) cover pairing, source selection,
Ctrl+C cleanup, and sanitized reports. Rust, Docker, and a bot token are not needed.

## Hardware and baseline

- Windows version/build:
- Host model (no serial number):
- Built-in Bluetooth radio confirmed:
- Source phone model/OS:
- Headphones/headset microphone and other peripherals tested:
- Normal Discord-call behavior before starting the receiver:

## 1. API preflight

In PowerShell in the extracted directory:

```powershell
.\windows-receiver-probe.exe --preflight | Set-Content -Encoding utf8 preflight.jsonl
$LASTEXITCODE
```

Exit code:

Paste sanitized stdout from `preflight.jsonl` below:

```json
```

## 2. Pair and select your phone

Pair your intended phone using Windows Settings, keeping your other Bluetooth
devices connected. Then run:

```powershell
.\windows-receiver-probe.exe --list-sources
```

Selected phone index:

If listing is empty or fails, paste only its JSON stdout here. Friendly names
appear on stderr for local selection; they are not needed in this file.

```json
```

## 3. Receive for 60 seconds

Replace `0` below with your phone's current index. Start music on the phone and
select the Windows host as its Bluetooth audio output when offered.

```powershell
.\windows-receiver-probe.exe --receive-index 0 --duration-seconds 60 | Set-Content -Encoding utf8 receiver-baseline.jsonl
$LASTEXITCODE
```

This baseline plays the incoming music locally. To finish early, press Ctrl+C.

Exit code:

Paste sanitized stdout from `receiver-baseline.jsonl`:

```json
```

Operator observations (leave unchecked until observed):

- [ ] The host was available as a phone Bluetooth audio output.
- [ ] Phone music was heard through Windows.
- [ ] The source kept streaming while other required Bluetooth devices were used.
- [ ] Headphone output and headset microphone remained usable relative to the baseline call.
- [ ] The receiver reported successful cleanup on duration expiry or Ctrl+C.

Failures, interruptions, mode changes, or other notes:

## 4. Capture-endpoint lead

While a receiver baseline is active, run this in a second PowerShell window:

```powershell
.\windows-receiver-probe.exe --list-capture-endpoints
```

Paste its JSON stdout only:

```json
```

Was an A2DP-named capture candidate shown locally? This is an inventory clue,
not proof that isolated PCM capture works.

## Build provenance

- CI run: `42543599395` — completed successfully on Windows.
- Artifact ID: `13642297887`, `windows-receiver-probe-x86_64`.
- PR merge source built: `b774f9fbed08d235cab0c2a9ad2918e07b0aa230`.
- Executable: x86_64 PE, 416768 bytes, static MSVC CRT requested at build.
- Verified executable SHA256: `f0da0aae154f6b14e66f48de948389670c163f531ca86167ad4d691697be7405`.
- Operator Windows runtime/audio/coexistence outcomes remain untested until filled above.
