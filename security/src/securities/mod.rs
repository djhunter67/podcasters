pub mod passworder;

/// This module tests the encryption, salt, and peppering of passwords
const PEPPER: [u8; 12] = *b"the_pepperer";
