use crate::avdtp;
use objc2::{
    AnyThread, ClassType, DefinedClass, MainThreadMarker, define_class, msg_send, rc::Retained,
    runtime::AnyObject, sel,
};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_core_bluetooth::CBManager;
use objc2_foundation::{NSDate, NSObject, NSObjectProtocol, NSRunLoop, NSString};
use objc2_io_bluetooth::{
    IOBluetoothDevice, IOBluetoothHostController, IOBluetoothL2CAPChannel,
    IOBluetoothL2CAPChannelDelegate,
};
use objc2_io_bluetooth_ui::{
    IOBluetoothDeviceSelectorController, kIOBluetoothServiceBrowserControllerOptionsNone,
    kIOBluetoothUISuccess,
};
use serde_json::{Value, json};
use std::{
    ffi::c_void,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

const PSM: u16 = 0x19;
const MAX_EVENTS: usize = 8;

#[derive(Debug)]
enum Event {
    Open(usize, i32),
    Data(usize, Vec<u8>),
    Closed(usize),
    Write(usize, i32),
    Oversized(usize),
}
struct DelegateState {
    sender: SyncSender<Event>,
    overflow: Arc<AtomicBool>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = AnyThread]
    #[ivars = DelegateState]
    struct DiscoverDelegate;
    unsafe impl NSObjectProtocol for DiscoverDelegate {}
    unsafe impl IOBluetoothL2CAPChannelDelegate for DiscoverDelegate {
        #[unsafe(method(l2capChannelOpenComplete:status:))]
        fn opened(&self, channel: Option<&IOBluetoothL2CAPChannel>, status: i32) {
            if let Some(channel) = channel {
                self.push(Event::Open(channel_id(channel), status));
            }
        }
        #[unsafe(method(l2capChannelData:data:length:))]
        fn received(
            &self,
            channel: Option<&IOBluetoothL2CAPChannel>,
            data: *mut c_void,
            length: usize,
        ) {
            let Some(channel) = channel else { return };
            if length > avdtp::MAX_RESPONSE || (length != 0 && data.is_null()) {
                self.push(Event::Oversized(channel_id(channel)));
                return;
            }
            // SAFETY: The OS callback lends this valid range until return. Copy
            // bounded bytes only; no framework objects cross threads.
            let bytes = if length == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length) }.to_vec()
            };
            self.push(Event::Data(channel_id(channel), bytes));
        }
        #[unsafe(method(l2capChannelClosed:))]
        fn closed(&self, channel: Option<&IOBluetoothL2CAPChannel>) {
            if let Some(channel) = channel {
                self.push(Event::Closed(channel_id(channel)));
            }
        }
        #[unsafe(method(l2capChannelWriteComplete:refcon:status:))]
        fn written(
            &self,
            channel: Option<&IOBluetoothL2CAPChannel>,
            _refcon: *mut c_void,
            status: i32,
        ) {
            if let Some(channel) = channel {
                self.push(Event::Write(channel_id(channel), status));
            }
        }
    }
);
impl DiscoverDelegate {
    fn new(sender: SyncSender<Event>, overflow: Arc<AtomicBool>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(DelegateState { sender, overflow });
        // SAFETY: NSObject init on a freshly allocated object.
        unsafe { msg_send![super(this), init] }
    }
    fn push(&self, event: Event) {
        if self.ivars().sender.try_send(event).is_err() {
            self.ivars().overflow.store(true, Ordering::Relaxed);
        }
    }
}
fn channel_id(channel: &IOBluetoothL2CAPChannel) -> usize {
    std::ptr::from_ref(channel).addr()
}

