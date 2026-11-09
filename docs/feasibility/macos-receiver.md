# Native Mac receiver prerequisite probe

BDB-005 / [GitHub #3](https://github.com/wagar-meilech/bluetooth-discord-bot/issues/3).
Observed on 2026-11-09. **The native Mac receiver gate has not passed.**

This file preserves the initial attempt. The subsequent
[controlled diagnosis](macos-sdp-diagnosis.md) demonstrates that a generic dynamic
PSM can return a record and clean up, while the same generic fixture requesting
assigned PSM `0x19` returns no record on the same binary. That narrows the failure
without proving a particular owner/restriction or passing native audio reception.

## Result

The reusable Rust bindings compiled and the corrected read-only preflight
succeeded. The single reviewed transient SDP-publication attempt returned `nil`
from `publishedServiceRecordWithDictionary:`. The probe reported
`publication_rejected` and exited **5**, with empty stderr. It did not return an
owned service-record object; no removal call was made. Controller state after
this rejection, headset behavior, peripheral coexistence, and live phone audio
were not observed. They must not be inferred from the preflight.

This is an undiagnosed local prerequisite failure, not proof that all native
macOS A2DP receiver approaches are impossible. The API's nil result does not
identify permission denial, profile/stack conflict, or another rejection reason.
No publication retry, Bluetooth reset, OS-consent bypass, channel takeover,
pairing/disconnection, or settings changes were attempted.

Structured stdout, exit statuses, and empty stderr for the final three modes
are preserved in [the evidence record](macos-receiver-results.json). No raw device
addresses, names, or audio were collected. The default-controller lookup does not
prove that the returned controller is built-in rather than external.

## Run

From the repository with the pinned Rust toolchain:

```sh
cargo build --locked -p macos-receiver-probe
cargo run --locked -p macos-receiver-probe
```

The default and `--preflight` modes inspect public selector availability and
controller power/class getters. They do not start discovery, publish services,
pair, or change power/name/class/routing. A macOS Bluetooth-consent prompt may
require the operator; the binary embeds a purpose string and never bypasses OS
consent.

A non-publishing semantic payload check is also available:

```sh
cargo run --locked -p macos-receiver-probe -- --validate-sdp
```

It passes the six bundled attribute values through the public
`IOBluetoothSDPDataElement::withElementValue` parser and reads the parsed L2CAP
UUID and PSM. The observed result was six accepted attributes and PSM 25 (`0x19`),
exit 0, with empty stderr. No controller/device lookup or server publication is
performed by this mode. This validates local attribute parsing, not server
acceptance or a functioning A2DP receiver.

The explicit prerequisite experiment is:

```sh
cargo run --locked -p macos-receiver-probe -- --probe-sdp
```

That mode **briefly attempts service advertisement**; it is not a working speaker.
It requests the assigned AVDTP PSM `0x19`, checks the returned PSM, then immediately
removes only its newly returned record before diagnostic return. A guard retries
owned cleanup on a Rust error/panic path. The record omits persistence and launch
attributes; Apple's contract also removes transient records on process exit.
There are no incoming-channel delegates or protocol/codec implementations.

The recorded native runs were bounded to 20 seconds with a subprocess timeout;
all recorded final modes completed without that timeout. Do not repeat a denied/hanging
experiment or change the host stack merely to obtain a successful result.

Exit 0 means only that the requested preflight/publication prerequisite succeeded.
It never means real reception or the complete gate passed. Exit 2 is usage or
platform mismatch; 3 means unavailable controller/API/state; 4 means a malformed
bundled definition; 5 means publication/PSM failure; 6 means explicit cleanup
failure; 7 means changed or unavailable post-cleanup controller getters. Even
unchanged power/class getters do not verify audio/peripheral coexistence.

## Implementation and validation

- Workspace and probe use the installed Rust 1.94.0 without changing the global
  default. OS was arm64 macOS 26.7.1 (25G241), with Xcode's MacOSX27.0 SDK.
- Reused `objc2`, `objc2-foundation`, and `objc2-io-bluetooth` 0.3.2 projections;
  Foundation parses the bundled plist instead of handwritten collection FFI.
- Initial preflight exposed a projection/runtime signedness discrepancy:
  `powerState` is unsigned in the SDK-generated type but advertises signed int
  in this runtime. `controller_snapshot` now verifies the runtime encoding and
  chooses an explicitly typed `objc2` call. Unknown encodings fail closed;
  signature checks are not disabled. The initial exit 101 happened before any
  publication and is not a Bluetooth-receiver failure.
- Reviewed the fixed plist against the SDK dictionary contract and Chromium's
  typed L2CAP example: AudioSink `0x110B`, L2CAP `0x0100` with 16-bit PSM 25,
  AVDTP `0x0019`/version `0x0103`, AdvancedAudioDistribution profile
  `0x110D`/version `0x0104`, Speaker feature 2. No `LocalAttributes`, persistence,
  launch target, or user-supplied service handle. This static review found no
  payload defect. The documented attribute parser subsequently accepted all six
  values and the expected L2CAP PSM; neither establishes server acceptance.
- An intermediate `withServiceDictionary:device:` local-object diagnostic could
  not find a PSM. Its equivalence to the publisher's input format was not
  established, so it is inconclusive and was removed from the final validator.
  The final validator uses the explicitly documented per-attribute parser.
- `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets
  -- -D warnings`, and `cargo build --locked --workspace` passed.
- No session/control behavior or new automatic test seam was introduced. This
  thin capability wrapper was verified at the approved installed-system seam;
  tautological FFI-wrapper tests were not added. Public session/input behavior
  will use the approved TDD boundary when that implementation is reached.

## Next evidence needed

First resolve the local publication rejection with a concrete public-API/OS
reason and the intended operator's Bluetooth-consent/baseline context. Do not
infer that another PSM, a different transport, or disabling host services meets
the specification. Only if exact AVDTP registration and baseline preservation
are established should work proceed to a safely owned incoming listener,
protocol negotiation, decoded live phone audio, and the full two-platform gate.
The full product remains behind that gate.

## Options after this result

1. **Preserve the requirements and diagnose the native rejection.** A bounded
   follow-up can establish the intended app identity and OS Bluetooth-consent
   context, inspect only relevant public diagnostics, and look for a documented
   explanation of the publication failure. This is the next investigation path;
   it is not evidence that another attempt will succeed. Avoid repeating the same
   publication without new information or changing the host stack.
2. **Revisit the Mac input requirement if the supported native path cannot be
   established.** An external Bluetooth audio receiver/capture path, or another
   Mac transport, would change the approved specification and needs an explicit
   product decision. No such fallback has been selected.
3. **Revisit platform sequencing.** Windows-first delivery would relax the user's
   requirement that both native host paths pass before the full application.
   Downstream work remains held under the current agreement.

The reviewed prerequisite code and evidence are published in
[draft PR #10](https://github.com/wagar-meilech/bluetooth-discord-bot/pull/10).
Issues #2/#3 remain open and blocked; this draft does not claim live reception.

## Primary references

The SDK public headers `IOBluetoothSDPServiceRecord.h` and
`IOBluetoothSDPDataElement.h` describe publication, possible PSM reassignment,
transient lifetime, and typed dictionary values. Online supporting sources:

- [Apple service-record publication](https://developer.apple.com/documentation/iobluetooth/iobluetoothsdpservicerecord/publishedservicerecord(with:)).
- [Apple Bluetooth usage purpose key](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothalwaysusagedescription).
- [objc2 IOBluetooth bindings](https://docs.rs/objc2-io-bluetooth/0.3.2/objc2_io_bluetooth/).
- [Chromium L2CAP service registration and validation](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/device/bluetooth/bluetooth_socket_mac.mm).
- [Bluetooth assigned numbers](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Assigned_Numbers/out/en/index-en.html).
