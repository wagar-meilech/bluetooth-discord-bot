mod discover;
mod paired;
mod pairing;
use objc2::{ClassType, msg_send, rc::Retained, sel};
use objc2_core_bluetooth::CBManager;
use objc2_foundation::{
    NSBundle, NSData, NSDate, NSDictionary, NSObject, NSProcessInfo, NSPropertyListReadOptions,
    NSPropertyListSerialization, NSRunLoop, NSString,
};
use objc2_io_bluetooth::{
    BluetoothHCIPowerState, IOBluetoothHostController, IOBluetoothSDPDataElement,
    IOBluetoothSDPServiceRecord, IOBluetoothSDPUUID,
};
use serde_json::{Value, json};
use std::{
    ptr, thread,
    time::{Duration, Instant},
};

const AVDTP_PSM: u16 = 0x0019;
const OWN_CANDIDATE_HANDLE: u32 = 0x6A2D0001;

pub fn run(mode: &str) -> (Value, u8) {
    if mode == "discover-phone" {
        return discover::run();
    }
    if mode == "pair-device-panel" {
        return pairing::run();
    }
    if mode == "list-paired" {
        return paired::list();
    }
    if mode == "diagnose-context" {
        return diagnose_context();
    }
    let warmup_requested = mode.ends_with("-warmup");
    let publication_mode = mode.strip_suffix("-warmup").unwrap_or(mode);
    // Inspection never starts discovery or accesses remote device identifiers.
    let host_class = IOBluetoothHostController::class();
    let record_class = IOBluetoothSDPServiceRecord::class();
    let selectors_available = host_class.class_method(sel!(defaultController)).is_some()
        && host_class.instance_method(sel!(powerState)).is_some()
        && host_class.instance_method(sel!(classOfDevice)).is_some()
        && record_class
            .class_method(sel!(publishedServiceRecordWithDictionary:))
            .is_some()
        && record_class.instance_method(sel!(getL2CAPPSM:)).is_some()
        && record_class
            .instance_method(sel!(removeServiceRecord))
            .is_some();
    let mut report = json!({
        "schema": 1,
        "mode": mode,
        "os": NSProcessInfo::processInfo().operatingSystemVersionString().to_string(),
        "arch": std::env::consts::ARCH,
        "public_selectors_available": selectors_available,
        "live_audio": "not_tested",
        "coexistence": "not_tested",
        "full_feasibility_gate": "not_passed",
        "receiver_implemented": false
    });
    if !selectors_available {
        report["status"] = json!("public_api_unavailable");
        return (report, 3);
    }
    if mode == "validate-sdp" {
        return validate_service_definition(report);
    }
    if mode == "validate-own-handle" {
        return validate_own_handle(report);
    }
    // SAFETY: This is the public, parameterless lookup, not a power setter.
    let Some(controller) = (unsafe { IOBluetoothHostController::defaultController() }) else {
        report["status"] = json!("controller_unavailable");
        report["detail"] = json!(
            "No controller returned; hardware or OS access may be unavailable. No settings were changed."
        );
        return (report, 3);
    };
    if warmup_requested {
        let Some((power_before, class_before)) = controller_snapshot(&controller) else {
            report["status"] = json!("controller_getter_abi_unsupported_before_warmup");
            return (report, 3);
        };
        report["controller_power_before_warmup"] = json!(power_before);
        report["controller_class_before_warmup"] = json!(class_before);
        let (elapsed, turns) = warm_main_run_loop();
        report["main_runloop_warmup_requested_ms"] = json!(1000);
        report["main_runloop_warmup_elapsed_ms"] = json!(elapsed);
        report["main_runloop_turns"] = json!(turns);
    }
    let Some((power, class_before)) = controller_snapshot(&controller) else {
        report["status"] = json!("controller_getter_abi_unsupported");
        return (report, 3);
    };
    report["controller_power"] = json!(power);
    report["controller_class_before"] = json!(class_before);
    report["controller_identity"] =
        json!("default controller; built-in transport not established by this getter");
    if power != BluetoothHCIPowerState::ON.0 {
        report["status"] = json!("controller_not_powered_on");
        report["detail"] = json!("Read-only result; the probe does not enable Bluetooth.");
        return (report, 3);
    }
    if publication_mode == "preflight" {
        report["status"] = json!("preflight_available");
        report["mutation_attempted"] = json!(false);
        return (report, 0);
    }

    // Only the explicit mode reaches publication. Parse the fixed, typed plist
    // through Foundation, rather than hand-writing Objective-C collection FFI.
    let Some(dictionary) = service_definition(publication_mode) else {
        report["status"] = json!("invalid_builtin_service_record");
        return (report, 4);
    };
    report["service_fixture"] = json!(service_fixture(publication_mode).map(|(name, _)| name));
    let generic_control = matches!(
        publication_mode,
        "probe-generic-l2cap"
            | "probe-generic-avdtp-psm"
            | "probe-generic-avdtp-own-handle"
            | "probe-generic-l2cap-own-handle"
            | "probe-generic-l2cap-own-handle-number"
            | "probe-generic-avdtp-own-handle-number"
    );
    let requested_psm = if matches!(
        publication_mode,
        "probe-generic-l2cap"
            | "probe-generic-l2cap-own-handle"
            | "probe-generic-l2cap-own-handle-number"
    ) {
        0x1001
    } else {
        AVDTP_PSM
    };
    report["experiment"] = json!(match publication_mode {
        "probe-sdp-bare-keys" => "a2dp_bare_decimal_keys",
        "probe-generic-l2cap" => "generic_custom_uuid_l2cap_control",
        "probe-generic-avdtp-psm" => "generic_custom_uuid_assigned_avdtp_psm_control",
        "probe-generic-avdtp-own-handle" => "generic_assigned_psm_fresh_own_handle_control",
        "probe-generic-l2cap-own-handle" => "generic_dynamic_psm_fresh_own_handle_control",
        "probe-generic-l2cap-own-handle-number" =>
            "generic_dynamic_psm_plain_number_own_handle_control",
        "probe-generic-avdtp-own-handle-number" =>
            "generic_assigned_psm_plain_number_own_handle_control",
        _ => "a2dp_original_hex_label_keys",
    });
    report["attribute_key_style"] = json!(if publication_mode == "probe-sdp" {
        "hex_with_label"
    } else {
        "bare_decimal"
    });
    report["generic_control_only"] = json!(generic_control);
    report["requested_psm"] = json!(requested_psm);
    let own_handle_case = matches!(
        publication_mode,
        "probe-generic-avdtp-own-handle"
            | "probe-generic-l2cap-own-handle"
            | "probe-generic-l2cap-own-handle-number"
            | "probe-generic-avdtp-own-handle-number"
    );
    if own_handle_case {
        report["requested_service_record_handle"] = json!(OWN_CANDIDATE_HANDLE);
        report["handle_encoding"] = json!(if publication_mode.ends_with("-number") {
            "plain_unsigned32_number_shortcut"
        } else {
            "explicit_unsigned_four_byte_element"
        });
        if record_class
            .instance_method(sel!(getServiceRecordHandle:))
            .is_none()
        {
            report["status"] = json!("owned_handle_getter_unavailable");
            return (report, 3);
        }
    }
    report["record_returned"] = json!(false);
    report["cleanup_attempted"] = json!(false);
    report["mutation_attempted"] = json!(false);
    let protocol_key = if publication_mode == "probe-sdp" {
        "0x0004 ProtocolDescriptorList"
    } else {
        "4"
    };
    let parsed_psm = parsed_l2cap_psm(&dictionary, protocol_key);
    let parsed_handle = if own_handle_case {
        parsed_unsigned_attribute(&dictionary, "0")
    } else {
        None
    };
    let fixture_valid = parsed_psm == Some(requested_psm)
        && (!own_handle_case || parsed_handle == Some((1, 4, OWN_CANDIDATE_HANDLE)));
    report["fixture_parsed_psm"] = json!(parsed_psm);
    report["fixture_parsed_requested_handle"] = json!(parsed_handle.map(|(_, _, value)| value));
    report["prepublication_fixture_valid"] = json!(fixture_valid);
    if !fixture_valid {
        report["status"] = json!("prepublication_fixture_mismatch");
        return (report, 4);
    }
    let started = Instant::now();
    report["mutation_attempted"] = json!(true);
    // SAFETY: This non-null NSDictionary was parsed from the bundled immutable
    // SDP plist, using Apple's documented types. It has no LocalAttributes,
    // Persistent, or TargetApplication. Only the explicitly named own-handle
    // control includes a fresh nonreserved requested handle; no OS/log handle.
    let Some(record) = (unsafe {
        IOBluetoothSDPServiceRecord::publishedServiceRecordWithDictionary(Some(&dictionary))
    }) else {
        report["status"] = json!("publication_rejected");
        report["detail"] = json!(
            "The public API returned nil; this does not distinguish permission denial from a service/stack conflict."
        );
        return (report, 5);
    };
    let mut owned = OwnedRegistration {
        record: Some(record),
    };
    report["record_returned"] = json!(true);
    let mut assigned_psm = 0u16;
    // SAFETY: We hold exactly the newly published record and provide a valid
    // writable u16. No discovered/system records or channels are touched.
    let psm_result = unsafe {
        owned
            .record
            .as_ref()
            .unwrap()
            .getL2CAPPSM(&mut assigned_psm)
    };
    let owned_handle = if own_handle_case {
        let mut actual_handle = 0u32;
        // SAFETY: Read only the just-returned owned record, with valid output
        // storage. Never look up/adopt a record by the requested or logged handle.
        let result = unsafe {
            owned
                .record
                .as_ref()
                .unwrap()
                .getServiceRecordHandle(&mut actual_handle)
        };
        Some((result, actual_handle))
    } else {
        None
    };
    // Remove before formatting/returning diagnostic results, on every branch.
    let removal_result = owned.remove();
    if let Some((result, handle)) = owned_handle {
        report["owned_handle_read_ioreturn"] = json!(result);
        report["owned_service_record_handle"] = if result == 0 {
            json!(handle)
        } else {
            Value::Null
        };
    }
    report["cleanup_attempted"] = json!(true);
    report["psm_read_ioreturn"] = json!(psm_result);
    report["psm_exact"] = json!(psm_result == 0 && assigned_psm == requested_psm);
    report["assigned_psm"] = if psm_result == 0 {
        json!(assigned_psm)
    } else {
        Value::Null
    };
    report["removal_ioreturn"] = json!(removal_result);
    report["registration_duration_ms"] = json!(started.elapsed().as_millis());
    let Some((power_after, class_after)) = controller_snapshot(&controller) else {
        report["status"] = json!("controller_getter_abi_unsupported_after_cleanup");
        return (report, 7);
    };
    report["controller_power_after"] = json!(power_after);
    report["controller_class_after"] = json!(class_after);
    report["controller_getters_unchanged"] =
        json!(power == power_after && class_before == class_after);
    report["coexistence"] =
        json!("not_tested; controller getters do not prove peripheral/audio behavior");
    let (status, code) = if removal_result != 0 {
        ("cleanup_failed", 6)
    } else if power != power_after || class_before != class_after {
        ("controller_state_changed", 7)
    } else if owned_handle.is_some_and(|(result, _)| result != 0) {
        ("owned_handle_unavailable", 5)
    } else if psm_result != 0 {
        ("assigned_psm_unavailable", 5)
    } else if assigned_psm != requested_psm {
        ("psm_remapped", 5)
    } else {
        ("exact_psm_published_and_removed_prerequisite_only", 0)
    };
    report["status"] = json!(status);
    (report, code)
}

