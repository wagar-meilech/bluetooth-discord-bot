use objc2::{ClassType, MainThreadMarker, msg_send, sel};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::NSString;
use objc2_io_bluetooth::IOBluetoothDevice;
use objc2_io_bluetooth_ui::{
    IOBluetoothPairingController, kIOBluetoothServiceBrowserControllerOptionsNone,
    kIOBluetoothUISuccess, kIOBluetoothUIUserCanceledErr,
};
use serde_json::{Value, json};

pub(super) fn run() -> (Value, u8) {
    let mut report = json!({"schema": 1, "mode": "pair-device-panel", "operator_managed": true,
        "os_panel_may_start_inquiry": true, "automatic_pin_confirmation": false,
        "disconnect_when_done_requested": false, "audio_channel_open_requested": false,
        "live_audio": "not_tested", "full_feasibility_gate": "not_passed"});
    let Some(main) = MainThreadMarker::new() else {
        report["status"] = json!("pairing_panel_requires_main_thread");
        return (report, 3);
    };
    if IOBluetoothPairingController::class()
        .class_method(sel!(pairingController))
        .is_none()
    {
        report["status"] = json!("pairing_panel_unavailable");
        return (report, 3);
    }
    let application = NSApplication::sharedApplication(main);
    application.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    #[allow(deprecated)]
    application.activateIgnoringOtherApps(true);
    // SAFETY: Public OS-managed UI on main. No DevicePair delegate, PIN override,
    // global class/service mutation, automatic choice or post-pair audio open.
    let Some(panel) = (unsafe { IOBluetoothPairingController::pairingController(main) }) else {
        report["status"] = json!("pairing_panel_unavailable");
        return (report, 3);
    };
    unsafe {
        panel.clearAllowedUUIDs();
        panel.setOptions(kIOBluetoothServiceBrowserControllerOptionsNone);
        panel.setTitle(Some(&NSString::from_str("Pair iPhone")));
        panel.setDescriptionText(Some(&NSString::from_str(
            "Select your iPhone and confirm the matching code on both devices.",
        )));
    }
    // The binding and concrete runtime can disagree on modal return width.
    // Select i32/isize only after checking the actual object's class, and
    // retain objc2 verification instead of suppressing the mismatch.
    let runtime_class = panel.class();
    let response: isize = if runtime_class.verify_sel::<(), i32>(sel!(runModal)).is_ok() {
        report["run_modal_return_encoding"] = json!("i32");
        // SAFETY: Verified concrete public-selector return encoding.
        let response: i32 = unsafe { msg_send![&*panel, runModal] };
        response as isize
    } else if runtime_class
        .verify_sel::<(), isize>(sel!(runModal))
        .is_ok()
    {
        report["run_modal_return_encoding"] = json!("isize");
        // SAFETY: Verified concrete public-selector return encoding. The OS
        // owns selection/PIN UI until operator completion or cancellation.
        unsafe { msg_send![&*panel, runModal] }
    } else {
        report["status"] = json!("pairing_panel_return_abi_unsupported");
        return (report, 3);
    };
    report["panel_response"] = json!(response);
    if response == kIOBluetoothUIUserCanceledErr as isize {
        report["status"] = json!("pairing_cancelled");
        return (report, 3);
    }
    if response != kIOBluetoothUISuccess as isize {
        report["status"] = json!("pairing_panel_failed");
        return (report, 3);
    }
    let Some(results) = (unsafe { panel.getResults() }) else {
        report["status"] = json!("pairing_results_unavailable");
        return (report, 3);
    };
    let mut paired = Vec::new();
    for index in 0..results.count() {
        let Ok(device) = results.objectAtIndex(index).downcast::<IOBluetoothDevice>() else {
            report["status"] = json!("pairing_result_unexpected_type");
            return (report, 3);
        };
        // SAFETY: Read public state on the selected OS-returned result only.
        paired.push(unsafe { device.isPaired() });
    }
    let verified = !paired.is_empty() && paired.iter().all(|paired| *paired);
    report["result_count"] = json!(paired.len());
    report["result_is_paired"] = json!(paired);
    report["status"] = json!(if verified {
        "pairing_completed"
    } else {
        "pairing_not_verified"
    });
    (report, if verified { 0 } else { 3 })
}
