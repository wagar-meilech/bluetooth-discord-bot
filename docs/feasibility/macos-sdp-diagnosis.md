# Mac SDP rejection diagnosis

BDB-005 / GitHub #3. Begun 2026-11-09 after the user requested continued diagnosis.
The feedback loop is the public publisher returning a record with the exact
requested PSM and successful own-record cleanup, versus nil/exit 5. This loop
only tests a publication prerequisite; it does not establish audio reception.

## Hypotheses and controlled cases

| Rank | Hypothesis | Prediction | Current evidence |
| --- | --- | --- | --- |
| 1 | CLI identity/TCC context | Matched own-process identity/authorization context or a controlled launch-context change may differ | Outside tool sandbox, own bundle ID/purpose string are visible and the CoreBluetooth class getter returns allowed-always 3; this does not certify Classic IOBluetooth publication |
| 2 | Publisher key interpretation | Key-only normalization may permit the same AudioSink/PSM record | One bare-decimal-key attempt still returned nil, exit 5 |
| 3 | AVDTP/profile/assigned-PSM restriction | A generic custom-UUID L2CAP dynamic-PSM record may publish while the audio record fails | Generic dynamic PSM0x1001 succeeds; changing only the requested PSM to0x19 returns nil, without A2DP-specific attributes |
| 4 | General SDP publication/admission failure | Generic publication may fail too | The generic control succeeded, so a blanket inability to publish in this CLI context is contradicted |

The existing [baseline](macos-receiver-results.json) retains the original
hex-labelled-key publication failure. No identical baseline mutation was repeated.
The bare-key fixture changes only the top-level keys to `1`, `4`, `5`, `9`, `256`,
and `785`. A local plist comparison verified every value tree equals the original.
Per-attribute parsing validates those values, not the publisher's key parser.
The candidate spelling follows current Chromium source; it is not claimed as a
universal Apple grammar.

Exact structured stdout, stderr and exit status are saved chronologically in
[the diagnosis results](macos-sdp-diagnosis-results.json). All diagnosis
runs explicitly used `require_escalated`, outside the tool's default sandbox.
No manager was instantiated for the authorization check. No raw device
identifiers, audio, credential data or environment dumps were captured.

## New modes

- `--diagnose-context`: read own NSBundle identifier, whether the bundle path ends
  in `.app`, whether the Bluetooth purpose key is present, and the public
  CoreBluetooth **class** authorization getter. No scan, advertisement, manager
  creation, or consent request is performed by the code.
- `--probe-sdp-bare-keys`: explicit transient AudioSink publication using identical
  attribute values and requested AVDTP PSM `0x19`, changing only the key strings.
  Existing owned-record cleanup and exact-PSM verification remain unchanged.
- `--probe-generic-l2cap`: explicit general-publication control using only
  bare service-class/protocol keys, custom UUID
  `8251d852-24b8-4e37-968a-1787ec4f204b`, L2CAP, and requested dynamic PSM `0x1001`.
  It uses the same own-record removal and PSM checks. It is not an audio service,
  and success does not establish A2DP or clear the receiver gate.
- `--probe-generic-avdtp-psm`: the same generic fixture with only the PSM integer
  changed to assigned `0x19`; immediate own-record cleanup remains unchanged if
  publication returns an object. This case returned nil in the recorded run.

The original `--probe-sdp` fixture/behavior is preserved as a baseline. Nothing
silently substitutes the generic control or changes required product transports.
All publication modes are explicit, bounded experiments and must not be retried
unchanged after a denial/hang merely to force a result.

## Interpretation so far

The generic custom-UUID L2CAP control returned a published record on the
requested PSM `0x1001`, read back that exact value, and removed its owned record
with IOReturn 0. Power/class getters matched before and after; this does not
prove peripheral/audio coexistence. The assigned-PSM generic fixture retains the
same custom UUID, keys, and every value except the explicitly typed PSM integer,
changed from `4097` (`0x1001`) to `25` (`0x19`). Restoring only that integer makes
the two parsed fixture trees identical. The assigned-PSM case returned nil and
exit 5, with no returned record or cleanup call.

Adding diagnosis modes rebuilt the executable between the initial control runs,
which could alter linker-generated ad-hoc identity. A final controlled pair
closed that confound: one unchanged binary, the same CLI launch context, explicit
`require_escalated` policy, and class authorization 3 were used for dynamic
`0x1001` followed by assigned `0x19`. The results repeated: dynamic record returned,
exact PSM and cleanup 0; assigned PSM returned nil/exit 5. SHA256 before and after
both was `43060259244209b31b3ac2846ffcc8ac35cb6284d3cfdf04e1cbb9a1c9bfc157`.
Exact stdout/stderr, timestamps, context and hashes are retained in
[the same-binary paired confirmation](macos-sdp-paired-confirmation.json).

