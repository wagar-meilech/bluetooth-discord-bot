# Bluetooth Discord bridge

A Rust project for sending phone Bluetooth audio or selected host-application
audio into a Discord server voice channel, using a native macOS/Windows adapter
and a local Dockerized bot core.

The current stage is **native receiver feasibility**. Product implementation
follows the recorded two-platform hardware gate.

- [Approved spec](docs/spec.md) and [task graph](docs/WORK.md).
- [Mac probe, observed results, and commands](docs/feasibility/macos-receiver.md).
- [Controlled Mac diagnosis](docs/feasibility/macos-sdp-diagnosis.md): a same-binary
  generic service publishes at dynamic PSM `0x1001`, while required AVDTP `0x19`
  returns no owned record through the tested public API.
- [Windows probe and PowerShell instructions](docs/feasibility/windows-receiver-probe.md).
- [Download the verified Windows probe](https://github.com/wagar-meilech/bluetooth-discord-bot/actions/runs/42543599395/artifacts/13642297887)
  and [record hardware results](docs/feasibility/windows-hardware-results.md).
- [Draft PR #10](https://github.com/wagar-meilech/bluetooth-discord-bot/pull/10)
  and [Windows artifact build](https://github.com/wagar-meilech/bluetooth-discord-bot/actions/runs/42543599395).

The workspace pins Rust 1.94.0. A local build is:

```sh
cargo build --locked --workspace
```

Both probes default to API inspection. Receiver/publication experiments require
explicit flags documented in their guides. The Windows receive baseline plays
locally; silent capture, real Discord delivery, and Bluetooth coexistence still
require recorded hardware evidence.