// Holds only the object returned by our own publication. The explicit remove
// path records its result; Drop is a fallback for later Rust error/panic paths.
// The record is also transient per Apple's contract and removed on process exit.
struct OwnedRegistration {
    record: Option<Retained<IOBluetoothSDPServiceRecord>>,
}

impl OwnedRegistration {
    fn remove(&mut self) -> i32 {
        let Some(record) = self.record.as_ref() else {
            return 0;
        };
        // SAFETY: This guard only owns the result of this process's publication.
        let result = unsafe { record.removeServiceRecord() };
        if result == 0 {
            self.record.take();
        }
        result
    }
}

impl Drop for OwnedRegistration {
    fn drop(&mut self) {
        if self.record.is_some() {
            let result = self.remove();
            if result != 0 {
                eprintln!(
                    "{}",
                    json!({"status": "cleanup_retry_failed", "ioreturn": result, "record_lifetime": "transient_until_process_exit"})
                );
            }
        }
    }
}

// The shipped framework on macOS 26.7.1 encodes powerState as signed int,
// whereas the SDK-generated BluetoothHCIPowerState uses unsigned int. Check
// runtime metadata before choosing the matching objc2 typed call; never disable
// objc2 signature verification or reinterpret an unknown return encoding.
fn controller_snapshot(controller: &IOBluetoothHostController) -> Option<(u32, u32)> {
    let class = IOBluetoothHostController::class();
    let power = if class.verify_sel::<(), i32>(sel!(powerState)).is_ok() {
        // SAFETY: The runtime signature was verified as a parameterless i32
        // getter. This is the same public selector as the generated binding.
        let value: i32 = unsafe { msg_send![controller, powerState] };
        value as u32
    } else if class
        .verify_sel::<(), BluetoothHCIPowerState>(sel!(powerState))
        .is_ok()
    {
        // SAFETY: Runtime and generated binding agree on the return encoding.
        unsafe { controller.powerState() }.0
    } else {
        return None;
    };
    let device_class = if class.verify_sel::<(), u32>(sel!(classOfDevice)).is_ok() {
        // SAFETY: Runtime and generated binding agree on the return encoding.
        unsafe { controller.classOfDevice() }
    } else if class.verify_sel::<(), i32>(sel!(classOfDevice)).is_ok() {
        // SAFETY: Runtime signature was verified as a parameterless i32 getter.
        let value: i32 = unsafe { msg_send![controller, classOfDevice] };
        value as u32
    } else {
        return None;
    };
    Some((power, device_class))
}

