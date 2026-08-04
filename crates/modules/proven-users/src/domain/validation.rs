//! Pure validation helpers shared by the application services (ADR-0006). No I/O.

use super::error::UsersError;

/// Minimal, dependency-free email shape check: `local@domain.tld`.
pub fn validate_email(email: &str) -> Result<(), UsersError> {
    let mut parts = email.splitn(2, '@');
    let (local, domain) = match (parts.next(), parts.next()) {
        (Some(local), Some(domain)) => (local, domain),
        _ => return Err(UsersError::validation(format!("invalid email: {email}"))),
    };

    let domain_valid = !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.split('.').all(|segment| !segment.is_empty());

    if !local.is_empty() && domain_valid {
        Ok(())
    } else {
        Err(UsersError::validation(format!("invalid email: {email}")))
    }
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), UsersError> {
    if value.trim().is_empty() {
        Err(UsersError::validation(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

/// `HH:MM` 24-hour time-of-day, used for notification quiet hours.
pub fn validate_hhmm(field: &'static str, value: &str) -> Result<(), UsersError> {
    let valid = value
        .split_once(':')
        .map(|(h, m)| {
            h.len() == 2
                && m.len() == 2
                && h.chars().all(|c| c.is_ascii_digit())
                && m.chars().all(|c| c.is_ascii_digit())
                && h.parse::<u8>().is_ok_and(|v| v < 24)
                && m.parse::<u8>().is_ok_and(|v| v < 60)
        })
        .unwrap_or(false);

    if valid {
        Ok(())
    } else {
        Err(UsersError::validation(format!(
            "{field} must be HH:MM (24h), got: {value}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_validation() {
        assert!(validate_email("worker@acme.test").is_ok());
        assert!(validate_email("not-an-email").is_err());
        assert!(validate_email("missing-domain@").is_err());
        assert!(validate_email("@missing-local.com").is_err());
    }

    #[test]
    fn hhmm_validation() {
        assert!(validate_hhmm("quiet_hours_start", "22:00").is_ok());
        assert!(validate_hhmm("quiet_hours_start", "6:00").is_err());
        assert!(validate_hhmm("quiet_hours_start", "25:00").is_err());
        assert!(validate_hhmm("quiet_hours_start", "not-a-time").is_err());
    }
}
