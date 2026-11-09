use crate::Mode;
use ::windows::{
    Devices::Enumeration::{DeviceClass, DeviceInformation},
    Foundation::Metadata::ApiInformation,
    Media::Audio::{
        AudioPlaybackConnection, AudioPlaybackConnectionOpenResultStatus,
        AudioPlaybackConnectionState,
    },
    System::Profile::AnalyticsInfo,
    Win32::System::{
        Com::{APTTYPE, APTTYPEQUALIFIER, CoGetApartmentType},
        WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
    },
    core::{Error, HSTRING, Interface, h},
};
use serde_json::{Value, json};
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows_future::{AsyncStatus, IAsyncInfo, IAsyncOperation};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
type ProbeResult<T> = Result<T, Value>;

// Keep the initializing thread's WinRT apartment alive until receiver/async objects drop.
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { RoUninitialize() };
    }
}

struct Receiver {
    connection: AudioPlaybackConnection,
    closed: bool,
}
impl Receiver {
    fn close(&mut self) -> ProbeResult<()> {
        api("close_receiver", self.connection.Close())?;
        self.closed = true;
        emit(json!({"event": "receiver_closed", "cleanup": "close_succeeded"}));
        Ok(())
    }
}
impl Drop for Receiver {
    fn drop(&mut self) {
        // Close only the connection this process created, including on early returns.
        if !self.closed
            && let Err(error) = self.close()
        {
            emit(error);
        }
    }
}

fn emit(mut event: Value) {
    event["hardware_gate"] = json!("untested");
    event["full_feasibility_gate"] = json!("not_passed");
    // A stopped output pipeline must not prevent receiver cleanup.
    let _ = writeln!(std::io::stdout().lock(), "{event}");
}

fn api_error(stage: &'static str, error: Error) -> Value {
    // OS error text can contain device identifiers. Persist only the HRESULT.
    json!({"status": "api_error", "stage": stage, "hresult": format!("0x{:08X}", error.code().0 as u32)})
}

fn api<T>(stage: &'static str, result: ::windows::core::Result<T>) -> ProbeResult<T> {
    result.map_err(|error| api_error(stage, error))
}

