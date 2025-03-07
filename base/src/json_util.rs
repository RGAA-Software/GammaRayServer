
pub fn get_string(data: &serde_json::Value, key: &str) -> String {
    data.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn get_string_or(data: &serde_json::Value, key: &str, default: &str) -> String {
    data.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

pub fn get_int(data: &serde_json::Value, key: &str) -> i64 {
    data.get(key)
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

pub fn get_int_or(data: &serde_json::Value, key: &str, default: i64) -> i64 {
    data.get(key)
        .and_then(|v| v.as_i64())
        .unwrap_or(default)
}