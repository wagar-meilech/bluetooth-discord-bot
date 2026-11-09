use objc2::{ClassType, msg_send, rc::Retained, sel};
use objc2_foundation::{
    NSData, NSDictionary, NSObject, NSProcessInfo, NSPropertyListReadOptions,
    NSPropertyListSerialization, NSString,
};
use objc2_io_bluetooth::{
    BluetoothHCIPowerState, IOBluetoothHostController, IOBluetoothSDPDataElement,
    IOBluetoothSDPServiceRecord, IOBluetoothSDPUUID,
};
use serde_json::{Value, json};
use std::{ptr, time::Instant};

const AVDTP_PSM: u16 = 0x0019;

pub fn run(mode: &str) -> (Value, u8) {
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
    // SAFETY: This is the public, parameterless lookup, not a power setter.
    let Some(controller) = (unsafe { IOBluetoothHostController::defaultController() }) else {
        report["status"] = json!("controller_unavailable");
        report["detail"] = json!(
            "No controller returned; hardware or OS access may be unavailable. No settings were changed."
        );
        return (report, 3);
    };
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
    if mode == "preflight" {
        report["status"] = json!("preflight_available");
        report["mutation_attempted"] = json!(false);
        return (report, 0);
    }

    // Only the explicit mode reaches publication. Parse the fixed, typed plist
    // through Foundation, rather than hand-writing Objective-C collection FFI.
    let Some(dictionary) = service_definition() else {
        report["status"] = json!("invalid_builtin_service_record");
        return (report, 4);
    };
    let started = Instant::now();
    report["mutation_attempted"] = json!(true);
    // SAFETY: This non-null NSDictionary was parsed from the bundled immutable
    // SDP plist, using Apple's documented types. It has no LocalAttributes,
    // Persistent, TargetApplication, or supplied service-record handle.
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
    // Remove before formatting/returning diagnostic results, on every branch.
    let removal_result = owned.remove();
    report["requested_psm"] = json!(AVDTP_PSM);
    report["psm_read_ioreturn"] = json!(psm_result);
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
    } else if psm_result != 0 {
        ("assigned_psm_unavailable", 5)
    } else if assigned_psm != AVDTP_PSM {
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

fn service_definition() -> Option<Retained<NSDictionary>> {
    let data = NSData::with_bytes(include_bytes!("../audio-sink.plist"));
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
    let Some(dictionary) = service_definition() else {
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