const NATIVE_DEADLINE: Duration = Duration::from_secs(25);
struct NativeWatchdog {
    _cancel: mpsc::Sender<()>,
}
impl NativeWatchdog {
    fn arm() -> std::io::Result<Self> {
        let (cancel, completion) = mpsc::channel::<()>();
        thread::Builder::new()
            .name("bt-probe-deadline".into())
            .spawn(move || {
                if completion.recv_timeout(NATIVE_DEADLINE) == Err(mpsc::RecvTimeoutError::Timeout)
                {
                    // SAFETY: Only this helper process exits. No ObjC access,
                    // allocation, telemetry, I/O, lock or atexit handler can
                    // delay termination after the timer expires. Exit6 may
                    // have no final JSON; native cleanup is then unconfirmed.
                    unsafe { libc::_exit(6) };
                }
            })?;
        Ok(Self { _cancel: cancel })
    }
}

struct OwnedChannel {
    channel: Retained<IOBluetoothL2CAPChannel>,
    cleanup_attempted: bool,
}
impl OwnedChannel {
    fn close(&mut self) -> (i32, i32) {
        // SAFETY: Only the outgoing Async result with this supplied delegate,
        // checked PSM, and no preexisting ACL is adopted. Never close baseband.
        let detached = unsafe { self.channel.setDelegate(None) };
        let closed = unsafe { self.channel.closeChannel() };
        self.cleanup_attempted = true;
        (detached, closed)
    }
}
impl Drop for OwnedChannel {
    fn drop(&mut self) {
        if !self.cleanup_attempted {
            let _ = self.close();
        }
        // closeChannel is asynchronous. Keep this explicitly created channel
        // alive with the leaked delegate/request until one-shot process exit,
        // even if a queued callback follows detachment or pending-open close.
        let _ = Retained::into_raw(self.channel.clone());
    }
}

fn valid_address(address: &NSString) -> bool {
    let value = address.to_string();
    // Public address strings normally use six hex pairs separated by '-'.
    // Accept the equivalent ':' or compact representation for diagnostics.
    let compact: String = value.chars().filter(|c| *c != '-' && *c != ':').collect();
    compact.len() == 12
        && compact.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !compact.bytes().all(|byte| byte == b'0')
        && !compact.bytes().all(|byte| byte.eq_ignore_ascii_case(&b'f'))
}

struct FreshMatch {
    device: Retained<IOBluetoothDevice>,
    equality_match: bool,
    address: Option<Retained<NSString>>,
    address_matches_selected: bool,
    paired: bool,
    connected: bool,
}

