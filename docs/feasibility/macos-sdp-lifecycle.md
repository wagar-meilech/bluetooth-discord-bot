# Mac SDP lifecycle and explicit-handle experiments

BDB-005 / GitHub #3, 2026-11-09. This extends the
[earlier requested-PSM diagnosis](macos-sdp-diagnosis.md); it does not replace its
observations or pass the receiver gate.

## Feedback loop and hypotheses

The actual loop remains public publication returning an owned record with exact
PSM/readback and successful own cleanup, versus a nil result/exit 5. A generic
custom-UUID fixture isolates that prerequisite without claiming A2DP or opening
channels. The ranked hypotheses for this continuation were:

1. Public controller/run-loop initialization before synchronous publication.
2. Requested/returned handle assignment and service-record materialization.
3. Special handling associated with the standard assigned PSM `0x19`.

No public documentation found guarantees that run-loop warm-up fixes publication.
The experiments keep public thread affinity and preserve the baseline fixtures.

## One-second main-run-loop initialization

`--preflight-warmup` reads initial controller state, processes this process's
Foundation main run loop for a measured one-second window, then reads state again.
It never sets radio power. The standalone read-only observation was power 1 before
and after, class 0 before and after, 1,005 ms and 32 `runUntilDate` invocations,
exit 0, with no publication.

The explicit `--probe-generic-l2cap-warmup` and
`--probe-generic-avdtp-psm-warmup` modes perform that initialization before the
unchanged dynamic/assigned generic publication path. The loop calls use bounded
20 ms slices with a short backoff if no input keeps them waiting. The reported
turn count means run-loop API invocations, not a verified count of delivered
callbacks. No owned timer/source or concurrent main-loop pumping during the
synchronous publisher is claimed.

A same-binary 2x2 outside the command sandbox gave:

| Initialization | Requested PSM | Record/readback/cleanup | Exit |
| --- | --- | --- | --- |
| Cold | `0x1001` | Own record, assigned `0x1001`, removal 0 | 0 |
| Cold | `0x19` | Nil; no owned object to remove | 5 |
| One-second warm-up | `0x1001` | Own record, assigned `0x1001`, removal 0; warm-up 1,002 ms | 0 |
| One-second warm-up | `0x19` | Nil; no owned object to remove; warm-up 1,003 ms | 5 |

All observed power values were 1. CoreBluetooth class authorization was 3.
The unchanged binary SHA256 before/after was
`8c2b7c5dd623c7b664a96498e458263017f610c2b8dfcc866ddfbc580164eb3c`.
[Exact stdout/stderr/context/timestamps](macos-sdp-lifecycle-results.json).

This fixed warm-up did not repair the path. It does not rule out every lifecycle
or asynchronous materialization mechanism, and it does not test a main loop
running concurrently while the synchronous method executes.

## Fresh requested service-record handle

Two fixtures add only public attribute zero as a typed unsigned four-byte value:
`DataElementType=1`, `DataElementSize=4`, candidate `0x6A2D0001` (1781334017).
That value is in the nonreserved application range and below the signed 32-bit
boundary; it is a candidate, not a claim of global uniqueness. No OS/log-derived
handle is used. The dynamic and assigned fixtures differ only in their typed PSM
integer; restoring that integer makes their parsed trees identical.

`--probe-generic-l2cap-own-handle` requests dynamic `0x1001`, and
`--probe-generic-avdtp-own-handle` requests assigned `0x19`. The public
`getServiceRecordHandle` getter is used only on an object actually returned by
publication; no numeric-handle lookup/removal or record adoption occurs. Any
returned object is removed before diagnostic return through the existing owned
cleanup path. Actual handle remapping would be reported separately.

The initial attempted pair returned nil for both reported modes, but a subsequent
code review found its **dynamic control invalid**: the CLI/metadata branch existed
without the intended fixture-selector arm, so it published the original AudioSink
fixture instead. The assigned own-handle mode did select its intended fixture.
The raw attempted pair is preserved and explicitly annotated as invalid in
[its history](macos-sdp-own-handle-results.json); it cannot support a conclusion
that the typed handle breaks generic dynamic publication.

The selector is now an explicit fixed-case mapping with no silent fallback, and
publication reports the selected fixture from that same mapping. Before the fix,
`--validate-own-handle` returned exit 4 with null parsed values; after it, the same
non-publishing command returned exit 0. Both the typed form and Apple's documented
plain-NSNumber unsigned32 shortcut parsed as type 1, size 4, and value 1781334017,
with their exact dynamic fixture names reported. This fixes a diagnostic dispatch
bug, not the underlying Bluetooth receiver behavior. The local validator fixed the dispatch defect without publication; corrected
publication comparisons are recorded separately below.

