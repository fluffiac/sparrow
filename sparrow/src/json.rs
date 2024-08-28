//! JSON definitions for memory reader events.

pub const CONNECTED: &str = r#"{"event": "connected"}"#;
pub const DISCONNECTED: &str = r#"{"event": "disconnected"}"#;

pub fn pace(data: &Vec<f32>) -> String {
    format!(r#"{{"event": "pace", "data": {:?}}}"#, data)
}

pub fn debug<T: std::fmt::Debug>(data: &T) -> String {
    format!(r#"{{"event": "debug_event", "data": {:?}}}"#, data)
}
