use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::io::{self, Read, Write};

use crate::libs::db::langs::postgres::lib::error::PgError;
use crate::libs::db::langs::postgres::lib::types::{Column, Row, Value};
// --- MD5 implementation (RFC 1321) ---

const S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

pub fn md5_digest(data: &[u8]) -> [u8; 16] {
    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }

        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | (!d)), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(K[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(S[i]));
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&a0.to_le_bytes());
    result[4..8].copy_from_slice(&b0.to_le_bytes());
    result[8..12].copy_from_slice(&c0.to_le_bytes());
    result[12..16].copy_from_slice(&d0.to_le_bytes());
    result
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Compute Postgres MD5 password: `"md5" + hex(md5(hex(md5(password + user)) + salt))`
pub fn md5_password(user: &str, password: &str, salt: &[u8; 4]) -> String {
    // inner = md5(password + user)
    let mut inner_input = Vec::with_capacity(password.len() + user.len());
    inner_input.extend_from_slice(password.as_bytes());
    inner_input.extend_from_slice(user.as_bytes());
    let inner = md5_digest(&inner_input);
    let inner_hex = hex_encode(&inner);

    // outer = md5(inner_hex + salt)
    let mut outer_input = Vec::with_capacity(inner_hex.len() + 4);
    outer_input.extend_from_slice(inner_hex.as_bytes());
    outer_input.extend_from_slice(salt);
    let outer = md5_digest(&outer_input);

    crate::vformat!("md5{}", hex_encode(&outer))
}

// --- Wire protocol helpers ---

fn write_i32(buf: &mut Vec<u8>, val: i32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_i16(buf: &mut Vec<u8>, val: i16) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_cstring(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
}

fn read_i32(data: &[u8], offset: &mut usize) -> Result<i32, PgError> {
    if *offset + 4 > data.len() {
        return Err(PgError::Protocol("truncated i32".into()));
    }
    let val = i32::from_be_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    Ok(val)
}

fn read_i16(data: &[u8], offset: &mut usize) -> Result<i16, PgError> {
    if *offset + 2 > data.len() {
        return Err(PgError::Protocol("truncated i16".into()));
    }
    let val = i16::from_be_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    Ok(val)
}

fn read_cstring(data: &[u8], offset: &mut usize) -> Result<String, PgError> {
    let start = *offset;
    while *offset < data.len() && data[*offset] != 0 {
        *offset += 1;
    }
    if *offset >= data.len() {
        return Err(PgError::Protocol("unterminated string".into()));
    }
    let s = String::from_utf8_lossy(&data[start..*offset]).to_owned();
    *offset += 1;
    Ok(s)
}

/// Send Postgres StartupMessage (no tag byte).
pub fn write_startup<W: Write>(stream: &mut W, user: &str, database: &str) -> io::Result<()> {
    let mut body = Vec::new();
    write_i32(&mut body, 196608); // version 3.0 = 0x00030000
    write_cstring(&mut body, "user");
    write_cstring(&mut body, user);
    write_cstring(&mut body, "database");
    write_cstring(&mut body, database);
    body.push(0);

    let len = (body.len() as i32) + 4; // +4 for length field itself
    let mut msg = Vec::with_capacity(4 + body.len());
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)?;
    stream.flush()
}

/// Read one Postgres message: returns (tag, payload).
pub fn read_message<R: Read>(stream: &mut R) -> Result<(u8, Vec<u8>), PgError> {
    let mut tag_buf = [0u8; 1];
    stream.read_exact(&mut tag_buf)?;
    let tag = tag_buf[0];

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = i32::from_be_bytes(len_buf);
    if len < 4 {
        return Err(PgError::Protocol(crate::vformat!(
            "invalid message length: {len}"
        )));
    }

    let payload_len = (len - 4) as usize;
    let mut payload = Vec::with_capacity(payload_len);
    for _ in 0..payload_len {
        payload.push(0);
    }
    if payload_len > 0 {
        stream.read_exact(&mut payload)?;
    }

    Ok((tag, payload))
}

