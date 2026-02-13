use std::collections::HashMap;
use std::net::TcpStream;

use crate::libs::db::langs::postgres::lib::error::PgError;
use crate::libs::db::langs::postgres::lib::protocol;
use crate::libs::db::langs::postgres::lib::types::Row;

pub struct Connection {
    stream: TcpStream,
    params: HashMap<String, String>,
    backend_pid: i32,
    backend_key: i32,
}

impl Connection {
    /// Connect to a Postgres server and complete authentication.
    pub fn connect(
        host: &str,
        port: u16,
        user: &str,
        database: &str,
        password: &str,
    ) -> Result<Self, PgError> {
        let mut stream = TcpStream::connect((host, port))?;

        protocol::write_startup(&mut stream, user, database)?;

        let mut params = HashMap::new();
        let mut backend_pid = 0i32;
        let mut backend_key = 0i32;

        loop {
            let (tag, payload) = protocol::read_message(&mut stream)?;
            match tag {
                b'R' => {
                    if payload.len() < 4 {
                        return Err(PgError::Protocol("truncated auth message".into()));
                    }
                    let auth_type = i32::from_be_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                    ]);
                    match auth_type {
                        0 => {
                            // AuthenticationOk
                        }
                        3 => {
                            // CleartextPassword
                            protocol::write_password(&mut stream, password)?;
                        }
                        5 => {
                            // MD5Password — salt is 4 bytes after auth type
                            if payload.len() < 8 {
                                return Err(PgError::Protocol(
                                    "truncated MD5 salt".into(),
                                ));
                            }
                            let salt: [u8; 4] =
                                [payload[4], payload[5], payload[6], payload[7]];
                            let hashed = protocol::md5_password(user, password, &salt);
                            protocol::write_password(&mut stream, &hashed)?;
                        }
                        10 => {
                            return Err(PgError::Auth(
                                "SASL authentication not supported".into(),
                            ));
                        }
                        _ => {
                            return Err(PgError::Auth(format!(
                                "unsupported auth type: {auth_type}"
                            )));
                        }
                    }
                }
                b'K' => {
                    // BackendKeyData
                    if payload.len() >= 8 {
                        backend_pid = i32::from_be_bytes([
                            payload[0], payload[1], payload[2], payload[3],
                        ]);
                        backend_key = i32::from_be_bytes([
                            payload[4], payload[5], payload[6], payload[7],
                        ]);
                    }
                }
                b'S' => {
                    // ParameterStatus
                    let mut offset = 0;
                    if let (Ok(key), Ok(val)) = (
                        read_cstring_from(&payload, &mut offset),
                        read_cstring_from(&payload, &mut offset),
                    ) {
                        params.insert(key, val);
                    }
                }
                b'Z' => {
                    // ReadyForQuery — connection established
                    break;
                }
                b'E' => {
                    return Err(protocol::parse_error_response(&payload));
                }
                b'N' => {
                    // NoticeResponse — ignore during startup
                }
                _ => {
                    return Err(PgError::Protocol(format!(
                        "unexpected message during startup: 0x{tag:02x}"
                    )));
                }
            }
        }

        Ok(Connection {
            stream,
            params,
            backend_pid,
            backend_key,
        })
    }

    /// Execute a simple query and return result rows.
    pub fn query(&mut self, sql: &str) -> Result<Vec<Row>, PgError> {
        protocol::write_query(&mut self.stream, sql)?;

        let mut columns = Vec::new();
        let mut rows = Vec::new();

        loop {
            let (tag, payload) = protocol::read_message(&mut self.stream)?;
            match tag {
                b'T' => {
                    columns = protocol::parse_row_description(&payload)?;
                }
                b'D' => {
                    let row = protocol::parse_data_row(&payload, &columns)?;
                    rows.push(row);
                }
                b'C' => {
                    // CommandComplete — query done
                }
                b'Z' => {
                    // ReadyForQuery
                    break;
                }
                b'E' => {
                    // Drain until ReadyForQuery then return error
                    let err = protocol::parse_error_response(&payload);
                    self.drain_until_ready()?;
                    return Err(err);
                }
                b'N' => {
                    // NoticeResponse — ignore
                }
                b'I' => {
                    // EmptyQueryResponse
                }
                _ => {
                    return Err(PgError::Protocol(format!(
                        "unexpected message in query: 0x{tag:02x}"
                    )));
                }
            }
        }

        Ok(rows)
    }

    /// Execute a statement that doesn't return rows (INSERT, UPDATE, DELETE, DDL).
    /// Returns the number of affected rows.
    pub fn execute(&mut self, sql: &str) -> Result<u64, PgError> {
        protocol::write_query(&mut self.stream, sql)?;

        let mut affected = 0u64;

        loop {
            let (tag, payload) = protocol::read_message(&mut self.stream)?;
            match tag {
                b'T' | b'D' => {}
                b'C' => {
                    affected = protocol::parse_command_complete(&payload);
                }
                b'Z' => {
                    break;
                }
                b'E' => {
                    let err = protocol::parse_error_response(&payload);
                    self.drain_until_ready()?;
                    return Err(err);
                }
                b'N' | b'I' => {}
                _ => {
                    return Err(PgError::Protocol(format!(
                        "unexpected message in execute: 0x{tag:02x}"
                    )));
                }
            }
        }

        Ok(affected)
    }

    /// Execute a parameterized query using the extended query protocol.
    pub fn query_params(&mut self, sql: &str, params: &[&str]) -> Result<Vec<Row>, PgError> {
        let stmt = "";
        let portal = "";

        // Send Parse, Bind, Describe, Execute, Sync
        protocol::write_parse(&mut self.stream, stmt, sql, &[])?;
        protocol::write_bind(&mut self.stream, portal, stmt, params)?;
        protocol::write_describe_portal(&mut self.stream, portal)?;
        protocol::write_execute(&mut self.stream, portal, 0)?;
        protocol::write_sync(&mut self.stream)?;

        let mut columns = Vec::new();
        let mut rows = Vec::new();

        loop {
            let (tag, payload) = protocol::read_message(&mut self.stream)?;
            match tag {
                b'1' => {
                    // ParseComplete
                }
                b'2' => {
                    // BindComplete
                }
                b'T' => {
                    columns = protocol::parse_row_description(&payload)?;
                }
                b'D' => {
                    let row = protocol::parse_data_row(&payload, &columns)?;
                    rows.push(row);
                }
                b'C' => {
                    // CommandComplete
                }
                b'Z' => {
                    break;
                }
                b'n' => {
                    // NoData (for queries returning no columns)
                }
                b'E' => {
                    let err = protocol::parse_error_response(&payload);
                    self.drain_until_ready()?;
                    return Err(err);
                }
                b'N' => {}
                _ => {
                    return Err(PgError::Protocol(format!(
                        "unexpected message in query_params: 0x{tag:02x}"
                    )));
                }
            }
        }

        Ok(rows)
    }

    /// Send Terminate and close the connection.
    pub fn close(mut self) -> Result<(), PgError> {
        protocol::write_terminate(&mut self.stream)?;
        Ok(())
    }

    /// Retrieve a server parameter received during startup (e.g., "server_version").
    pub fn server_param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Backend process ID.
    pub fn backend_pid(&self) -> i32 {
        self.backend_pid
    }

    /// Backend secret key (for cancel requests).
    pub fn backend_key(&self) -> i32 {
        self.backend_key
    }

    /// Drain messages until ReadyForQuery, used after receiving an error.
    fn drain_until_ready(&mut self) -> Result<(), PgError> {
        loop {
            let (tag, _) = protocol::read_message(&mut self.stream)?;
            if tag == b'Z' {
                return Ok(());
            }
        }
    }
}

/// Read a null-terminated string from a byte slice.
fn read_cstring_from(data: &[u8], offset: &mut usize) -> Result<String, PgError> {
    let start = *offset;
    while *offset < data.len() && data[*offset] != 0 {
        *offset += 1;
    }
    if *offset >= data.len() {
        return Err(PgError::Protocol("unterminated string".into()));
    }
    let s = String::from_utf8_lossy(&data[start..*offset]).into_owned();
    *offset += 1;
    Ok(s)
}
