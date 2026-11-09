use serde_json::json;
use std::process::ExitCode;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone, Copy)]
enum Mode {
    Preflight,
    ListSources,
    ListCaptureEndpoints,
    Receive { index: u32, duration_seconds: u64 },
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::ListSources => "list_sources",
            Self::ListCaptureEndpoints => "list_capture_endpoints",
            Self::Receive { .. } => "receive_baseline",
        }
    }

    fn selection(self) -> Option<(u32, u64)> {
        match self {
            Self::Receive {
                index,
                duration_seconds,
            } => Some((index, duration_seconds)),
            _ => None,
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mode = match args.as_slice() {
        [] => Mode::Preflight,
        [arg] if arg == "--preflight" => Mode::Preflight,
        [arg] if arg == "--list-sources" => Mode::ListSources,
        [arg] if arg == "--list-capture-endpoints" => Mode::ListCaptureEndpoints,
        [flag, index, duration_flag, seconds]
            if flag == "--receive-index" && duration_flag == "--duration-seconds" =>
        {
            match (index.parse::<u32>(), seconds.parse::<u64>()) {
                (Ok(index), Ok(duration_seconds @ 1..=1800)) => Mode::Receive {
                    index,
                    duration_seconds,
                },
                _ => return invalid_arguments(),
            }
        }
        [arg] if arg == "--help" => {
            println!(
                "{}",
                json!({
                    "usage": "windows-receiver-probe [--preflight | --list-sources | --list-capture-endpoints | --receive-index N --duration-seconds 1..1800]",
                    "default": "preflight: inspect OS, apartment, and API presence only; no enumeration or reception",
                    "list_sources": "enumerate the paired AudioPlaybackConnection selector; names only on local stderr",
                    "list_capture_endpoints": "read-only audio-capture endpoint inventory; no PCM capture",
                    "receive_baseline": "bounded activation of one selected paired source; Windows plays locally; no Discord forwarding",
                    "reports": "stdout is sanitized JSON Lines; local stderr may contain friendly device names",
                    "full_feasibility_gate": "not_passed"
                })
            );
            return ExitCode::SUCCESS;
        }
        _ => return invalid_arguments(),
    };

    #[cfg(target_os = "windows")]
    let code = windows::run(mode);
    #[cfg(not(target_os = "windows"))]
    let code = {
        println!(
            "{}",
            json!({
                "status": "unsupported_platform", "mode": mode.name(),
                "requested_selection": mode.selection(),
                "hardware_gate": "untested", "full_feasibility_gate": "not_passed"
            })
        );
        2
    };
    ExitCode::from(code)
}

fn invalid_arguments() -> ExitCode {
    println!(
        "{}",
        json!({
            "status": "invalid_arguments",
            "usage": "windows-receiver-probe --help",
            "full_feasibility_gate": "not_passed"
        })
    );
    ExitCode::from(2)
}