/// Send a cleartext PasswordMessage.
pub fn write_password<W: Write>(stream: &mut W, password: &str) -> io::Result<()> {
    let mut body = Vec::new();
    write_cstring(&mut body, password);

    let len = (body.len() as i32) + 4;
    let mut msg = Vec::with_capacity(1 + 4 + body.len());
    msg.push(b'p');
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)?;
    stream.flush()
}

/// Send a simple Query message.
pub fn write_query<W: Write>(stream: &mut W, sql: &str) -> io::Result<()> {
    let mut body = Vec::new();
    write_cstring(&mut body, sql);

    let len = (body.len() as i32) + 4;
    let mut msg = Vec::with_capacity(1 + 4 + body.len());
    msg.push(b'Q');
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)?;
    stream.flush()
}

/// Send a Terminate message.
pub fn write_terminate<W: Write>(stream: &mut W) -> io::Result<()> {
    let msg: [u8; 5] = [b'X', 0, 0, 0, 4];
    stream.write_all(&msg)?;
    stream.flush()
}

// --- Extended query protocol ---

/// Send Parse message: named statement with SQL containing $1, $2 placeholders.
pub fn write_parse<W: Write>(
    stream: &mut W,
    stmt_name: &str,
    sql: &str,
    param_types: &[u32],
) -> io::Result<()> {
    let mut body = Vec::new();
    write_cstring(&mut body, stmt_name);
    write_cstring(&mut body, sql);
    write_i16(&mut body, param_types.len() as i16);
    for &oid in param_types {
        write_i32(&mut body, oid as i32);
    }

    let len = (body.len() as i32) + 4;
    let mut msg = Vec::with_capacity(1 + 4 + body.len());
    msg.push(b'P');
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)
}

/// Send Bind message: bind parameters to a portal.
pub fn write_bind<W: Write>(
    stream: &mut W,
    portal: &str,
    stmt_name: &str,
    params: &[&str],
) -> io::Result<()> {
    let mut body = Vec::new();
    write_cstring(&mut body, portal);
    write_cstring(&mut body, stmt_name);

    write_i16(&mut body, 0);
    write_i16(&mut body, params.len() as i16);
    for &p in params {
        let bytes = p.as_bytes();
        write_i32(&mut body, bytes.len() as i32);
        body.extend_from_slice(bytes);
    }

    write_i16(&mut body, 0);

    let len = (body.len() as i32) + 4;
    let mut msg = Vec::with_capacity(1 + 4 + body.len());
    msg.push(b'B');
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)
}

/// Send Describe message for a portal.
pub fn write_describe_portal<W: Write>(stream: &mut W, portal: &str) -> io::Result<()> {
    let mut body = Vec::new();
    body.push(b'P'); // 'P' for portal
    write_cstring(&mut body, portal);

    let len = (body.len() as i32) + 4;
    let mut msg = Vec::with_capacity(1 + 4 + body.len());
    msg.push(b'D');
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)
}

/// Send Execute message for a portal.
pub fn write_execute<W: Write>(stream: &mut W, portal: &str, max_rows: i32) -> io::Result<()> {
    let mut body = Vec::new();
    write_cstring(&mut body, portal);
    write_i32(&mut body, max_rows);

    let len = (body.len() as i32) + 4;
    let mut msg = Vec::with_capacity(1 + 4 + body.len());
    msg.push(b'E');
    msg.extend_from_slice(&len.to_be_bytes());
    msg.extend_from_slice(&body);
    stream.write_all(&msg)
}

/// Send Sync message to signal end of extended query cycle.
pub fn write_sync<W: Write>(stream: &mut W) -> io::Result<()> {
    let msg: [u8; 5] = [b'S', 0, 0, 0, 4];
    stream.write_all(&msg)?;
    stream.flush()
}

