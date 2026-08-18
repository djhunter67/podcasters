pub mod passworder;
pub mod session;
pub mod validate;

/// This module tests the encryption, salt, and peppering of passwords
const PEPPER: [u8; 12] = *b"the_pepperer";

// Tests for the PassWorder struct
#[cfg(test)]
mod tests {}
