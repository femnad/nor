use ::notify_rust::{Hint, Notification, Urgency};
use crate::NodeType;

const NOTIFICATION_SUMMARY: &str = "nor";

fn get_volume_classifier(volume: f32, node_type: &NodeType) -> String {
    let vol = if volume == 0.0 {
        "muted"
    } else if volume <= 0.3 {
        "low"
    } else if volume <= 0.6 {
        "medium"
    } else if volume <= 1.0 {
        "high"
    } else {
        if let NodeType::Sink = node_type {
            "overamplified"
        } else {
            "high"
        }
    };
    vol.to_string()
}

fn get_icon(volume: f32, node_type: &NodeType) -> String {
    let prefix = match node_type {
        NodeType::Sink => {"audio-volume"}
        NodeType::Source => {"microphone-sensitivity"}
    };
    let classifier = get_volume_classifier(volume, node_type);
    format!("{prefix}-{classifier}-symbolic")
}

pub fn message(msg: &str) {
    Notification::new()
        .summary(NOTIFICATION_SUMMARY)
        .body(msg)
        .show()
        .expect("error sending notification");
}

pub fn volume(volume: f32, node_name: String, node_type: &NodeType) {
    let icon = get_icon(volume, node_type);
    let vol_scaled = volume * 100.0;
    let vol_int = vol_scaled.round() as u32;

    Notification::new()
        .appname("volume")
        .urgency(Urgency::Low)
        .hint(Hint::CustomInt("value".to_string(), vol_int as i32))
        .icon(icon.as_str())
        .summary(node_name.as_str())
        .show()
        .expect("error sending notification");
}