// --- Response parsing ---

/// Parse RowDescription payload into column metadata.
pub fn parse_row_description(data: &[u8]) -> Result<Vec<Column>, PgError> {
    let mut offset = 0;
    let num_fields = read_i16(data, &mut offset)? as usize;
    let mut columns = Vec::with_capacity(num_fields);

    for _ in 0..num_fields {
        let name = read_cstring(data, &mut offset)?;
        let _table_oid = read_i32(data, &mut offset)?;
        let _col_attr = read_i16(data, &mut offset)?;
        let type_oid = read_i32(data, &mut offset)? as u32;
        let _type_size = read_i16(data, &mut offset)?;
        let _type_mod = read_i32(data, &mut offset)?;
        let _format_code = read_i16(data, &mut offset)?;
        columns.push(Column { name, type_oid });
    }

    Ok(columns)
}

/// Parse DataRow payload into a `Row`.
pub fn parse_data_row(data: &[u8], columns: &[Column]) -> Result<Row, PgError> {
    let mut offset = 0;
    let num_cols = read_i16(data, &mut offset)? as usize;
    let mut values = Vec::with_capacity(num_cols);

    for i in 0..num_cols {
        let len = read_i32(data, &mut offset)?;
        if len == -1 {
            values.push(Value::Null);
        } else {
            let len = len as usize;
            if offset + len > data.len() {
                return Err(PgError::Protocol("truncated data row".into()));
            }
            let text = String::from_utf8_lossy(&data[offset..offset + len]).to_owned();
            offset += len;

            let type_oid = columns.get(i).map(|c| c.type_oid).unwrap_or(0);
            values.push(Value::from_text(&text, type_oid));
        }
    }

    let cols: Vec<Column> = columns.iter().cloned().collect();
    Ok(Row::new(cols, values))
}

/// Parse ErrorResponse/NoticeResponse payload into field map.
/// Fields are identified by single-byte codes: S=severity, C=code, M=message, etc.
pub fn parse_error_response(data: &[u8]) -> PgError {
    let mut severity = String::new();
    let mut code = String::new();
    let mut message = String::new();
    let mut offset = 0;

    while offset < data.len() {
        let field_type = data[offset];
        offset += 1;
        if field_type == 0 {
            break;
        }
        match read_cstring(data, &mut offset) {
            Ok(val) => match field_type {
                b'S' | b'V' => {
                    if severity.is_empty() {
                        severity = val;
                    }
                }
                b'C' => code = val,
                b'M' => message = val,
                _ => {}
            },
            Err(_) => break,
        }
    }

    PgError::Server {
        severity,
        code,
        message,
    }
}