fn service_fixture(mode: &str) -> Option<(&'static str, &'static [u8])> {
    // Keep every publication case explicit. An unknown mode must not silently
    // publish a different record while reporting the requested experiment.
    Some(match mode {
        "probe-sdp" | "validate-sdp" => ("audio-sink.plist", include_bytes!("../audio-sink.plist")),
        "probe-sdp-bare-keys" => (
            "audio-sink-bare-keys.plist",
            include_bytes!("../audio-sink-bare-keys.plist"),
        ),
        "probe-generic-l2cap" => (
            "generic-l2cap-control.plist",
            include_bytes!("../generic-l2cap-control.plist"),
        ),
        "probe-generic-avdtp-psm" => (
            "generic-avdtp-psm-control.plist",
            include_bytes!("../generic-avdtp-psm-control.plist"),
        ),
        "probe-generic-avdtp-own-handle" => (
            "generic-avdtp-own-handle-control.plist",
            include_bytes!("../generic-avdtp-own-handle-control.plist"),
        ),
        "probe-generic-l2cap-own-handle" => (
            "generic-dynamic-own-handle-control.plist",
            include_bytes!("../generic-dynamic-own-handle-control.plist"),
        ),
        "probe-generic-l2cap-own-handle-number" => (
            "generic-dynamic-own-handle-number-control.plist",
            include_bytes!("../generic-dynamic-own-handle-number-control.plist"),
        ),
        "probe-generic-avdtp-own-handle-number" => (
            "generic-avdtp-own-handle-number-control.plist",
            include_bytes!("../generic-avdtp-own-handle-number-control.plist"),
        ),
        _ => return None,
    })
}

