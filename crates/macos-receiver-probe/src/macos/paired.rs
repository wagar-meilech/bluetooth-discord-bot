use objc2::{ClassType, sel};
use objc2_io_bluetooth::IOBluetoothDevice;
use serde_json::{Value, json};

pub(super) fn list() -> (Value, u8) {
    let class = IOBluetoothDevice::class();
    let mut report = json!({"schema": 1, "mode": "list-paired", "mutation_attempted": false,
        "inquiry_started": false, "pairing_started": false, "connection_started": false,
        "identifiers_in_stdout": false, "live_audio": "not_tested", "full_feasibility_gate": "not_passed"});
    if class.class_method(sel!(pairedDevices)).is_none() {
        report["status"] = json!("paired_inventory_api_unavailable");
        return (report, 3);
    }
    // SAFETY: Public cached paired-device inventory; no inquiry or connection.
    let Some(devices) = (unsafe { IOBluetoothDevice::pairedDevices() }) else {
        report["status"] = json!("paired_inventory_unavailable");
        report["detail"] =
            json!("nil inventory cannot distinguish empty store from unavailable access");
        return (report, 3);
    };
    let mut rows = Vec::new();
    for index in 0..devices.count() {
        let Ok(device) = devices.objectAtIndex(index).downcast::<IOBluetoothDevice>() else {
            report["status"] = json!("paired_inventory_unexpected_entry");
            return (report, 3);
        };
        // SAFETY: Read-only public state/cached name on a retained device.
        let paired = unsafe { device.isPaired() };
        let connected = unsafe { device.isConnected() };
        #[allow(deprecated)]
        let name = unsafe { device.getName() }.map(|name| name.to_string());
        let iphone_hint = name
            .as_deref()
            .is_some_and(|name| name.to_lowercase().contains("iphone"));
        // Local operator selection display only; evidence should retain stdout,
        // not this stderr name. Never request or display a Bluetooth address.
        eprintln!(
            "{}",
            json!({"local_selection_only": true, "slot": index + 1,
            "cached_name": name, "is_paired": paired, "is_connected": connected})
        );
        rows.push(
            json!({"slot": index + 1, "is_paired": paired, "is_connected": connected,
            "name_hint_is_iphone": iphone_hint}),
        );
    }
    report["status"] = json!("paired_inventory_observed");
    report["paired_device_count"] = json!(rows.len());
    report["devices"] = json!(rows);
    (report, 0)
}
