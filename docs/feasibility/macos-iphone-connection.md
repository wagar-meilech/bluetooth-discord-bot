# Mac ↔ iPhone preparation and connection evidence

BDB-005 / GitHub #3. Operator observations, offline preparation, and real phone
signaling are distinct evidence. Omit device names, Bluetooth addresses, PINs and
credentials from saved results.

## Observed preparation

The source is a nearby iPhone, initially unpaired. Normal Bluetooth Settings
pairing reported “not supported”. The first read-only inventory contained 14
cached paired devices, one connected, with zero cached-name iPhone hints. A
renamed phone could lack that hint, so these counts do not establish identity.

The first explicit OS pairing-panel launch exited 101 before usable modal UI:
the generated binding expected return encoding `q`, while that concrete runtime
reported `i`. The corrected helper verifies each actual object's signature and
uses the matching i32/isize public call; it preserves objc2 message checks.

The corrected panel returned -1000 (modal success), one selected result, and
immediate `isPaired=false`; the helper exited 3 (`pairing_not_verified`). The
operator reported “Paired, didn't stay connected I think”. A later fresh process
reported 15 paired entries, all `isPaired=true`, one connected. This discrepancy's
cause is unclassified: the extra entry alone does not independently correlate
the selected phone's identity. Actual sanitized outputs and their original binary
hashes are in [the preparation artifact](macos-iphone-preparation-results.json).

The operator subsequently confirmed the Mac appears in iPhone Settings →
Bluetooth → My Devices as **Not Connected**. This supports phone-side stored
pairing. The next native probe still requires exact selected-device correlation
to the Mac's fresh paired inventory before it opens a channel. The operator was
briefly unavailable after preparation and then returned; no phone tests were
performed during that absence.

## Helper commands

Use the repository's pinned Rust toolchain:

```sh
cargo build --locked -p macos-receiver-probe
target/debug/macos-receiver-probe --list-paired
```

`--list-paired` reads public cached inventory and paired/connected state only.
Stdout contains sanitized slot/state JSON. Stderr contains cached names for
**local display only**; do not save or publish it. Slots belong to that invocation,
not persistent identity. This mode does not request Bluetooth addresses.

```sh
target/debug/macos-receiver-probe --pair-device-panel
```

This explicit mode opens the OS-managed pairing panel. Keep iPhone Bluetooth
Settings open, select it locally, and approve any matching code on both devices.
The OS panel may start discovery. Options 0 does not request DisconnectWhenDone;
clearing this panel's UUID filter changes no global Bluetooth services. The
helper does not auto-select a device, confirm a PIN, disconnect other devices,
or open an audio channel. Cancellation, modal success, and immediate returned
paired-state verification are reported separately.

```sh
target/debug/macos-receiver-probe --discover-phone
```

This explicit mode checks controller power and CoreBluetooth authorization,
then opens the public OS device picker. Select the paired iPhone locally.
The picker may start OS-managed discovery but is not the pairing panel. The
retained selection must correlate to one fresh paired identity, using public
object equality and exact address comparison **only in process memory**. Multiple
records/references may represent that same identity: all must have the selected
valid non-wildcard address and consistent paired/disconnected state before one
fresh object is retained. Conflicting identities or states fail closed. The
helper never changes the OS pairing store. Names and addresses are not emitted.

After the modal picker the helper rechecks power, authorization, paired state,
and `isConnected == false`. It refuses an existing ACL rather than adopting or
closing a system signaling channel: AVDTP permits one signaling channel per
device pair. If necessary, disconnect only the phone manually before trying a
reviewed test. No radio setter or automatic disconnect is used. CoreBluetooth
status 3 is a context check, not proof of Classic Bluetooth channel permission.

The helper verifies the public `openL2CAPChannelAsync:withPSM:delegate:` ABI and
supplies its own retained delegate before the call. It checks the returned
channel is outgoing, has requested PSM 0x19 and that exact supplied delegate.
The public API's new-channel contract and immediately disconnected snapshot are
the experimental ownership basis. Static implementation inspection found that
an initializer can bind an underlying channel; this API is not an atomic
force-new reservation against concurrent OS activity. Contradictory ownership
results are not adopted or closed.

A Rust-only 25-second process watchdog is armed **before native open** and stays
active through native validation, run-loop callbacks, writes and cleanup. If
native progress blocks, expiry does only POSIX `_exit(6)`, terminating this
helper. It performs no fatal-path allocation, stdout locking/writing/flushing,
Objective-C access or native atexit work. A forced exit 6 may therefore have no
final JSON; the collector must record deadline termination and native cleanup
as unconfirmed. This keeps output backpressure independent of the deadline.
The operator's modal selection precedes this native operation deadline.

The main run loop waits at most 20 seconds after a returned channel. The only
request is AVDTP DISCOVER `[0x10, 0x01]`; no capability negotiation, media
configuration or streaming command is sent. Wrong transactions and unsolicited
commands are counted/ignored while the outstanding response waits. Oversize
responses, queue overflow, unsupported fragments, malformed responses and native
errors fail closed. An accepted response without an audio source SEP proves no
usable audio source. Even a source SEP is signaling evidence only.