## Corrected handle-encoding comparisons

The fixture-dispatch mistake was corrected before the following runs. Every
publication now reports its selected immutable fixture and passes an in-memory
semantic guard: actual parsed L2CAP PSM must match the scenario, and an own-handle
case must contain type 1, size 4, and value `0x6A2D0001`. A mismatch exits 4 with
`mutation_attempted=false` before the publisher. Unknown modes have no fallback.
This guard addresses the actual diagnostic failure rather than adding vendor
wrapper unit tests.

The corrected typed pair and plain-NSNumber pair ran on one unchanged binary,
outside the command sandbox, with CoreBluetooth class authorization 3:

| Handle representation | Requested PSM | Owned result | Exit |
| --- | --- | --- | --- |
| Explicit unsigned four-byte element | `0x1001` | Record returned; actual PSM 4097; handle getter 0, actual handle 1330218448; removal 0 | 0 |
| Explicit unsigned four-byte element | `0x19` | Nil; no owned handle getter or removal | 5 |
| Plain NSNumber unsigned32 shortcut | `0x1001` | Record returned; actual PSM 4097; handle getter 0, actual handle 1330218944; removal 0 | 0 |
| Plain NSNumber unsigned32 shortcut | `0x19` | Nil; no owned handle getter or removal | 5 |

Every actual fixture tag and prepublication guard matched its intended scenario.
Both dynamic results returned handles different from the requested 1781334017;
the observed assignment/remapping does not by itself identify collision or
other policy. Cleanup used only each returned object, never those numbers.
The unchanged before/after SHA256 was
`a2064179ad72f30eea502b3d143109597d17ebe273fae295baa2342db255753e`.
[Exact context/parser checks/outputs/timestamps](macos-sdp-handle-encoding-results.json).

Both legal handle representations therefore work for the dynamic control, and
neither repaired the assigned-PSM path in this experiment. The earlier invalid
dynamic control is excluded from this conclusion. Local parser red/green evidence
for the dispatch fix is retained in
[fixture validation history](macos-sdp-fixture-validation.json). This does not
resolve the underlying native publication/materialization mechanism or pass the
receiver gate; tracing the actual public call's error path is a separate next step.

## Unused trace hook

A bounded normally-launched stdin/PID trace gate was prepared and built for a
possible public-wrapper debugger observation. It was not executed, produced no
native-publication evidence, and was removed before commit when work moved to
an outbound transport proof. No private call, debugger permission change, or
additional Bluetooth mutation was performed through that hook.

## Ownership and interpretation limits

These are fixed public-API cases with bounded subprocess lifetimes and default
transient records. No system service is displaced, radio or class changed, device
unpaired, or channel delegate installed. Nil does not justify removing a requested
or logged numeric handle. The lower-level add/log and public record return are
separate stages; these outcomes alone do not identify the exact native failure
branch or prove a categorical server rejection. Native phone audio and Bluetooth
coexistence remain untested, and the full hardware gate remains unpassed.

## Primary references

- [Apple public publisher](https://developer.apple.com/documentation/iobluetooth/iobluetoothsdpservicerecord/publishedservicerecord(with:)): requested handles may be remapped if occupied; transient lifetime is the default.
- [Apple SDP element encoding](https://developer.apple.com/documentation/iobluetooth/iobluetoothsdpdataelement): explicit typed numeric elements and the plain-number unsigned-32 shortcut.
- [SIG SDP specification](https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Core_v7.0/out/en/host/service-discovery-protocol--sdp--specification.html): handle typing and reserved range.
- [Apple RunLoop](https://developer.apple.com/documentation/foundation/runloop) and [run-loop management](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/RunLoopManagement/RunLoopManagement.html): thread ownership and input-source behavior.
- [Chromium synchronous publication/check](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/device/bluetooth/bluetooth_socket_mac.mm) and [Qt publication/check](https://github.com/qt/qtconnectivity/blob/dev/src/bluetooth/qbluetoothserviceinfo_macos.mm): no fixed warm-up is established by these examples.

## Validation

The retained diagnostics pass formatting, locked workspace build, and Clippy with
warnings denied. The public local CLI parser exposed and verified the fixture
selection repair before the corrected hardware comparisons. No new vendor-wrapper
unit seam or full session implementation was introduced. The assigned-PSM
receiver path remains unresolved; an independently owned outbound signaling
experiment is a separate transport hypothesis, not a substitute for A2DP
advertising or a passed hardware gate.