fn service_definition(mode: &str) -> Option<Retained<NSDictionary>> {
    let (_, bytes) = service_fixture(mode)?;
    let data = NSData::with_bytes(bytes);
    // SAFETY: A null format output is permitted; the immutable plist is owned.
    let plist = unsafe {
        NSPropertyListSerialization::propertyListWithData_options_format_error(
            &data,
            NSPropertyListReadOptions::Immutable,
            ptr::null_mut(),
        )
    }
    .ok()?;
    plist.downcast::<NSDictionary>().ok()
}

fn validate_service_definition(mut report: Value) -> (Value, u8) {
    report["mutation_attempted"] = json!(false);
    report["server_publication_attempted"] = json!(false);
    let Some(dictionary) = service_definition("validate-sdp") else {
        report["status"] = json!("invalid_builtin_service_record");
        return (report, 4);
    };
    // Unlike the canonical-record constructor, withElementValue explicitly
    // accepts the documented plist shorthand for each attribute value.
    let fields = [
        "0x0001 ServiceClassIDList",
        "0x0004 ProtocolDescriptorList",
        "0x0005 BrowseGroupList",
        "0x0009 BluetoothProfileDescriptorList",
        "0x0100 ServiceName",
        "0x0311 SupportedFeatures",
    ];
    let mut protocol = None;
    for field in fields {
        let key = NSString::from_str(field);
        let parsed = dictionary.objectForKey(&key).and_then(|value| {
            let object = value.downcast_ref::<NSObject>()?;
            // SAFETY: The fixed plist contains only Apple's documented
            // NSData/string/array/typed-number-dictionary shorthand objects.
            unsafe { IOBluetoothSDPDataElement::withElementValue(Some(object)) }
        });
        let Some(parsed) = parsed else {
            report["status"] = json!("local_sdp_attribute_parser_rejected");
            report["attribute"] = json!(field);
            return (report, 4);
        };
        if field == "0x0004 ProtocolDescriptorList" {
            protocol = Some(parsed);
        }
    }
    let psm = protocol.and_then(|protocol| unsafe {
        // SAFETY: These retained objects were produced by the public parser.
        // Check every optional value, class and sequence length before access.
        let descriptors = protocol.getArrayValue()?;
        if descriptors.count() != 2 {
            return None;
        }
        let l2cap = descriptors
            .objectAtIndex(0)
            .downcast::<IOBluetoothSDPDataElement>()
            .ok()?;
        let entries = l2cap.getArrayValue()?;
        if entries.count() != 2 {
            return None;
        }
        let uuid_element = entries
            .objectAtIndex(0)
            .downcast::<IOBluetoothSDPDataElement>()
            .ok()?;
        let uuid = uuid_element.getUUIDValue()?;
        let expected = IOBluetoothSDPUUID::uuid16(0x0100)?;
        if !uuid.isEqualToUUID(Some(&expected)) {
            return None;
        }
        let number_element = entries
            .objectAtIndex(1)
            .downcast::<IOBluetoothSDPDataElement>()
            .ok()?;
        Some(number_element.getNumberValue()?.unsignedShortValue())
    });
    report["parsed_psm"] = json!(psm);
    report["attributes_parsed"] = json!(fields.len());
    let valid = psm == Some(AVDTP_PSM);
    report["status"] = json!(if valid {
        "local_sdp_attributes_accepted_expected_psm"
    } else {
        "local_sdp_protocol_list_mismatch"
    });
    report["detail"] = json!(
        "In-memory attribute parsing only; no server publication, channel registration, device connection, or audio reception."
    );
    (report, if valid { 0 } else { 4 })
}

