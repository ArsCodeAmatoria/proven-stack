//! Pure validation helpers shared by the application services (ADR-0005). No I/O.

use super::error::CompaniesError;

/// Minimal, dependency-free email shape check: `local@domain.tld`.
pub fn validate_email(email: &str) -> Result<(), CompaniesError> {
    let mut parts = email.splitn(2, '@');
    let (local, domain) = match (parts.next(), parts.next()) {
        (Some(local), Some(domain)) => (local, domain),
        _ => return Err(CompaniesError::validation(format!("invalid email: {email}"))),
    };

    let domain_valid = !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.split('.').all(|segment| !segment.is_empty());

    if !local.is_empty() && domain_valid {
        Ok(())
    } else {
        Err(CompaniesError::validation(format!("invalid email: {email}")))
    }
}

/// ISO 3166-1 alpha-2 country codes must be exactly 2 (ASCII alphabetic) characters.
pub fn validate_country_code(country_code: &str) -> Result<(), CompaniesError> {
    if country_code.len() == 2 && country_code.chars().all(|c| c.is_ascii_alphabetic()) {
        Ok(())
    } else {
        Err(CompaniesError::validation(format!(
            "country_code must be exactly 2 letters, got: {country_code}"
        )))
    }
}

/// `#RRGGBB` hex color.
pub fn validate_hex_color(color: &str) -> Result<(), CompaniesError> {
    let bytes = color.as_bytes();
    let valid = bytes.len() == 7
        && bytes[0] == b'#'
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(CompaniesError::validation(format!(
            "color must be a #RRGGBB hex value, got: {color}"
        )))
    }
}

/// ISO 4217 currency codes must be exactly 3 (ASCII alphabetic) characters, e.g. `CAD`.
pub fn validate_currency_code(currency_code: &str) -> Result<(), CompaniesError> {
    if currency_code.len() == 3 && currency_code.chars().all(|c| c.is_ascii_alphabetic()) {
        Ok(())
    } else {
        Err(CompaniesError::validation(format!(
            "currency_code must be exactly 3 letters, got: {currency_code}"
        )))
    }
}

pub fn validate_max_upload_bytes(max_upload_bytes: i64) -> Result<(), CompaniesError> {
    if max_upload_bytes > 0 {
        Ok(())
    } else {
        Err(CompaniesError::validation(
            "max_upload_bytes must be greater than 0",
        ))
    }
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CompaniesError> {
    if value.trim().is_empty() {
        Err(CompaniesError::validation(format!(
            "{field} must not be empty"
        )))
    } else {
        Ok(())
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
    fn country_code_validation() {
        assert!(validate_country_code("CA").is_ok());
        assert!(validate_country_code("USA").is_err());
        assert!(validate_country_code("1").is_err());
    }

    #[test]
    fn hex_color_validation() {
        assert!(validate_hex_color("#1A2b3C").is_ok());
        assert!(validate_hex_color("1A2B3C").is_err());
        assert!(validate_hex_color("#1A2B3").is_err());
    }
}
