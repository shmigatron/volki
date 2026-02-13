/// Known Postgres type OIDs for text-format conversion.
const OID_BOOL: u32 = 16;
const OID_BYTEA: u32 = 17;
const OID_INT8: u32 = 20;
const OID_INT2: u32 = 21;
const OID_INT4: u32 = 23;
const OID_FLOAT4: u32 = 700;
const OID_FLOAT8: u32 = 701;
const OID_TEXT: u32 = 25;
const OID_VARCHAR: u32 = 1043;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
}

impl Value {
    /// Convert a text-format Postgres value to a typed `Value` based on OID.
    pub fn from_text(text: &str, type_oid: u32) -> Value {
        match type_oid {
            OID_BOOL => match text {
                "t" | "true" | "TRUE" | "1" => Value::Bool(true),
                _ => Value::Bool(false),
            },
            OID_INT2 | OID_INT4 | OID_INT8 => match text.parse::<i64>() {
                Ok(n) => Value::Int(n),
                Err(_) => Value::Text(text.to_string()),
            },
            OID_FLOAT4 | OID_FLOAT8 => match text.parse::<f64>() {
                Ok(n) => Value::Float(n),
                Err(_) => Value::Text(text.to_string()),
            },
            OID_BYTEA => Value::Bytes(decode_bytea_hex(text)),
            OID_TEXT | OID_VARCHAR => Value::Text(text.to_string()),
            _ => Value::Text(text.to_string()),
        }
    }
}

/// Decode Postgres hex-format bytea (`\x...`) into bytes.
fn decode_bytea_hex(text: &str) -> Vec<u8> {
    let hex = if text.starts_with("\\x") {
        &text[2..]
    } else {
        text
    };
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let hi = hex_nibble(bytes[i]);
        let lo = hex_nibble(bytes[i + 1]);
        out.push((hi << 4) | lo);
        i += 2;
    }
    out
}

fn hex_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_oid: u32,
}

#[derive(Debug, Clone)]
pub struct Row {
    columns: Vec<Column>,
    values: Vec<Value>,
}

impl Row {
    pub fn new(columns: Vec<Column>, values: Vec<Value>) -> Self {
        Self { columns, values }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn get_value(&self, idx: usize) -> Option<&Value> {
        self.values.get(idx)
    }

    pub fn get_str(&self, idx: usize) -> Option<&str> {
        match self.values.get(idx)? {
            Value::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_int(&self, idx: usize) -> Option<i64> {
        match self.values.get(idx)? {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn get_float(&self, idx: usize) -> Option<f64> {
        match self.values.get(idx)? {
            Value::Float(n) => Some(*n),
            _ => None,
        }
    }

    pub fn get_bool(&self, idx: usize) -> Option<bool> {
        match self.values.get(idx)? {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Look up a value by column name, returning a reference.
    pub fn get_by_name(&self, name: &str) -> Option<&Value> {
        let idx = self.columns.iter().position(|c| c.name == name)?;
        self.values.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_from_text_bool() {
        assert_eq!(Value::from_text("t", OID_BOOL), Value::Bool(true));
        assert_eq!(Value::from_text("true", OID_BOOL), Value::Bool(true));
        assert_eq!(Value::from_text("f", OID_BOOL), Value::Bool(false));
        assert_eq!(Value::from_text("false", OID_BOOL), Value::Bool(false));
    }

    #[test]
    fn value_from_text_int() {
        assert_eq!(Value::from_text("42", OID_INT4), Value::Int(42));
        assert_eq!(Value::from_text("-100", OID_INT8), Value::Int(-100));
        assert_eq!(Value::from_text("0", OID_INT2), Value::Int(0));
    }

    #[test]
    fn value_from_text_int_invalid() {
        assert_eq!(
            Value::from_text("abc", OID_INT4),
            Value::Text("abc".into())
        );
    }

    #[test]
    fn value_from_text_float() {
        assert_eq!(Value::from_text("3.14", OID_FLOAT4), Value::Float(3.14));
        assert_eq!(Value::from_text("-1.5", OID_FLOAT8), Value::Float(-1.5));
    }

    #[test]
    fn value_from_text_string() {
        assert_eq!(
            Value::from_text("hello", OID_TEXT),
            Value::Text("hello".into())
        );
        assert_eq!(
            Value::from_text("world", OID_VARCHAR),
            Value::Text("world".into())
        );
    }

    #[test]
    fn value_from_text_bytea() {
        assert_eq!(
            Value::from_text("\\xDEAD", OID_BYTEA),
            Value::Bytes(vec![0xDE, 0xAD])
        );
    }

    #[test]
    fn value_from_text_unknown_oid() {
        assert_eq!(
            Value::from_text("whatever", 9999),
            Value::Text("whatever".into())
        );
    }

    #[test]
    fn row_accessors() {
        let cols = vec![
            Column {
                name: "id".into(),
                type_oid: OID_INT4,
            },
            Column {
                name: "name".into(),
                type_oid: OID_TEXT,
            },
            Column {
                name: "active".into(),
                type_oid: OID_BOOL,
            },
        ];
        let vals = vec![
            Value::Int(1),
            Value::Text("alice".into()),
            Value::Bool(true),
        ];
        let row = Row::new(cols, vals);

        assert_eq!(row.len(), 3);
        assert!(!row.is_empty());
        assert_eq!(row.get_int(0), Some(1));
        assert_eq!(row.get_str(1), Some("alice"));
        assert_eq!(row.get_bool(2), Some(true));
    }

    #[test]
    fn row_get_by_name() {
        let cols = vec![
            Column {
                name: "x".into(),
                type_oid: OID_INT4,
            },
            Column {
                name: "y".into(),
                type_oid: OID_TEXT,
            },
        ];
        let vals = vec![Value::Int(10), Value::Text("hi".into())];
        let row = Row::new(cols, vals);

        assert_eq!(row.get_by_name("x"), Some(&Value::Int(10)));
        assert_eq!(row.get_by_name("y"), Some(&Value::Text("hi".into())));
        assert_eq!(row.get_by_name("z"), None);
    }

    #[test]
    fn row_out_of_bounds() {
        let row = Row::new(vec![], vec![]);
        assert!(row.is_empty());
        assert_eq!(row.get_value(0), None);
        assert_eq!(row.get_str(0), None);
        assert_eq!(row.get_int(0), None);
        assert_eq!(row.get_bool(0), None);
        assert_eq!(row.get_by_name("nope"), None);
    }

    #[test]
    fn row_type_mismatch_returns_none() {
        let cols = vec![Column {
            name: "val".into(),
            type_oid: OID_INT4,
        }];
        let vals = vec![Value::Int(42)];
        let row = Row::new(cols, vals);

        assert_eq!(row.get_str(0), None);
        assert_eq!(row.get_bool(0), None);
        assert_eq!(row.get_int(0), Some(42));
    }

    #[test]
    fn decode_bytea_hex_basic() {
        assert_eq!(decode_bytea_hex("\\xCAFE"), vec![0xCA, 0xFE]);
        assert_eq!(decode_bytea_hex("\\x00FF"), vec![0x00, 0xFF]);
        assert_eq!(decode_bytea_hex("\\x"), vec![]);
    }
}