fn diagnose_context() -> (Value, u8) {
    let bundle = NSBundle::mainBundle();
    let purpose_key = NSString::from_str("NSBluetoothAlwaysUsageDescription");
    let mut report = json!({
        "schema": 1,
        "mode": "diagnose-context",
        "mutation_attempted": false,
        "server_publication_attempted": false,
        "manager_instantiated": false,
        "bundle_identifier": bundle.bundleIdentifier().map(|identifier| identifier.to_string()),
        "application_bundle": bundle.bundlePath().to_string().ends_with(".app"),
        "bluetooth_usage_description_present": bundle.objectForInfoDictionaryKey(&purpose_key).is_some(),
        "authorization_scope": "CoreBluetooth only; not proof of IOBluetooth publication permission",
        "live_audio": "not_tested",
        "coexistence": "not_tested",
        "full_feasibility_gate": "not_passed",
        "receiver_implemented": false
    });
    if CBManager::class()
        .class_method(sel!(authorization))
        .is_none()
    {
        report["status"] = json!("authorization_class_getter_unavailable");
        return (report, 3);
    }
    // SAFETY: This is the public class getter, explicitly documented for checking
    // authorization before allocating a manager. No manager is instantiated,
    // scanning/advertising started, or consent request forced by this code.
    let authorization = unsafe { CBManager::authorization_class() }.0;
    report["core_bluetooth_authorization"] = json!({
        "raw": authorization,
        "name": match authorization {
            0 => "not_determined", 1 => "restricted", 2 => "denied", 3 => "allowed_always", _ => "unknown"
        }
    });
    report["status"] = json!("context_observed");
    (report, 0)
}

// Process only this CLI's public Foundation main run loop for a bounded window
// after controller initialization. No manager, inquiry, listener, or radio setter
// is created. Back off if the loop has no sources instead of busy-spinning.
fn warm_main_run_loop() -> (u128, u32) {
    let started = Instant::now();
    let window = Duration::from_secs(1);
    let run_loop = NSRunLoop::mainRunLoop();
    let mut turns = 0;
    while started.elapsed() < window {
        let remaining = window.saturating_sub(started.elapsed());
        let slice = remaining.min(Duration::from_millis(20));
        run_loop.runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(slice.as_secs_f64()));
        turns += 1;
        let remaining = window.saturating_sub(started.elapsed());
        if !remaining.is_zero() {
            thread::sleep(remaining.min(Duration::from_millis(2)));
        }
    }
    (started.elapsed().as_millis(), turns)
}