fn selected_fresh_paired(
    report: &mut Value,
    main: MainThreadMarker,
    use_original_picker_object: bool,
) -> Option<Retained<IOBluetoothDevice>> {
    let app = NSApplication::sharedApplication(main);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    // SAFETY: Explicit operator UI, no pairing/PIN delegate or automatic choice.
    let selector = unsafe { IOBluetoothDeviceSelectorController::deviceSelector(main) }?;
    unsafe {
        selector.clearAllowedUUIDs();
        selector.setOptions(kIOBluetoothServiceBrowserControllerOptionsNone);
        selector.setTitle(Some(&NSString::from_str("Choose paired iPhone")));
        selector.setDescriptionText(Some(&NSString::from_str("Choose your paired, disconnected iPhone. This test only requests its Bluetooth audio endpoint list.")));
        selector.setPrompt(Some(&NSString::from_str("Run DISCOVER test")));
    }
    let class = selector.class();
    let response: isize = if class.verify_sel::<(), i32>(sel!(runModal)).is_ok() {
        let response: i32 = unsafe { msg_send![&*selector, runModal] };
        response as isize
    } else if class.verify_sel::<(), isize>(sel!(runModal)).is_ok() {
        unsafe { msg_send![&*selector, runModal] }
    } else {
        report["status"] = json!("selector_return_abi_unsupported");
        return None;
    };
    report["selector_response"] = json!(response);
    if response != kIOBluetoothUISuccess as isize {
        report["status"] = json!("device_selection_not_completed");
        return None;
    }
    let selected = unsafe { selector.getResults() }?;
    if selected.count() != 1 {
        report["status"] = json!("one_explicit_device_required");
        return None;
    }
    let selected = selected
        .objectAtIndex(0)
        .downcast::<IOBluetoothDevice>()
        .ok()?;
    report["selected_object_is_paired"] = json!(unsafe { selected.isPaired() });
    let Some(fresh) = (unsafe { IOBluetoothDevice::pairedDevices() }) else {
        report["status"] = json!("fresh_paired_inventory_unavailable");
        return None;
    };
    // Addresses remain private process-memory values, used only to correlate
    // this operator selection with a fresh retained paired-device object.
    #[allow(deprecated)]
    let selected_address = unsafe { selected.getAddressString() };
    let selected_address_nonempty = selected_address.as_ref().is_some_and(|a| !a.is_empty());
    let selected_address_valid = selected_address.as_ref().is_some_and(|a| valid_address(a));
    let mut exact_object_matches = 0;
    let mut equality_matches = 0;
    let mut address_matches = 0;
    let mut nonempty_addresses = 0;
    let mut valid_addresses = 0;
    let mut unexpected_entries = 0;
    let mut matches = Vec::new();
    for index in 0..fresh.count() {
        let Ok(device) = fresh.objectAtIndex(index).downcast::<IOBluetoothDevice>() else {
            unexpected_entries += 1;
            continue;
        };
        exact_object_matches += usize::from(std::ptr::eq(&*selected, &*device));
        let equal = selected.isEqual(Some(&device));
        #[allow(deprecated)]
        let address = unsafe { device.getAddressString() };
        let nonempty = address.as_ref().is_some_and(|a| !a.is_empty());
        let valid = address.as_ref().is_some_and(|a| valid_address(a));
        nonempty_addresses += usize::from(nonempty);
        valid_addresses += usize::from(valid);
        // Compare NSString contents explicitly; no pointer/string identifiers
        // are emitted. Reject absent/malformed addresses as identity evidence.
        let address_equal = selected_address_valid
            && valid
            && selected_address
                .as_ref()
                .zip(address.as_ref())
                .is_some_and(|(selected, candidate)| selected.isEqualToString(candidate));
        equality_matches += usize::from(equal);
        address_matches += usize::from(address_equal);
        if equal || address_equal {
            let paired = unsafe { device.isPaired() };
            let connected = unsafe { device.isConnected() };
            matches.push(FreshMatch {
                device,
                equality_match: equal,
                address,
                address_matches_selected: address_equal,
                paired,
                connected,
            });
        }
    }
    let all_selected_address = selected_address_valid
        && matches
            .iter()
            .all(|candidate| candidate.address_matches_selected);
    let paired_consistent = matches.first().is_some_and(|first| {
        matches
            .iter()
            .all(|candidate| candidate.paired == first.paired)
    });
    let connected_consistent = matches.first().is_some_and(|first| {
        matches
            .iter()
            .all(|candidate| candidate.connected == first.connected)
    });
    let distinct_identities = {
        let mut identities: Vec<&NSString> = Vec::new();
        let mut all_valid = true;
        for candidate in &matches {
            if let Some(address) = candidate.address.as_deref().filter(|a| valid_address(a)) {
                if !identities
                    .iter()
                    .any(|existing| existing.isEqualToString(address))
                {
                    identities.push(address);
                }
            } else {
                all_valid = false;
            }
        }
        all_valid.then_some(identities.len())
    };
    report["fresh_identity_diagnostic"] = json!({
        "inventory_count":fresh.count(), "unexpected_entry_count":unexpected_entries,
        "equality_match_count":equality_matches,"address_match_count":address_matches,
        "exact_selected_object_record_count":exact_object_matches,
        "matching_record_count":matches.len(),"distinct_identity_count":distinct_identities,
        "matching_records_all_selected_address":all_selected_address,
        "matching_paired_states_consistent":paired_consistent,
        "matching_connected_states_consistent":connected_consistent,
        "selected_address_nonempty":selected_address_nonempty,
        "selected_address_valid_non_wildcard":selected_address_valid,
        "inventory_nonempty_address_count":nonempty_addresses,
        "inventory_valid_non_wildcard_address_count":valid_addresses,
        "address_comparison":"NSString_isEqualToString_content_only_in_memory"
    });
    if matches.is_empty() {
        report["status"] = json!("selected_phone_absent_from_fresh_paired_inventory");
        return None;
    }
    // Multiple cached records are one identity only when every record matches
    // the operator-selected valid non-wildcard address and states agree. Never
    // choose between conflicting records or mutate the OS pairing store.
    if !all_selected_address || distinct_identities != Some(1) {
        report["status"] = json!("selected_phone_fresh_identity_conflict");
        return None;
    }
    if !paired_consistent || !connected_consistent {
        report["status"] = json!("selected_phone_fresh_state_conflict");
        return None;
    }
    let candidate = matches.pop().unwrap();
    report["fresh_selected_is_paired"] = json!(candidate.paired);
    report["fresh_selected_is_connected"] = json!(candidate.connected);
    if !candidate.paired || candidate.connected {
        report["status"] = json!("fresh_disconnected_paired_phone_required");
        return None;
    }
    report["selected_identity_match"] = json!(if candidate.equality_match {
        "public_equality_and_exact_local_address_group"
    } else {
        "exact_local_address_group"
    });
    let original_is_baseline_object = std::ptr::eq(&*selected, &*candidate.device);
    report["original_is_baseline_group_member_object"] = json!(original_is_baseline_object);
    if use_original_picker_object {
        // Controlled object-origin variation only: validate against the same
        // fresh identity group, then retain the original operator-picked object.
        let paired = unsafe { selected.isPaired() };
        let connected = unsafe { selected.isConnected() };
        report["original_selected_is_paired_after_validation"] = json!(paired);
        report["original_selected_is_connected_after_validation"] = json!(connected);
        if !paired || connected {
            report["status"] = json!("original_selected_disconnected_paired_phone_required");
            return None;
        }
        if original_is_baseline_object {
            report["status"] = json!("selected_object_variant_matches_baseline_object");
            report["object_origin_variant_native_open_skipped"] = json!(true);
            return None;
        }
        Some(selected)
    } else {
        Some(candidate.device)
    }
}