The observed failure therefore depends on the requested PSM even without
AudioSink/profile descriptors. More precisely, the public high-level publication
method does not return a service-record object for `0x19` in this environment.
That does **not** establish that the underlying SDP server categorically rejected
addition, a documented blanket API ban, a particular system owner's conflict,
or permission policy as the exact cause. No attempt was made to displace the
host handler or change its state. The receiver bug is not fixed and the live
audio/coexistence gate remains unpassed.

The key-only change did not resolve the observed rejection. It returned no owned
record, so there was no cleanup call and no observed post-failure controller
snapshot. CoreBluetooth authorization 3 does not identify why Classic SDP
publication returned nil. A sandboxed context can report different authorization
from an unsandboxed one; launch-context comparisons must use matching tool policy
and the same binary before attributing a change to app packaging.

Native phone audio, receiver ownership, source/peripheral coexistence, and the
complete two-platform hardware gate remain untested/unpassed. No radio reset,
service replacement, channel takeover, discovery, pairing, routing, TCC reset,
or private API was used.

## Primary sources

- [Apple publisher contract](https://developer.apple.com/documentation/iobluetooth/iobluetoothsdpservicerecord/publishedservicerecord(with:)) and the installed SDK's `IOBluetoothSDPServiceRecord.h`.
- [Chromium key construction and typed L2CAP publication](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/device/bluetooth/bluetooth_socket_mac.mm).
- [Apple class authorization getter](https://developer.apple.com/documentation/corebluetooth/cbmanager/authorization-swift.type.property).
- [Apple command-line tool Info.plist/signing support](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html#//apple_ref/doc/uid/TP40005929-CH4-SW6).

## Independent environment and framework observations

System Information reports an Apple BCM_4387 controller, powered on, using PCIe.
Only chipset/state/transport/vendor were retained; no device names, addresses,
or pairing records were saved. This supports the presence of onboard hardware
independently of the framework's default-controller getter.

A minimal ad-hoc-signed copy of the probe was packaged as an app in a temporary
directory for a **context-only** check. With the default command sandbox its
CoreBluetooth class getter reported denied (2). Outside that sandbox both the
CLI and app copy reported allowed (3). The earlier denied result is therefore a
launch-policy confound, not evidence that the operator denied the app. These are
context observations, not a matched native-publication comparison or proof that
signing repairs SDP. No manager or new permission request was created.

Probe-only unified logs during a nil-returning attempt say `TCC is approved`,
`Add Service Dict`, `Added service with handle <opaque handle>`, then
`IOBluetoothGetLocalServices`. Treat the add message as a stage observation,
not proof that the full public call returned an owned receiver.

Read-only disassembly of this host's public wrapper in a probe stopped before
main shows it returning nil when its helper reports an error or supplies no
record. Inspection of that helper's existing implementation shows a lower-layer
add followed by lookup of the returned handle among local service records; a
missing match takes a generic-error path. No private selector was invoked by
the debugger or added to product code. This control flow is consistent with a
post-add lookup/materialization failure; the precise runtime error/branch was
not observed directly.

An attempted breakpoint capture of that native return did not reach publication:
the debugger-launched probe observed controller power 0 and exited 3, while
direct runs observed power 1. The program's check was not bypassed. This changed
debugger context did not reproduce the established loop and is not evidence that
the real controller was off or that a particular native error code occurred.

## Standard profile requirement and implementation consequence

A2DP v1.4.1 section 5.3, Table 5.7 marks the sink record's L2CAP parameter
`PSM=AVDTP` mandatory. SIG Assigned Numbers section 2.5 assigns AVDTP `0x0019`.
Consequently the successful generic `0x1001` control cannot be substituted for
standard phone-speaker compatibility. [A2DP specification](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/a2dp_v1-4-1_1752513648/A2DP_v1.4.1/out/en/index-en.html),
[assigned PSMs](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Assigned_Numbers/out/en/index-en.html).

The current public-API backend cannot obtain the owned standard-PSM service record
needed to proceed safely to its receiver listener. An identified alternative
public backend or a resolved framework behavior is still needed. This conclusion
is about the tested path, not universal Mac impossibility or a proven service
owner. The code does not adopt an ambiguous system channel, depend on log-derived
handles, or call an undocumented replacement API.

Windows feasibility can advance independently using its documented receiver
API. Both native paths and the installed source-to-listener system must still
pass before full product implementation.

## Validation and scope

The final code passes `cargo fmt --all -- --check`,
`cargo clippy --locked --workspace --all-targets -- -D warnings`, and
`cargo build --locked --workspace`. The probe retains only explicit, fixed
experimental cases; no arbitrary service/PSM input or listener was added. No
temporary debug logs or session/control implementation was introduced. Tests at
a new FFI-wrapper seam would not reproduce this real OS path; the approved
installed-system seam and captured paired commands supply the actual evidence.
Product regression coverage remains at the previously agreed public session/input
boundary when that implementation is reached.