fn validate_own_handle(mut report: Value) -> (Value, u8) {
    report["mutation_attempted"] = json!(false);
    report["server_publication_attempted"] = json!(false);
    let mut entries = Vec::new();
    let mut all_valid = true;
    for (mode, encoding) in [
        (
            "probe-generic-l2cap-own-handle",
            "explicit_unsigned_four_byte_element",
        ),
        (
            "probe-generic-l2cap-own-handle-number",
            "plain_unsigned32_number_shortcut",
        ),
    ] {
        let definition = service_definition(mode);
        let psm = definition
            .as_ref()
            .and_then(|dictionary| parsed_l2cap_psm(dictionary, "4"));
        let parsed = definition.and_then(|dictionary| {
            let key = NSString::from_str("0");
            let value = dictionary.objectForKey(&key)?;
            let object = value.downcast_ref::<NSObject>()?;
            // SAFETY: Only the fixed documented typed-number/NSNumber inputs
            // are parsed; these objects are never registered with the server.
            unsafe { IOBluetoothSDPDataElement::withElementValue(Some(object)) }
        });
        let (kind, size, value) = match parsed {
            Some(element) => unsafe {
                // SAFETY: Access only the locally parsed retained element and
                // check its optional numeric representation before reading it.
                (
                    Some(element.getTypeDescriptor()),
                    Some(element.getSize()),
                    element
                        .getNumberValue()
                        .map(|number| number.unsignedIntValue()),
                )
            },
            None => (None, None, None),
        };
        let valid = kind == Some(1)
            && size == Some(4)
            && value == Some(OWN_CANDIDATE_HANDLE)
            && psm == Some(0x1001);
        all_valid &= valid;
        entries.push(json!({"encoding": encoding, "service_fixture": service_fixture(mode).map(|(name, _)| name), "parsed_psm": psm, "type": kind, "size": size, "value": value, "valid_unsigned32_candidate": valid}));
    }
    report["handle_values"] = json!(entries);
    report["status"] = json!(if all_valid {
        "local_handle_values_valid"
    } else {
        "local_handle_value_invalid"
    });
    (report, if all_valid { 0 } else { 4 })
}

// Semantic checks use the public, in-memory SDP parser on the actual selected
// dictionary. They never register services or inspect OS-owned records.
fn parsed_attribute(
    dictionary: &NSDictionary,
    key: &str,
) -> Option<Retained<IOBluetoothSDPDataElement>> {
    let key = NSString::from_str(key);
    let value = dictionary.objectForKey(&key)?;
    let object = value.downcast_ref::<NSObject>()?;
    // SAFETY: Fixed fixture values use Apple's documented plist shorthand.
    unsafe { IOBluetoothSDPDataElement::withElementValue(Some(object)) }
}

fn parsed_unsigned_attribute(dictionary: &NSDictionary, key: &str) -> Option<(u8, u32, u32)> {
    let element = parsed_attribute(dictionary, key)?;
    // SAFETY: Access only the retained local parser result; optional numeric
    // representation is checked before reading. No server publication occurs.
    unsafe {
        Some((
            element.getTypeDescriptor(),
            element.getSize(),
            element.getNumberValue()?.unsignedIntValue(),
        ))
    }
}

fn parsed_l2cap_psm(dictionary: &NSDictionary, key: &str) -> Option<u16> {
    let element = parsed_attribute(dictionary, key)?;
    // SAFETY: Retain/check every local parsed sequence and UUID before access.
    unsafe {
        let descriptors = element.getArrayValue()?;
        if descriptors.count() == 0 {
            return None;
        }
        let l2cap = descriptors
            .objectAtIndex(0)
            .downcast::<IOBluetoothSDPDataElement>()
            .ok()?;
        let entries = l2cap.getArrayValue()?;
        if entries.count() != 2 {
            return None;
        }
        let uuid_element = entries
            .objectAtIndex(0)
            .downcast::<IOBluetoothSDPDataElement>()
            .ok()?;
        let uuid = uuid_element.getUUIDValue()?;
        let expected = IOBluetoothSDPUUID::uuid16(0x0100)?;
        if !uuid.isEqualToUUID(Some(&expected)) {
            return None;
        }
        let psm = entries
            .objectAtIndex(1)
            .downcast::<IOBluetoothSDPDataElement>()
            .ok()?;
        if psm.getTypeDescriptor() != 1 || psm.getSize() != 2 {
            return None;
        }
        Some(psm.getNumberValue()?.unsignedShortValue())
    }
}