fn wait(info: &IAsyncInfo, stage: &'static str, stop: &AtomicBool) -> ProbeResult<()> {
    let deadline = Instant::now() + OPERATION_TIMEOUT;
    while api(stage, info.Status())? == AsyncStatus::Started {
        if stop.load(Ordering::SeqCst) || Instant::now() >= deadline {
            let cancelled = info.Cancel().is_ok();
            return Err(json!({
                "status": if stop.load(Ordering::SeqCst) { "cancelled" } else { "operation_timeout" },
                "stage": stage, "cancel_requested": cancelled
            }));
        }
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

fn results<T: ::windows::core::RuntimeType>(
    operation: IAsyncOperation<T>,
    stage: &'static str,
    stop: &AtomicBool,
) -> ProbeResult<T> {
    wait(&api(stage, operation.cast())?, stage, stop)?;
    api(stage, operation.GetResults())
}

pub(crate) fn run(mode: Mode) -> u8 {
    match run_inner(mode) {
        Ok(()) => 0,
        Err(mut error) => {
            error["mode"] = json!(mode.name());
            emit(error);
            2
        }
    }
}

fn run_inner(mode: Mode) -> ProbeResult<()> {
    api("initialize_mta", unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)
    })?;
    let _apartment = Apartment;
    let mut apartment_type = APTTYPE::default();
    let mut qualifier = APTTYPEQUALIFIER::default();
    api("apartment_type", unsafe {
        CoGetApartmentType(&mut apartment_type, &mut qualifier)
    })?;
    let version = api("os_version_info", AnalyticsInfo::VersionInfo())?;
    let encoded = api("os_version", version.DeviceFamilyVersion())?
        .to_string()
        .parse::<u64>()
        .map_err(|_| json!({"status": "invalid_os_version", "stage": "os_version"}))?;
    let major = (encoded >> 48) & 0xffff;
    let minor = (encoded >> 32) & 0xffff;
    let build = (encoded >> 16) & 0xffff;
    let revision = encoded & 0xffff;
    let available = api(
        "receiver_type_present",
        ApiInformation::IsTypePresent(h!("Windows.Media.Audio.AudioPlaybackConnection")),
    )?;
    emit(json!({
        "event": "preflight", "mode": mode.name(), "status": "inspection_complete",
        "os": {"major": major, "minor": minor, "build": build, "revision": revision},
        "apartment_type": apartment_type.0, "apartment_qualifier": qualifier.0,
        "receiver_api_present": available,
        "minimum_os": "Windows 11 build 22000",
        "radio_enumerated": false, "receiver_opened": false,
        "audio_capture": "not_tested", "silent_forwarding": "not_tested", "coexistence": "not_tested"
    }));
    if !available || major < 10 || build < 22000 {
        return Err(json!({"status": "unsupported_os_or_api"}));
    }
    if matches!(mode, Mode::Preflight) {
        return Ok(());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_signal = Arc::clone(&stop);
    ctrlc::set_handler(move || stop_signal.store(true, Ordering::SeqCst))
        .map_err(|_| json!({"status": "signal_handler_unavailable"}))?;
    match mode {
        Mode::ListSources => {
            let sources = sources(&stop)?;
            print_sources(&sources)?;
            emit(
                json!({"event": "source_inventory", "paired_source_count": sources.len(), "receiver_opened": false}),
            );
        }
        Mode::ListCaptureEndpoints => list_capture_endpoints(&stop)?,
        Mode::Receive { .. } => {
            let (index, duration_seconds) = mode.selection().expect("receive mode");
            let sources = sources(&stop)?;
            let source = sources.get(index as usize).ok_or_else(|| {
                json!({
                    "status": "source_index_out_of_range", "source_count": sources.len()
                })
            })?;
            eprintln!(
                "Selected local source [{index}]: {:?}",
                api("source_name", source.Name())?.to_string()
            );
            eprintln!(
                "RECEIVER BASELINE: Windows plays incoming phone audio locally. This does not provide silent forwarding or Discord playback. Stop with Ctrl+C."
            );
            if stop.load(Ordering::SeqCst) {
                return Err(json!({"status": "cancelled", "stage": "before_create"}));
            }
            let id = api("source_id", source.Id())?;
            let mut receiver = Receiver {
                connection: api(
                    "create_receiver",
                    AudioPlaybackConnection::TryCreateFromId(&id),
                )?,
                closed: false,
            };
            if stop.load(Ordering::SeqCst) {
                return Err(json!({"status": "cancelled", "stage": "before_start"}));
            }
            let start = api("start_receiver", receiver.connection.StartAsync())?;
            wait(
                &api("start_receiver", start.cast())?,
                "start_receiver",
                &stop,
            )?;
            api("start_receiver", start.GetResults())?;
            if stop.load(Ordering::SeqCst) {
                return Err(json!({"status": "cancelled", "stage": "before_open"}));
            }
            let opened = results(
                api("open_receiver", receiver.connection.OpenAsync())?,
                "open_receiver",
                &stop,
            )?;
            let status = api("open_status", opened.Status())?;
            let extended_error = api("open_extended_error", opened.ExtendedError())?;
            emit(json!({
                "event": "receiver_open_result", "source_index": index,
                "open_status": status.0,
                "extended_hresult": format!("0x{:08X}", extended_error.0 as u32),
                "baseline_local_playback": true,
                "frames_observed": "not_measured", "discord_forwarding": "not_implemented"
            }));
            if status != AudioPlaybackConnectionOpenResultStatus::Success {
                return Err(json!({"status": "receiver_open_failed", "open_status": status.0}));
            }
            let deadline = Instant::now() + Duration::from_secs(duration_seconds);
            let mut last_state = None;
            while !stop.load(Ordering::SeqCst) && Instant::now() < deadline {
                let state = api("receiver_state", receiver.connection.State())?;
                if last_state != Some(state) {
                    emit(json!({"event": "receiver_state", "state": state.0}));
                    last_state = Some(state);
                }
                if state == AudioPlaybackConnectionState::Closed {
                    return Err(json!({"status": "receiver_closed_early"}));
                }
                thread::sleep(Duration::from_millis(100));
            }
            emit(
                json!({"event": "baseline_finished", "reason": if stop.load(Ordering::SeqCst) { "ctrl_c" } else { "duration_elapsed" }, "requested_duration_seconds": duration_seconds}),
            );
            receiver.close()?;
        }
        Mode::Preflight => unreachable!(),
    }
    Ok(())
}

fn sources(stop: &AtomicBool) -> ProbeResult<Vec<DeviceInformation>> {
    let selector = api(
        "source_selector",
        AudioPlaybackConnection::GetDeviceSelector(),
    )?;
    let found = results(
        api(
            "enumerate_paired_sources",
            DeviceInformation::FindAllAsyncAqsFilter(&selector),
        )?,
        "enumerate_paired_sources",
        stop,
    )?;
    let mut paired = Vec::<(HSTRING, DeviceInformation)>::new();
    let mut skipped = 0;
    for index in 0..api("source_count", found.Size())? {
        let source = api("source_item", found.GetAt(index))?;
        match source.Pairing().and_then(|pairing| pairing.IsPaired()) {
            Ok(true) => paired.push((api("source_id", source.Id())?, source)),
            Ok(false) => skipped += 1,
            Err(error) => {
                skipped += 1;
                emit(api_error("verify_source_pairing", error));
            }
        }
    }
    emit(
        json!({"event": "source_pairing_filter", "paired_source_count": paired.len(), "unpaired_or_unverified_skipped": skipped}),
    );
    // Stable ordering within an unchanged paired set; no identifiers are persisted.
    paired.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(paired.into_iter().map(|(_, source)| source).collect())
}

fn print_sources(sources: &[DeviceInformation]) -> ProbeResult<()> {
    eprintln!("Paired receiver-capable sources (local console only; rerun after pairing changes):");
    for (index, source) in sources.iter().enumerate() {
        eprintln!(
            "[{index}] {:?}",
            api("source_name", source.Name())?.to_string()
        );
    }
    Ok(())
}

fn list_capture_endpoints(stop: &AtomicBool) -> ProbeResult<()> {
    let found = results(
        api(
            "enumerate_capture_endpoints",
            DeviceInformation::FindAllAsyncDeviceClass(DeviceClass::AudioCapture),
        )?,
        "enumerate_capture_endpoints",
        stop,
    )?;
    let count = api("capture_endpoint_count", found.Size())?;
    let mut candidates = 0;
    eprintln!("Audio-capture endpoints (names local only; listing proves no audio isolation):");
    for index in 0..count {
        let name = api(
            "capture_endpoint_name",
            api("capture_endpoint", found.GetAt(index))?.Name(),
        )?;
        let candidate = name.to_string().to_ascii_lowercase().contains("a2dp");
        candidates += u32::from(candidate);
        eprintln!(
            "[{index}] {:?}{}",
            name.to_string(),
            if candidate {
                " (A2DP name candidate only)"
            } else {
                ""
            }
        );
    }
    emit(
        json!({"event": "capture_inventory", "endpoint_count": count, "a2dp_name_candidates": candidates, "pcm_captured": false}),
    );
    Ok(())
}
