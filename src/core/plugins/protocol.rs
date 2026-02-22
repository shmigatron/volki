use crate::core::volkiwithstds::collections::json::{JsonValue, extract_top_level};
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::vvec;

#[derive(Debug, Clone)]
pub enum JsonOut {
    Str(String),
    Int(i64),
    Bool(bool),
    Null,
    Array(Vec<JsonOut>),
    Object(Vec<(String, JsonOut)>),
}

impl JsonOut {
    pub fn serialize(&self) -> String {
        let mut buf = String::new();
        self.write_to(&mut buf);
        buf
    }

    fn write_to(&self, buf: &mut String) {
        match self {
            JsonOut::Str(s) => {
                buf.push('"');
                for ch in s.chars() {
                    match ch {
                        '"' => buf.push_str("\\\""),
                        '\\' => buf.push_str("\\\\"),
                        '\n' => buf.push_str("\\n"),
                        '\r' => buf.push_str("\\r"),
                        '\t' => buf.push_str("\\t"),
                        c => buf.push(c),
                    }
                }
                buf.push('"');
            }
            JsonOut::Int(n) => {
                let s = crate::vformat!("{n}");
                buf.push_str(&s);
            }
            JsonOut::Bool(b) => {
                buf.push_str(if *b { "true" } else { "false" });
            }
            JsonOut::Null => {
                buf.push_str("null");
            }
            JsonOut::Array(items) => {
                buf.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        buf.push(',');
                    }
                    item.write_to(buf);
                }
                buf.push(']');
            }
            JsonOut::Object(entries) => {
                buf.push('{');
                for (i, (key, val)) in entries.iter().enumerate() {
                    if i > 0 {
                        buf.push(',');
                    }
                    buf.push('"');
                    buf.push_str(key);
                    buf.push_str("\":");
                    val.write_to(buf);
                }
                buf.push('}');
            }
        }
    }
}

pub struct PluginRequest {
    pub hook: String,
    pub data: JsonOut,
    pub plugin_options: Vec<(String, String)>,
}

impl PluginRequest {
    pub fn to_json(&self) -> String {
        let options = JsonOut::Object(
            self.plugin_options
                .iter()
                .map(|(k, v)| (k.clone(), JsonOut::Str(v.clone())))
                .collect(),
        );

        let obj = JsonOut::Object(vvec![
            (String::from("version"), JsonOut::Int(1)),
            (String::from("hook"), JsonOut::Str(self.hook.clone())),
            (String::from("data"), self.data.clone()),
            (String::from("plugin_options"), options),
        ]);

        obj.serialize()
    }
}

#[derive(Debug)]
pub enum PluginResponse {
    Ok { data: JsonValue },
    Skip,
    Error { message: String },
}

pub fn parse_response(raw: &str) -> Result<PluginResponse, String> {
    let map = extract_top_level(raw);
    let status_key = String::from("status");
    let data_key = String::from("data");
    let error_key = String::from("error");

    let status = map
        .get(&status_key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| String::from("missing 'status' field"))?;

    match status {
        "ok" => {
            let data = map.get(&data_key).cloned().unwrap_or(JsonValue::Null);
            Ok(PluginResponse::Ok { data })
        }
        "skip" => Ok(PluginResponse::Skip),
        "error" => {
            let message = String::from(
                map.get(&error_key)
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error"),
            );
            Ok(PluginResponse::Error { message })
        }
        other => Err(crate::vformat!("unknown status: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_string() {
        let j = JsonOut::Str(String::from("hello"));
        assert_eq!(j.serialize().as_str(), "\"hello\"");
    }

    #[test]
    fn serialize_string_with_escapes() {
        let j = JsonOut::Str(String::from("a\"b\\c\nd"));
        assert_eq!(j.serialize().as_str(), "\"a\\\"b\\\\c\\nd\"");
    }

    #[test]
    fn serialize_int() {
        assert_eq!(JsonOut::Int(42).serialize().as_str(), "42");
        assert_eq!(JsonOut::Int(-1).serialize().as_str(), "-1");
    }

    #[test]
    fn serialize_bool() {
        assert_eq!(JsonOut::Bool(true).serialize().as_str(), "true");
        assert_eq!(JsonOut::Bool(false).serialize().as_str(), "false");
    }

    #[test]
    fn serialize_null() {
        assert_eq!(JsonOut::Null.serialize().as_str(), "null");
    }

    #[test]
    fn serialize_array() {
        let j = JsonOut::Array(vvec![JsonOut::Int(1), JsonOut::Str(String::from("a"))]);
        assert_eq!(j.serialize().as_str(), "[1,\"a\"]");
    }

    #[test]
    fn serialize_object() {
        let j = JsonOut::Object(vvec![(
            String::from("key"),
            JsonOut::Str(String::from("val"))
        ),]);
        assert_eq!(j.serialize().as_str(), "{\"key\":\"val\"}");
    }

    #[test]
    fn serialize_nested() {
        let j = JsonOut::Object(vvec![(
            String::from("arr"),
            JsonOut::Array(vvec![JsonOut::Bool(true)])
        ),]);
        assert_eq!(j.serialize().as_str(), "{\"arr\":[true]}");
    }

    #[test]
    fn request_to_json() {
        let req = PluginRequest {
            hook: String::from("formatter.before_all"),
            data: JsonOut::Object(vvec![(String::from("tokens"), JsonOut::Array(Vec::new()))]),
            plugin_options: vvec![(String::from("key"), String::from("val"))],
        };
        let json = req.to_json();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"hook\":\"formatter.before_all\""));
        assert!(json.contains("\"plugin_options\":{\"key\":\"val\"}"));
    }

    #[test]
    fn parse_ok_response() {
        let raw = r#"{"version":1,"status":"ok","data":{"tokens":[]}}"#;
        let resp = parse_response(raw).unwrap();
        assert!(matches!(resp, PluginResponse::Ok { .. }));
    }

    #[test]
    fn parse_skip_response() {
        let raw = r#"{"version":1,"status":"skip"}"#;
        let resp = parse_response(raw).unwrap();
        assert!(matches!(resp, PluginResponse::Skip));
    }

    #[test]
    fn parse_error_response() {
        let raw = r#"{"version":1,"status":"error","error":"bad stuff"}"#;
        let resp = parse_response(raw).unwrap();
        match resp {
            PluginResponse::Error { message } => assert_eq!(message.as_str(), "bad stuff"),
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn parse_missing_status() {
        let raw = r#"{"version":1}"#;
        assert!(parse_response(raw).is_err());
    }

    #[test]
    fn parse_unknown_status() {
        let raw = r#"{"version":1,"status":"wat"}"#;
        assert!(parse_response(raw).is_err());
    }
}
