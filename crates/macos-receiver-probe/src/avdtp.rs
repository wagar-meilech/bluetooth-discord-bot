use serde_json::{Value, json};

pub(crate) const TRANSACTION: u8 = 1;
#[cfg(target_os = "macos")]
pub(crate) const DISCOVER_REQUEST: [u8; 2] = [0x10, 0x01];
pub(crate) const MAX_RESPONSE: usize = 1024;

#[derive(Clone, Debug)]
pub(crate) struct Endpoint {
    pub(crate) seid: u8,
    pub(crate) in_use: bool,
    pub(crate) media_type: u8,
    pub(crate) source: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum DecodeError {
    InvalidHex,
    TooLong,
    Truncated,
    TransactionMismatch,
    FragmentationUnsupported,
    SignalMismatch,
    UnexpectedCommand,
    GeneralReject,
    Rejected(u8),
    MalformedPayload,
    InvalidSeid,
    DuplicateSeid,
}

pub(crate) fn parse(packet: &[u8]) -> Result<Vec<Endpoint>, DecodeError> {
    if packet.len() > MAX_RESPONSE {
        return Err(DecodeError::TooLong);
    }
    let Some(&header) = packet.first() else {
        return Err(DecodeError::Truncated);
    };
    if header >> 4 != TRANSACTION {
        return Err(DecodeError::TransactionMismatch);
    }
    if (header >> 2) & 3 != 0 {
        return Err(DecodeError::FragmentationUnsupported);
    }
    if packet.len() < 2 {
        return Err(DecodeError::Truncated);
    }
    if header & 3 == 1 {
        return Err(DecodeError::GeneralReject);
    }
    // Signal-byte RFA bits are ignored, as are endpoint RFA bits below.
    if packet[1] & 0x3f != 1 {
        return Err(DecodeError::SignalMismatch);
    }
    match header & 3 {
        0 => return Err(DecodeError::UnexpectedCommand),
        3 => {
            return if packet.len() == 3 {
                Err(DecodeError::Rejected(packet[2]))
            } else {
                Err(DecodeError::MalformedPayload)
            };
        }
        2 => {}
        _ => unreachable!(),
    }
    if !(packet.len() - 2).is_multiple_of(2) {
        return Err(DecodeError::MalformedPayload);
    }
    let mut endpoints = Vec::new();
    let mut seen = [false; 64];
    for pair in packet[2..].chunks_exact(2) {
        let seid = pair[0] >> 2;
        if seid == 0 || seid == 63 {
            return Err(DecodeError::InvalidSeid);
        }
        if seen[seid as usize] {
            return Err(DecodeError::DuplicateSeid);
        }
        seen[seid as usize] = true;
        endpoints.push(Endpoint {
            seid,
            in_use: pair[0] & 2 != 0,
            media_type: pair[1] >> 4,
            source: pair[1] & 8 == 0,
        });
    }
    Ok(endpoints)
}

pub(crate) fn report(result: Result<Vec<Endpoint>, DecodeError>, origin: &str) -> (Value, u8) {
    let mut report = json!({"schema":1,"evidence":origin,"transaction_label":TRANSACTION,
        "live_audio":"not_tested","receiver_implemented":false,"full_feasibility_gate":"not_passed"});
    match result {
        Ok(endpoints) => {
            let sources: Vec<_> = endpoints
                .iter()
                .filter(|endpoint| endpoint.source && endpoint.media_type == 0)
                .map(|endpoint| json!({"seid":endpoint.seid,"in_use":endpoint.in_use}))
                .collect();
            report["status"] = json!("discover_accepted");
            report["endpoint_count"] = json!(endpoints.len());
            report["audio_source_seps"] = json!(sources);
            (report, 0)
        }
        Err(error) => {
            let status = match error {
                DecodeError::InvalidHex => "discover_hex_invalid",
                DecodeError::TooLong => "discover_response_too_long",
                DecodeError::Truncated => "discover_response_truncated",
                DecodeError::TransactionMismatch => "discover_transaction_mismatch",
                DecodeError::FragmentationUnsupported => "discover_fragmentation_unsupported",
                DecodeError::SignalMismatch => "discover_signal_mismatch",
                DecodeError::UnexpectedCommand => "discover_unexpected_command",
                DecodeError::GeneralReject => "discover_general_reject",
                DecodeError::Rejected(code) => {
                    report["reject_error_code"] = json!(code);
                    "discover_rejected"
                }
                DecodeError::MalformedPayload => "discover_payload_malformed",
                DecodeError::InvalidSeid => "discover_seid_invalid",
                DecodeError::DuplicateSeid => "discover_seid_duplicate",
            };
            report["status"] = json!(status);
            (report, 4)
        }
    }
}

pub(crate) fn decode_hex(input: &str) -> (Value, u8) {
    let compact: String = input
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    let result = if compact.len() > MAX_RESPONSE * 2 {
        Err(DecodeError::TooLong)
    } else if !compact.len().is_multiple_of(2)
        || !compact.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        Err(DecodeError::InvalidHex)
    } else {
        let bytes: Vec<_> = compact
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        parse(&bytes)
    };
    report(result, "offline_protocol_replay")
}