fn context_ready(controller: &IOBluetoothHostController, report: &mut Value, phase: &str) -> bool {
    let power = super::controller_snapshot(controller).map(|snapshot| snapshot.0);
    let authorization = if CBManager::class()
        .class_method(sel!(authorization))
        .is_some()
    {
        // Public class getter only: no manager instance or forced consent.
        Some(unsafe { CBManager::authorization_class() }.0)
    } else {
        None
    };
    report[phase] = json!({"controller_power":power,"core_bluetooth_authorization":authorization,
        "authorization_scope":"CoreBluetooth only; not proof of Classic channel permission"});
    if power != Some(1) || authorization != Some(3) {
        report["status"] = json!("powered_authorized_context_required");
        return false;
    }
    true
}

pub(super) fn run(use_original_picker_object: bool) -> (Value, u8) {
    let mut report = json!({"schema":1,"mode":"discover-phone","status":"phone_preparation_unavailable",
        "connection_started":false,"inquiry_by_probe":false,"os_selector_may_start_inquiry":true,
        "automatic_pairing_requested":false,"open_api":"openL2CAPChannelAsync:withPSM:delegate:",
        "existing_acl_refused":true,"atomic_exclusive_channel_reservation":false,
        "ownership_basis":"disconnected_snapshot_public_new_channel_contract_expected_delegate_outgoing_psm",
        "media_configuration_sent":false,
        "live_audio":"not_tested","full_feasibility_gate":"not_passed"});
    report["mode"] = json!(if use_original_picker_object {
        "discover-phone-selected-object"
    } else {
        "discover-phone"
    });
    report["connection_object_origin"] = json!(if use_original_picker_object {
        "original_os_picker_after_fresh_group_validation"
    } else {
        "fresh_paired_inventory_group_member"
    });
    let Some(main) = MainThreadMarker::new() else {
        return (report, 3);
    };
    if IOBluetoothHostController::class()
        .class_method(sel!(defaultController))
        .is_none()
    {
        report["status"] = json!("controller_api_unavailable");
        return (report, 3);
    }
    let Some(controller) = (unsafe { IOBluetoothHostController::defaultController() }) else {
        report["status"] = json!("controller_unavailable");
        return (report, 3);
    };
    if !context_ready(&controller, &mut report, "context_before_selector") {
        return (report, 3);
    }
    let Some(device) = selected_fresh_paired(&mut report, main, use_original_picker_object) else {
        return (report, 3);
    };
    let class = device.class();
    let open_selector = sel!(openL2CAPChannelAsync:withPSM:delegate:);
    if class
        .verify_sel::<(*mut *mut IOBluetoothL2CAPChannel, u16, *mut AnyObject), i32>(open_selector)
        .is_err()
    {
        report["status"] = json!("fresh_open_api_abi_unsupported");
        return (report, 3);
    }
    if !context_ready(&controller, &mut report, "context_before_open") {
        return (report, 3);
    }
    // Strict AVDTP one-signaling-channel-per-device-pair precondition. Recheck
    // the retained fresh identity immediately before open, never disconnect it.
    let connected = unsafe { device.isConnected() };
    let paired = unsafe { device.isPaired() };
    report["phone_connected_before_open"] = json!(connected);
    report["phone_paired_before_open"] = json!(paired);
    if connected || !paired {
        report["status"] = json!("fresh_disconnected_paired_phone_required");
        return (report, 3);
    }
    let (sender, receiver) = mpsc::sync_channel(MAX_EVENTS);
    let overflow = Arc::new(AtomicBool::new(false));
    let delegate = DiscoverDelegate::new(sender, overflow.clone());
    // One delegate survives until process exit, including callbacks during open.
    let delegate = Retained::into_raw(delegate);
    let delegate = unsafe { &*delegate };
    let watchdog = match NativeWatchdog::arm() {
        Ok(watchdog) => watchdog,
        Err(error) => {
            report["status"] = json!("native_watchdog_unavailable");
            report["error_kind"] = json!(format!("{:?}", error.kind()));
            return (report, 3);
        }
    };
    let mut channel = None;
    report["native_watchdog_seconds"] = json!(NATIVE_DEADLINE.as_secs());
    report["native_watchdog_armed_before_open"] = json!(true);
    report["connection_started"] = json!(true);
    // The public Async contract says new channel; its implementation can bind
    // an underlying existing channel. Our experimental ownership basis is the
    // immediately disconnected device plus outgoing/PSM/own-delegate checks,
    // not an atomic reservation against concurrent system activity.
    // The delegate is ready before entering any native open call.
    let opened = unsafe {
        device.openL2CAPChannelAsync_withPSM_delegate(Some(&mut channel), PSM, Some(delegate))
    };
    report["open_ioreturn"] = json!(opened);
    report["output_channel_present"] = json!(channel.is_some());
    if opened != 0 {
        report["status"] = json!("fresh_open_not_started");
        return (report, 5);
    }
    let Some(channel) = channel else {
        report["status"] = json!("fresh_open_returned_no_channel");
        return (report, 5);
    };
    let our_delegate = unsafe { channel.delegate() }
        .as_deref()
        .is_some_and(|assigned| std::ptr::eq(assigned, delegate as &AnyObject));
    let outgoing = !unsafe { channel.isIncoming() };
    let actual_psm = unsafe { channel.PSM() };
    report["returned_channel_has_supplied_delegate"] = json!(our_delegate);
    report["returned_channel_outgoing"] = json!(outgoing);
    report["returned_channel_psm"] = json!(actual_psm);
    // A contradictory delegate/incoming/PSM result is never adopted or closed.
    if !our_delegate || !outgoing || actual_psm != PSM {
        report["status"] = json!("fresh_channel_ownership_not_established");
        report["cleanup_unconfirmed"] = json!(true);
        return (report, 5);
    }
    let mut owned = OwnedChannel {
        channel,
        cleanup_attempted: false,
    };
    let identity = channel_id(&owned.channel);
    report["fresh_channel_owned"] = json!(true);
    let mut code = 5;
    {
        let request = Box::leak(Box::new(avdtp::DISCOVER_REQUEST));
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut sent = false;
        let mut ignored = 0u32;
        let mut closed_observed = false;
        let run_loop = NSRunLoop::mainRunLoop();
        report["status"] = json!("discover_timeout");
        'wait: while Instant::now() < deadline {
            if overflow.load(Ordering::Relaxed) {
                report["status"] = json!("callback_queue_overflow");
                break;
            }
            // Bound event processing as well as elapsed run-loop waiting.
            for _ in 0..MAX_EVENTS {
                if Instant::now() >= deadline {
                    break 'wait;
                }
                let Ok(event) = receiver.try_recv() else {
                    break;
                };
                match event {
                    Event::Open(id, status) if id == identity => {
                        report["open_complete_ioreturn"] = json!(status);
                        if status != 0 {
                            report["status"] = json!("channel_open_failed");
                            break 'wait;
                        }
                        if !sent {
                            let written = unsafe {
                                owned.channel.writeAsync_length_refcon(
                                    request.as_mut_ptr().cast(),
                                    2,
                                    request.as_mut_ptr().cast(),
                                )
                            };
                            report["discover_write_ioreturn"] = json!(written);
                            if written != 0 {
                                report["status"] = json!("discover_write_failed");
                                break 'wait;
                            }
                            sent = true;
                        }
                    }
                    Event::Data(id, bytes) if id == identity => {
                        if !sent
                            || bytes.first().is_some_and(|header| {
                                header >> 4 != avdtp::TRANSACTION || header & 3 == 0
                            })
                        {
                            ignored += 1;
                            continue;
                        }
                        let parsed = avdtp::parse(&bytes);
                        let has_source = parsed.as_ref().is_ok_and(|endpoints| {
                            endpoints
                                .iter()
                                .any(|endpoint| endpoint.source && endpoint.media_type == 0)
                        });
                        report["discover_response"] =
                            avdtp::report(parsed, "actual_phone_signaling").0;
                        report["status"] = json!(if has_source {
                            "discover_audio_source_identified"
                        } else {
                            "discover_no_audio_source_proven"
                        });
                        code = if has_source { 0 } else { 5 };
                        break 'wait;
                    }
                    Event::Write(id, status) if id == identity => {
                        report["write_complete_ioreturn"] = json!(status);
                        if status != 0 {
                            report["status"] = json!("discover_write_failed");
                            break 'wait;
                        }
                    }
                    Event::Closed(id) if id == identity => {
                        closed_observed = true;
                        report["status"] = json!("channel_closed_before_response");
                        break 'wait;
                    }
                    Event::Oversized(id) if id == identity => {
                        report["status"] = json!("response_limit_exceeded");
                        break 'wait;
                    }
                    _ => {
                        ignored += 1;
                    }
                }
            }
            run_loop.runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(0.02));
            thread::sleep(Duration::from_millis(2));
        }
        report["owned_channel_closed_callback_observed"] = json!(closed_observed);
        report["ignored_events"] = json!(ignored);
        report["discover_request_sent"] = json!(sent);
    }
    let (detached, closed) = owned.close();
    report["delegate_detach_ioreturn"] = json!(detached);
    report["owned_channel_close_request_ioreturn"] = json!(closed);
    report["owned_channel_close_request_accepted"] = json!(closed == 0);
    report["owned_channel_retained_until_process_exit"] = json!(true);
    // A successful close request alone does not confirm native closure.
    report["owned_channel_close_confirmed"] = report["owned_channel_closed_callback_observed"]
        .as_bool()
        .unwrap_or(false)
        .into();
    report["device_close_connection_called"] = json!(false);
    if detached != 0 || closed != 0 {
        report["operation_status_before_cleanup_failure"] = report["status"].clone();
        report["status"] = json!("owned_channel_cleanup_failed");
        code = 5;
    }
    // Complete the own-channel retain before disarming the native deadline.
    drop(owned);
    // Dropping the sender disarms the Rust-only deadline on every normal path.
    drop(watchdog);
    (report, code)
}