/// Parse CommandComplete tag to extract affected row count.
pub fn parse_command_complete(data: &[u8]) -> u64 {
    let tag = String::from_utf8_lossy(data);
    let tag = tag.trim_end_matches('\0');
    // Format: "INSERT 0 5", "UPDATE 3", "DELETE 1", "SELECT 10", etc.
    tag.rsplit(' ')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vvec;

    // --- MD5 tests (RFC 1321 Appendix A.5) ---

    #[test]
    fn md5_empty() {
        let digest = md5_digest(b"");
        assert_eq!(hex_encode(&digest), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn md5_a() {
        let digest = md5_digest(b"a");
        assert_eq!(hex_encode(&digest), "0cc175b9c0f1b6a831c399e269772661");
    }

    #[test]
    fn md5_abc() {
        let digest = md5_digest(b"abc");
        assert_eq!(hex_encode(&digest), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn md5_message_digest() {
        let digest = md5_digest(b"message digest");
        assert_eq!(hex_encode(&digest), "f96b697d7cb7938d525a2f31aaf161d0");
    }

    #[test]
    fn md5_alphabet() {
        let digest = md5_digest(b"abcdefghijklmnopqrstuvwxyz");
        assert_eq!(hex_encode(&digest), "c3fcd3d76192e4007dfb496cca67e13b");
    }

    #[test]
    fn md5_alphanumeric() {
        let digest = md5_digest(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
        assert_eq!(hex_encode(&digest), "d174ab98d277d9f5a5611c2c9f419d9f");
    }

    #[test]
    fn md5_numeric() {
        let digest = md5_digest(
            b"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
        );
        assert_eq!(hex_encode(&digest), "57edf4a22be3c955ac49da2e2107b67a");
    }

    // --- MD5 password test ---

    #[test]
    fn md5_password_known() {
        // Known test: user="user", password="password", salt=[0x01,0x02,0x03,0x04]
        let result = md5_password("user", "password", &[0x01, 0x02, 0x03, 0x04]);
        assert!(result.starts_with("md5"));
        assert_eq!(result.len(), 35); // "md5" + 32 hex chars
    }

    // --- Startup message ---

    #[test]
    fn startup_message_encoding() {
        let mut buf = Vec::new();
        write_startup(&mut buf, "testuser", "testdb").unwrap();

        // First 4 bytes: length (big-endian i32)
        let len = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        assert_eq!(len as usize, buf.len());

        // Next 4 bytes: version 3.0 = 196608
        let version = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        assert_eq!(version, 196608);

        // Should contain "user\0testuser\0database\0testdb\0\0"
        let body = &buf[8..];
        assert!(body.starts_with(b"user\0testuser\0database\0testdb\0"));
        assert_eq!(*body.last().unwrap(), 0);
    }

    // --- Frame reading/writing ---

    #[test]
    fn read_message_roundtrip() {
        // Build a fake message: tag='Z', payload=[b'I']
        let mut wire = Vec::new();
        wire.push(b'Z');
        let len: i32 = 5; // 4 + 1 byte payload
        wire.extend_from_slice(&len.to_be_bytes());
        wire.push(b'I');

        let mut cursor = io::Cursor::new(wire);
        let (tag, payload) = read_message(&mut cursor).unwrap();
        assert_eq!(tag, b'Z');
        assert_eq!(payload, vvec![b'I']);
    }

    #[test]
    fn read_message_empty_payload() {
        let mut wire = Vec::new();
        wire.push(b'X');
        let len: i32 = 4; // no payload
        wire.extend_from_slice(&len.to_be_bytes());

        let mut cursor = io::Cursor::new(wire);
        let (tag, payload) = read_message(&mut cursor).unwrap();
        assert_eq!(tag, b'X');
        assert!(payload.is_empty());
    }

    // --- RowDescription parsing ---

    #[test]
    fn parse_row_description_basic() {
        let mut data = Vec::new();
        // 2 fields
        data.extend_from_slice(&2i16.to_be_bytes());

        // Field 1: "id", table_oid=0, col_attr=0, type_oid=23 (int4), size=4, mod=-1, format=0
        data.extend_from_slice(b"id\0");
        data.extend_from_slice(&0i32.to_be_bytes()); // table oid
        data.extend_from_slice(&0i16.to_be_bytes()); // column attr
        data.extend_from_slice(&23i32.to_be_bytes()); // type oid (int4)
        data.extend_from_slice(&4i16.to_be_bytes()); // type size
        data.extend_from_slice(&(-1i32).to_be_bytes()); // type mod
        data.extend_from_slice(&0i16.to_be_bytes()); // format code

        // Field 2: "name", type_oid=25 (text)
        data.extend_from_slice(b"name\0");
        data.extend_from_slice(&0i32.to_be_bytes());
        data.extend_from_slice(&0i16.to_be_bytes());
        data.extend_from_slice(&25i32.to_be_bytes());
        data.extend_from_slice(&(-1i16).to_be_bytes());
        data.extend_from_slice(&(-1i32).to_be_bytes());
        data.extend_from_slice(&0i16.to_be_bytes());

        let cols = parse_row_description(&data).unwrap();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].name, "id");
        assert_eq!(cols[0].type_oid, 23);
        assert_eq!(cols[1].name, "name");
        assert_eq!(cols[1].type_oid, 25);
    }

    // --- DataRow parsing ---

    #[test]
    fn parse_data_row_basic() {
        let columns = vvec![
            Column {
                name: "id".into(),
                type_oid: 23,
            },
            Column {
                name: "name".into(),
                type_oid: 25,
            },
        ];

        let mut data = Vec::new();
        data.extend_from_slice(&2i16.to_be_bytes()); // 2 columns

        // Column 1: "42"
        let val1 = b"42";
        data.extend_from_slice(&(val1.len() as i32).to_be_bytes());
        data.extend_from_slice(val1);

        // Column 2: "alice"
        let val2 = b"alice";
        data.extend_from_slice(&(val2.len() as i32).to_be_bytes());
        data.extend_from_slice(val2);

        let row = parse_data_row(&data, &columns).unwrap();
        assert_eq!(row.get_int(0), Some(42));
        assert_eq!(row.get_str(1), Some("alice"));
    }

    #[test]
    fn parse_data_row_with_null() {
        let columns = vvec![Column {
            name: "val".into(),
            type_oid: 25,
        }];

        let mut data = Vec::new();
        data.extend_from_slice(&1i16.to_be_bytes());
        data.extend_from_slice(&(-1i32).to_be_bytes()); // NULL

        let row = parse_data_row(&data, &columns).unwrap();
        assert_eq!(row.get_value(0), Some(&Value::Null));
    }

    // --- ErrorResponse parsing ---

    #[test]
    fn parse_error_response_basic() {
        let mut data = Vec::new();
        data.push(b'S');
        data.extend_from_slice(b"ERROR\0");
        data.push(b'C');
        data.extend_from_slice(b"42P01\0");
        data.push(b'M');
        data.extend_from_slice(b"relation \"foo\" does not exist\0");
        data.push(0); // terminator

        let err = parse_error_response(&data);
        match err {
            PgError::Server {
                severity,
                code,
                message,
            } => {
                assert_eq!(severity, "ERROR");
                assert_eq!(code, "42P01");
                assert_eq!(message, "relation \"foo\" does not exist");
            }
            _ => panic!("expected PgError::Server"),
        }
    }

    // --- CommandComplete parsing ---

    #[test]
    fn parse_command_complete_insert() {
        assert_eq!(parse_command_complete(b"INSERT 0 5\0"), 5);
    }

    #[test]
    fn parse_command_complete_update() {
        assert_eq!(parse_command_complete(b"UPDATE 3\0"), 3);
    }

    #[test]
    fn parse_command_complete_select() {
        assert_eq!(parse_command_complete(b"SELECT 10\0"), 10);
    }

    #[test]
    fn parse_command_complete_delete() {
        assert_eq!(parse_command_complete(b"DELETE 0\0"), 0);
    }

    // --- Password message ---

    #[test]
    fn write_password_message() {
        let mut buf = Vec::new();
        write_password(&mut buf, "secret").unwrap();
        assert_eq!(buf[0], b'p');
        let len = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(len as usize, buf.len() - 1); // tag not included in length
        // Body should contain "secret\0"
        assert_eq!(&buf[5..], b"secret\0");
    }

    // --- Query message ---

    #[test]
    fn write_query_message() {
        let mut buf = Vec::new();
        write_query(&mut buf, "SELECT 1").unwrap();
        assert_eq!(buf[0], b'Q');
        let len = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(len as usize, buf.len() - 1);
        assert_eq!(&buf[5..], b"SELECT 1\0");
    }

    // --- Terminate message ---

    #[test]
    fn write_terminate_message() {
        let mut buf = Vec::new();
        write_terminate(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), &[b'X', 0, 0, 0, 4]);
    }
}
