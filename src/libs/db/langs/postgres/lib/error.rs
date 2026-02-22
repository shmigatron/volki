use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::fmt;
use crate::core::volkiwithstds::io;

#[derive(Debug)]
pub enum PgError {
    Io(io::IoError),
    Auth(String),
    Protocol(String),
    Server {
        code: String,
        message: String,
        severity: String,
    },
}

impl fmt::Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgError::Io(e) => write!(f, "I/O error: {e}"),
            PgError::Auth(msg) => write!(f, "authentication error: {msg}"),
            PgError::Protocol(msg) => write!(f, "protocol error: {msg}"),
            PgError::Server {
                severity,
                code,
                message,
            } => write!(f, "server error ({severity} {code}): {message}"),
        }
    }
}

impl From<io::IoError> for PgError {
    fn from(e: io::IoError) -> Self {
        PgError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let err = PgError::Io(io::IoError::new(
            io::IoErrorKind::ConnectionRefused,
            "refused",
        ));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("refused"));
    }

    #[test]
    fn display_auth_error() {
        let err = PgError::Auth("bad password".into());
        assert_eq!(
            crate::vformat!("{err}").as_str(),
            "authentication error: bad password"
        );
    }

    #[test]
    fn display_protocol_error() {
        let err = PgError::Protocol("unexpected tag".into());
        assert_eq!(
            crate::vformat!("{err}").as_str(),
            "protocol error: unexpected tag"
        );
    }

    #[test]
    fn display_server_error() {
        let err = PgError::Server {
            severity: "ERROR".into(),
            code: "42P01".into(),
            message: "relation does not exist".into(),
        };
        assert_eq!(
            crate::vformat!("{err}").as_str(),
            "server error (ERROR 42P01): relation does not exist"
        );
    }

    #[test]
    fn from_io_error() {
        let io_err = io::IoError::new(io::IoErrorKind::BrokenPipe, "pipe");
        let pg_err: PgError = io_err.into();
        assert!(matches!(pg_err, PgError::Io(_)));
    }
}