Cleanup detaches the delegate and requests close on this owned channel only.
Close-request acceptance and observed native closure are distinct report fields.
The owned channel, delegate and two-byte request stay retained until this one-shot
process exits to protect pending callbacks/writes. `closeConnection` is never
called. No incoming-channel notification registration or SDP advertisement is
installed by this mode.

## Offline packet replay

```sh
target/debug/macos-receiver-probe --decode-discover 12010400
target/debug/macos-receiver-probe --decode-discover 12c10507
target/debug/macos-receiver-probe --decode-discover 22010400
target/debug/macos-receiver-probe --decode-discover 130111
target/debug/macos-receiver-probe --decode-discover 160201
```

These public diagnostic invocations make no Bluetooth calls. The known-spec
examples distinguish a source SEP, ignored reserved bits, wrong transaction,
rejection, and unsupported fragmentation. [Recorded manual replay outputs](macos-avdtp-offline-replay.json)
include additional malformed, duplicate, empty, command and invalid-hex cases.
This diagnostic preparation is separate from tests at the agreed public
bridge-session/input boundary and from real installed-system evidence.

## Operator result fields

- [x] OS panel returned modal success; immediate selected result was unpaired.
- [x] A fresh inventory later contained one additional paired entry; selected
  identity remained uncorrelated at that point.
- [x] iPhone My Devices lists the Mac as Not Connected (operator report).
- [x] A DISCOVER picker selected the intended iPhone and fresh identity matched
  in the grouped legacy retry; the subsequent native open did not return.
- [ ] Helper refused an existing connection or other preparation prerequisite.
- [ ] Fresh channel opened and a DISCOVER response was received.
- [ ] Another result; add exact sanitized status/error below.

**Connection-test result / notes:**

The first reviewed DISCOVER-mode run returned modal selection success (-1000)
and selected-object `isPaired=true`, then failed exact fresh-inventory correlation
before opening any channel (`connection_started=false`, exit 3). That binary did
not report whether there were zero or duplicate matches. Its actual hash/output
is preserved in [the connection-attempt artifact](macos-avdtp-connection-results.json).
A second diagnostic run found two fresh records/references matching both object
equality and the exact selected address, among 15 inventory entries. It also
stopped before connection (exit 3). The probe had incorrectly equated multiple
matching records with multiple identities. This does not show different devices
or pairing-store corruption. The corrected build groups the exact selected
non-wildcard address only when all matched records agree on paired/disconnected
state. It preserves raw match counts and reports distinct identity count.

The grouped retry reached the deprecated public open call, which blocked inside
`openL2CAPChannelSync` / `waitforChanneOpen` with nested callbacks, contrary to its
SDK asynchronous wording. A coordinator sample inspected only the owned helper
process. The Rust timer was after the call and never reached. After approximately
164 seconds the coordinator terminated only that helper (SIGTERM, exit -15),
which produced no stdout JSON. No returned channel was observed, no DISCOVER was
submitted, and cleanup is unconfirmed. The original wrapper result and sanitized
public-method stack summary are preserved in the connection artifact.

The current build replaces that legacy call with modern Async and the pre-call
hard process deadline described above. Its reviewed native retry is waiting in
the operator's device picker; no outbound DISCOVER packet has yet been sent.

## Evidence status

| Layer | Current evidence |
| --- | --- |
| Read-only inventory | Observed 14 entries initially, 15 after panel; selected identity not established by counts |
| OS pairing panel | Modal -1000; immediate selected isPaired=false; exit3; later phone-side stored pairing reported |
| Offline AVDTP replay | 12 manual known-spec/invalid packet cases matched expected decode statuses/exits |
| Outbound DISCOVER helper | Two identity failures before open; grouped legacy-open run blocked natively and was terminated; modern Async/watchdog build prepared |
| Live phone audio / coexistence / full gate | Not passed |

Standard A2DP sink SDP advertising and sink acceptor support remain mandatory for
the complete phone-as-speaker product. Local standard-PSM publication remains
unresolved. Outbound signaling cannot waive that requirement or prove that the
phone's audio picker will expose this host.

## Build checks

The pinned toolchain passed these commands (exit 0):

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --locked -p macos-receiver-probe
```

The offline replay artifact preserves the tested binary's hash. Actual operator
runs preserve their own binary hashes independently of later source builds.
Windows-target workspace Clippy with static CRT also passed after the Mac-only
request constant was conditionally compiled for macOS.

## API references

- [Public pairing panel](https://developer.apple.com/documentation/iobluetoothui/iobluetoothpairingcontroller).
- [Rust pairing bindings](https://docs.rs/objc2-io-bluetooth-ui/0.3.2/objc2_io_bluetooth_ui/struct.IOBluetoothPairingController.html).
- [Public outbound L2CAP API](https://developer.apple.com/documentation/iobluetooth/iobluetoothdevice).
- [A2DP profile requirements](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/a2dp_v1-4-1_1752513648/A2DP_v1.4.1/out/en/index-en.html).
