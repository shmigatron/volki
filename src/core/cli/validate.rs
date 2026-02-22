use super::error::CliError;

pub fn validate_identifier(value: &str, label: &str) -> Result<(), CliError> {
    if value.is_empty() {
        return Err(CliError::InvalidUsage(crate::vformat!(
            "{label} must not be empty"
        )));
    }
    if !value.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(CliError::InvalidUsage(crate::vformat!(
            "{label} must contain only alphanumeric characters and underscores"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_name() {
        assert!(validate_identifier("my_db", "name").is_ok());
    }

    #[test]
    fn valid_alphanumeric() {
        assert!(validate_identifier("test123", "name").is_ok());
    }

    #[test]
    fn rejects_empty() {
        let err = validate_identifier("", "database name").unwrap_err();
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("must not be empty"));
    }

    #[test]
    fn rejects_special_chars() {
        let err = validate_identifier("my-db", "name").unwrap_err();
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("alphanumeric"));
    }

    #[test]
    fn rejects_spaces() {
        assert!(validate_identifier("my db", "name").is_err());
    }

    #[test]
    fn rejects_semicolon() {
        assert!(validate_identifier("db; DROP TABLE", "name").is_err());
    }
}
