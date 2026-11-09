use serde_json::json;
use std::process::ExitCode;

#[cfg(target_os = "macos")]
mod macos;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = match args.as_slice() {
        [] => "preflight",
        [arg] if arg == "--preflight" => "preflight",
        [arg] if arg == "--probe-sdp" => "probe-sdp",
        [arg] if arg == "--validate-sdp" => "validate-sdp",
        [arg] if arg == "--diagnose-context" => "diagnose-context",
        [arg] if arg == "--probe-sdp-bare-keys" => "probe-sdp-bare-keys",
        [arg] if arg == "--probe-generic-l2cap" => "probe-generic-l2cap",
        [arg] if arg == "--probe-generic-avdtp-psm" => "probe-generic-avdtp-psm",
        [arg] if arg == "--help" => {
            println!(
                "{}",
                json!({
                    "usage": "macos-receiver-probe [--preflight | --validate-sdp | --diagnose-context | --probe-sdp | --probe-sdp-bare-keys | --probe-generic-l2cap | --probe-generic-avdtp-psm]",
                    "default": "preflight: controller and API inspection only",
                    "validate-sdp": "parse bundled SDP in memory without publishing or contacting a device",
                    "probe-sdp": "publish one transient AudioSink SDP record, verify PSM, immediately remove it",
                    "probe-sdp-bare-keys": "same transient A2DP experiment, changing only attribute keys to bare decimal",
                    "probe-generic-l2cap": "transient custom-UUID L2CAP control on dynamic PSM0x1001; not an audio service",
                    "probe-generic-avdtp-psm": "same generic control with only requested PSM changed to assigned0x19; not A2DP",
                    "diagnose-context": "read own bundle identity and CoreBluetooth authorization class getter only",
                    "receiver_implemented": false
                })
            );
            return ExitCode::SUCCESS;
        }
        _ => {
            println!(
                "{}",
                json!({"status": "invalid_arguments", "usage": "macos-receiver-probe [--preflight | --validate-sdp | --diagnose-context | --probe-sdp | --probe-sdp-bare-keys | --probe-generic-l2cap | --probe-generic-avdtp-psm]"})
            );
            return ExitCode::from(2);
        }
    };

    #[cfg(target_os = "macos")]
    let (report, code) = objc2::rc::autoreleasepool(|_| macos::run(mode));
    #[cfg(not(target_os = "macos"))]
    let (report, code) = (
        json!({"status": "unsupported_platform", "mode": mode, "live_audio": "not_tested", "full_feasibility_gate": "not_passed"}),
        2,
    );

    println!("{report}");
    ExitCode::from(code)
}
