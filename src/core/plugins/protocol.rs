use crate::libs::lang::shared::license::parsers::json::{JsonValue, extract_top_level};

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
                buf.push_str(&n.to_string());
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

        let obj = JsonOut::Object(vec![
            ("version".to_string(), JsonOut::Int(1)),
            ("hook".to_string(), JsonOut::Str(self.hook.clone())),
            ("data".to_string(), self.data.clone()),
            ("plugin_options".to_string(), options),
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

    let status = map
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing 'status' field".to_string())?;

    match status {
        "ok" => {
            let data = map
                .get("data")
                .cloned()
                .unwrap_or(JsonValue::Null);
            Ok(PluginResponse::Ok { data })
        }
        "skip" => Ok(PluginResponse::Skip),
        "error" => {
            let message = map
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Ok(PluginResponse::Error { message })
        }
        other => Err(format!("unknown status: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_string() {
        let j = JsonOut::Str("hello".into());
        assert_eq!(j.serialize(), "\"hello\"");
    }

    #[test]
    fn serialize_string_with_escapes() {
        let j = JsonOut::Str("a\"b\\c\nd".into());
        assert_eq!(j.serialize(), "\"a\\\"b\\\\c\\nd\"");
    }

    #[test]
    fn serialize_int() {
        assert_eq!(JsonOut::Int(42).serialize(), "42");
        assert_eq!(JsonOut::Int(-1).serialize(), "-1");
    }

    #[test]
    fn serialize_bool() {
        assert_eq!(JsonOut::Bool(true).serialize(), "true");
        assert_eq!(JsonOut::Bool(false).serialize(), "false");
    }

    #[test]
    fn serialize_null() {
        assert_eq!(JsonOut::Null.serialize(), "null");
    }

    #[test]
    fn serialize_array() {
        let j = JsonOut::Array(vec![JsonOut::Int(1), JsonOut::Str("a".into())]);
        assert_eq!(j.serialize(), "[1,\"a\"]");
    }

    #[test]
    fn serialize_object() {
        let j = JsonOut::Object(vec![
            ("key".into(), JsonOut::Str("val".into())),
        ]);
        assert_eq!(j.serialize(), "{\"key\":\"val\"}");
    }

    #[test]
    fn serialize_nested() {
        let j = JsonOut::Object(vec![
            ("arr".into(), JsonOut::Array(vec![JsonOut::Bool(true)])),
        ]);
        assert_eq!(j.serialize(), "{\"arr\":[true]}");
    }

    #[test]
    fn request_to_json() {
        let req = PluginRequest {
            hook: "formatter.before_all".into(),
            data: JsonOut::Object(vec![("tokens".into(), JsonOut::Array(vec![]))]),
            plugin_options: vec![("key".into(), "val".into())],
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
            PluginResponse::Error { message } => assert_eq!(message, "bad stuff"),
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
