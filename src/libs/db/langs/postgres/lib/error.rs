use std::fmt;
use std::io;

#[derive(Debug)]
pub enum PgError {
    Io(io::Error),
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

impl std::error::Error for PgError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PgError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for PgError {
    fn from(e: io::Error) -> Self {
        PgError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let err = PgError::Io(io::Error::new(io::ErrorKind::ConnectionRefused, "refused"));
        assert!(err.to_string().contains("I/O error"));
        assert!(err.to_string().contains("refused"));
    }

    #[test]
    fn display_auth_error() {
        let err = PgError::Auth("bad password".into());
        assert_eq!(err.to_string(), "authentication error: bad password");
    }

    #[test]
    fn display_protocol_error() {
        let err = PgError::Protocol("unexpected tag".into());
        assert_eq!(err.to_string(), "protocol error: unexpected tag");
    }

    #[test]
    fn display_server_error() {
        let err = PgError::Server {
            severity: "ERROR".into(),
            code: "42P01".into(),
            message: "relation does not exist".into(),
        };
        assert_eq!(
            err.to_string(),
            "server error (ERROR 42P01): relation does not exist"
        );
    }

    #[test]
    fn from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe");
        let pg_err: PgError = io_err.into();
        assert!(matches!(pg_err, PgError::Io(_)));
    }

    #[test]
    fn error_source() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let pg_err = PgError::Io(io_err);
        assert!(std::error::Error::source(&pg_err).is_some());

        let auth_err = PgError::Auth("x".into());
        assert!(std::error::Error::source(&auth_err).is_none());
    }
}
