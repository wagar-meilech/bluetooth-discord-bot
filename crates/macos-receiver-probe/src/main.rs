mod avdtp;

use serde_json::json;
use std::process::ExitCode;

#[cfg(target_os = "macos")]
mod macos;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let [flag, packet] = args.as_slice()
        && flag == "--decode-discover"
    {
        let (report, code) = avdtp::decode_hex(packet.to_str().unwrap_or(""));
        println!("{report}");
        return ExitCode::from(code);
    }
    let mode = match args.as_slice() {
        [] => "preflight",
        [arg] if arg == "--preflight" => "preflight",
        [arg] if arg == "--preflight-warmup" => "preflight-warmup",
        [arg] if arg == "--probe-sdp" => "probe-sdp",
        [arg] if arg == "--validate-sdp" => "validate-sdp",
        [arg] if arg == "--validate-own-handle" => "validate-own-handle",
        [arg] if arg == "--diagnose-context" => "diagnose-context",
        [arg] if arg == "--list-paired" => "list-paired",
        [arg] if arg == "--pair-device-panel" => "pair-device-panel",
        [arg] if arg == "--connect-phone" => "connect-phone",
        [arg] if arg == "--discover-phone" => "discover-phone",
        [arg] if arg == "--discover-phone-selected-object" => "discover-phone-selected-object",
        [arg] if arg == "--probe-sdp-bare-keys" => "probe-sdp-bare-keys",
        [arg] if arg == "--probe-generic-l2cap" => "probe-generic-l2cap",
        [arg] if arg == "--probe-generic-avdtp-psm" => "probe-generic-avdtp-psm",
        [arg] if arg == "--probe-generic-l2cap-warmup" => "probe-generic-l2cap-warmup",
        [arg] if arg == "--probe-generic-avdtp-psm-warmup" => "probe-generic-avdtp-psm-warmup",
        [arg] if arg == "--probe-generic-avdtp-own-handle" => "probe-generic-avdtp-own-handle",
        [arg] if arg == "--probe-generic-l2cap-own-handle" => "probe-generic-l2cap-own-handle",
        [arg] if arg == "--probe-generic-l2cap-own-handle-number" => {
            "probe-generic-l2cap-own-handle-number"
        }
        [arg] if arg == "--probe-generic-avdtp-own-handle-number" => {
            "probe-generic-avdtp-own-handle-number"
        }
        [arg] if arg == "--help" => {
            println!(
                "{}",
                json!({
                    "usage": "macos-receiver-probe [--preflight | --preflight-warmup | --validate-sdp | --validate-own-handle | --diagnose-context | --list-paired | --pair-device-panel | --connect-phone | --discover-phone | --discover-phone-selected-object | --decode-discover HEX | --probe-sdp | --probe-sdp-bare-keys | --probe-generic-l2cap | --probe-generic-avdtp-psm | --probe-generic-l2cap-warmup | --probe-generic-avdtp-psm-warmup | --probe-generic-avdtp-own-handle | --probe-generic-l2cap-own-handle | --probe-generic-l2cap-own-handle-number | --probe-generic-avdtp-own-handle-number]",
                    "default": "preflight: controller and API inspection only",
                    "preflight-warmup": "read-only preflight after one second of public main-run-loop processing",
                    "validate-sdp": "parse bundled SDP in memory without publishing or contacting a device",
                    "validate-own-handle": "parse typed/plain unsigned32 handle values locally without publication",
                    "probe-sdp": "publish one transient AudioSink SDP record, verify PSM, immediately remove it",
                    "probe-sdp-bare-keys": "same transient A2DP experiment, changing only attribute keys to bare decimal",
                    "probe-generic-l2cap": "transient custom-UUID L2CAP control on dynamic PSM0x1001; not an audio service",
                    "probe-generic-avdtp-psm": "same generic control with only requested PSM changed to assigned0x19; not A2DP",
                    "probe-generic-l2cap-warmup": "same dynamic-PSM control after one second main-run-loop warm-up",
                    "probe-generic-avdtp-psm-warmup": "same assigned-PSM control after one second main-run-loop warm-up",
                    "probe-generic-avdtp-own-handle": "same assigned-PSM control with one fresh nonreserved requested uint32 service handle",
                    "probe-generic-l2cap-own-handle": "matching dynamic-PSM control with the same fresh requested handle",
                    "probe-generic-l2cap-own-handle-number": "dynamic-PSM own-handle control using plain unsigned32 NSNumber shortcut",
                    "probe-generic-avdtp-own-handle-number": "assigned-PSM own-handle control using the same NSNumber shortcut",
                    "diagnose-context": "read own bundle identity and CoreBluetooth authorization class getter only",
                    "list-paired": "read cached paired-device state; sanitized stdout and local names on stderr",
                    "pair-device-panel": "show explicit OS-managed pairing/discovery UI; operator confirms PIN",
                    "connect-phone": "explicit OS picker and bounded public async baseband connection only; no authentication requirement or ACL close",
                    "discover-phone": "explicit OS picker, fresh paired/disconnected phone, one owned outbound AVDTP DISCOVER only",
                    "discover-phone-selected-object": "same guarded DISCOVER, retaining original OS-picker object after fresh identity validation",
                    "decode-discover": "offline known-spec packet replay; no Bluetooth calls",
                    "receiver_implemented": false
                })
            );
            return ExitCode::SUCCESS;
        }
        _ => {
            println!(
                "{}",
                json!({"status": "invalid_arguments", "usage": "macos-receiver-probe [--preflight | --preflight-warmup | --validate-sdp | --validate-own-handle | --diagnose-context | --list-paired | --pair-device-panel | --connect-phone | --discover-phone | --discover-phone-selected-object | --decode-discover HEX | --probe-sdp | --probe-sdp-bare-keys | --probe-generic-l2cap | --probe-generic-avdtp-psm | --probe-generic-l2cap-warmup | --probe-generic-avdtp-psm-warmup | --probe-generic-avdtp-own-handle | --probe-generic-l2cap-own-handle | --probe-generic-l2cap-own-handle-number | --probe-generic-avdtp-own-handle-number]"})
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
