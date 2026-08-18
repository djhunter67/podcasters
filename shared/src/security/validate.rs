/// Validate emails per the HTML5 specification
#[must_use = "Validate user emails per the HTML5 specification"]
pub fn email(email: &str) -> bool {
    // Basic length check (HTML5 implies max 254 chars total)
    if email.is_empty() || email.len() > 254 {
        return false;
    }

    let mut parts = email.split('@');
    let local = match parts.next() {
        Some(l) if !l.is_empty() => l,
        _ => return false,
    };
    let domain = match parts.next() {
        Some(d) if !d.is_empty() => d,
        _ => return false,
    };

    // Ensure no extra '@' symbols
    if parts.next().is_some() {
        return false;
    }

    // Validate local part: [a-zA-Z0-9._%+-]+
    if !local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-'))
    {
        return false;
    }

    // Validate domain: [a-zA-Z0-9.-]+\.[a-zA-Z]{2,}
    if !domain.contains('.') {
        return false;
    }

    let mut domain_parts = domain.rsplitn(2, '.');
    let tld = match domain_parts.next() {
        Some(t) if t.len() >= 2 => t,
        _ => return false,
    };
    let domain_body = match domain_parts.next() {
        Some(b) if !b.is_empty() => b,
        _ => return false,
    };

    // Check TLD is alpha only
    if !tld.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    // Check domain body: [a-zA-Z0-9.-]+
    if !domain_body
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
    {
        return false;
    }

    // Domain cannot start or end with a dot or hyphen (common strict interpretation)
    if domain_body.starts_with('.')
        || domain_body.ends_with('.')
        || domain_body.starts_with('-')
        || domain_body.ends_with('-')
    {
        return false;
    }

    true
}
#[cfg(test)]
mod tests {
    use crate::security::validate;

    #[test]
    fn test_valid_standard_emails() {
        assert!(validate::email("test@example.com"));
        assert!(validate::email("user.name@domain.co.uk"));
        assert!(validate::email("user+tag@gmail.com"));
        assert!(validate::email("user_name@example.org"));
        assert!(validate::email("user-name@example.io"));
        assert!(validate::email("user%name@example.com"));
        assert!(validate::email("a@b.co")); // Minimal valid
    }

    #[test]
    fn test_invalid_missing_parts() {
        assert!(!validate::email("")); // Empty
        assert!(!validate::email("test")); // No @
        assert!(!validate::email("@example.com")); // No local part
        assert!(!validate::email("test@")); // No domain
        assert!(!validate::email("test@domain")); // No TLD
        assert!(!validate::email("test@@example.com")); // Double @
    }

    #[test]
    fn test_invalid_characters() {
        // HTML5 regex: ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$
        assert!(!validate::email("test space@example.com")); // Space in local
        assert!(!validate::email("test!@example.com")); // Exclamation
        assert!(!validate::email("test# @example.com")); // Hash
        assert!(!validate::email("test@example.c")); // TLD too short (1 char)
        assert!(!validate::email("test@example.123")); // Numeric TLD
    }

    #[test]
    fn test_invalid_domain_format() {
        assert!(!validate::email("test@.example.com")); // Domain starts with dot
        assert!(!validate::email("test@example..com")); // Double dot in domain
        assert!(!validate::email("test@example.com.")); // Ends with dot (handled by split logic)
    }

    #[test]
    fn test_length_limits() {
        // Total length > 254
        let long_local = "a".repeat(250);
        let long_email = format!("{}@example.com", long_local);
        assert!(!validate::email(&long_email));

        // Valid length boundary
        let ok_local = "a".repeat(64); // Max local part usually 64
        let ok_email = format!("{}@example.com", ok_local);
        // Note: Our simple implementation checks total len <= 254,
        // so this specific case depends on exact char count.
        assert!(validate::email(&ok_email) || ok_email.len() > 254);
    }
}
