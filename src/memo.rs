use std::fmt;

/// Represents the different types of Stellar transaction memos.
///
/// Stellar memos are optional metadata fields attached to transactions.
/// Each type has specific validation constraints enforced by the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoType {
    /// No memo is attached to the transaction.
    None,
    /// A short text message (UTF-8 encoded string).
    Text(String),
    /// A 32-byte hash, typically represented as a hexadecimal string.
    Hash(String),
    /// A 64-bit unsigned integer.
    Id(u64),
    /// A 32-byte hash, typically representing a transaction ID being refunded.
    Return(String),
}

/// Errors that can occur during Stellar memo validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoError {
    /// The text memo exceeds the 28-byte limit.
    TextTooLong,
    /// The hash memo is not a valid 32-byte hash (must be 64 hex characters).
    InvalidHash,
    /// The return memo is not a valid 32-byte hash (must be 64 hex characters).
    InvalidReturn,
}

impl fmt::Display for MemoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoError::TextTooLong => write!(f, "Text memo exceeds the 28-byte limit"),
            MemoError::InvalidHash => {
                write!(f, "Hash memo must be a 64-character hexadecimal string representing a 32-byte hash")
            }
            MemoError::InvalidReturn => {
                write!(f, "Return memo must be a 64-character hexadecimal string representing a 32-byte hash")
            }
        }
    }
}

impl std::error::Error for MemoError {}

/// Validates a Stellar transaction memo based on its type.
///
/// # Validation Rules
///
/// - `MemoType::None` is always valid.
/// - `MemoType::Text` is valid if its UTF-8 representation is at most 28 bytes.
/// - `MemoType::Hash` is valid if it represents a 32-byte hash (64 hex characters, optionally prefixed with "0x").
/// - `MemoType::Id` is always valid as a 64-bit unsigned integer.
/// - `MemoType::Return` is valid if it represents a 32-byte hash (64 hex characters, optionally prefixed with "0x").
///
/// # Examples
///
/// ```
/// use soroban_toolkit::memo::{validate_memo, MemoType, MemoError};
///
/// // Valid Text Memo
/// let text_memo = MemoType::Text("Hello, Stellar!".to_string());
/// assert!(validate_memo(&text_memo).is_ok());
///
/// // Invalid Text Memo (over 28 bytes)
/// let long_text = MemoType::Text("This text is way too long for a Stellar memo!".to_string());
/// assert_eq!(validate_memo(&long_text), Err(MemoError::TextTooLong));
///
/// // Valid Hash Memo
/// let hash_memo = MemoType::Hash("a".repeat(64));
/// assert!(validate_memo(&hash_memo).is_ok());
///
/// // Invalid Hash Memo
/// let invalid_hash = MemoType::Hash("invalid_hash_string".to_string());
/// assert_eq!(validate_memo(&invalid_hash), Err(MemoError::InvalidHash));
/// ```
pub fn validate_memo(memo: &MemoType) -> Result<(), MemoError> {
    match memo {
        MemoType::None => Ok(()),
        MemoType::Text(text) => {
            if text.len() > 28 {
                Err(MemoError::TextTooLong)
            } else {
                Ok(())
            }
        }
        MemoType::Hash(hash) => {
            if is_valid_hash(hash) {
                Ok(())
            } else {
                Err(MemoError::InvalidHash)
            }
        }
        MemoType::Id(_) => Ok(()),
        MemoType::Return(ret) => {
            if is_valid_hash(ret) {
                Ok(())
            } else {
                Err(MemoError::InvalidReturn)
            }
        }
    }
}

/// Helper function to validate if a string represents a 32-byte hash.
/// Supports optional "0x" prefix.
fn is_valid_hash(hash: &str) -> bool {
    let clean_hash = hash.strip_prefix("0x").unwrap_or(hash);
    clean_hash.len() == 64 && clean_hash.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_none() {
        assert!(validate_memo(&MemoType::None).is_ok());
    }

    #[test]
    fn test_memo_text_valid() {
        assert!(validate_memo(&MemoType::Text("Hello World".to_string())).is_ok());
        assert!(validate_memo(&MemoType::Text("".to_string())).is_ok());

        // Exact 28 bytes limit with ASCII
        assert!(validate_memo(&MemoType::Text("a".repeat(28))).is_ok());

        // Exactly 28 bytes with multi-byte characters
        // '⚡' is 3 bytes in UTF-8. 9 * 3 = 27 bytes + 1 byte 'a' = 28 bytes.
        let multi_byte_28 = format!("{}a", "⚡".repeat(9));
        assert_eq!(multi_byte_28.len(), 28);
        assert!(validate_memo(&MemoType::Text(multi_byte_28)).is_ok());
    }

    #[test]
    fn test_memo_text_invalid() {
        // Exceeds 28 bytes (29 bytes)
        assert_eq!(
            validate_memo(&MemoType::Text("a".repeat(29))),
            Err(MemoError::TextTooLong)
        );

        // Exceeds 28 bytes due to multi-byte character representation
        // '⚡' is 3 bytes in UTF-8. 10 * 3 = 30 bytes (even though it is only 10 characters).
        let multi_byte_30 = "⚡".repeat(10);
        assert_eq!(multi_byte_30.len(), 30);
        assert_eq!(
            validate_memo(&MemoType::Text(multi_byte_30)),
            Err(MemoError::TextTooLong)
        );
    }

    #[test]
    fn test_memo_hash_valid() {
        // 64 hex characters (lowercase)
        assert!(validate_memo(&MemoType::Hash("a".repeat(64))).is_ok());
        // 64 hex characters (uppercase)
        assert!(validate_memo(&MemoType::Hash("A".repeat(64))).is_ok());
        // 64 hex characters (mixed case)
        assert!(validate_memo(&MemoType::Hash("1a2B3c4D".repeat(8))).is_ok());
        // With 0x prefix (66 characters)
        assert!(validate_memo(&MemoType::Hash(format!("0x{}", "f".repeat(64)))).is_ok());
    }

    #[test]
    fn test_memo_hash_invalid() {
        // Too short
        assert_eq!(
            validate_memo(&MemoType::Hash("a".repeat(63))),
            Err(MemoError::InvalidHash)
        );
        // Too long
        assert_eq!(
            validate_memo(&MemoType::Hash("a".repeat(65))),
            Err(MemoError::InvalidHash)
        );
        // Invalid characters
        assert_eq!(
            validate_memo(&MemoType::Hash("g".repeat(64))),
            Err(MemoError::InvalidHash)
        );
        // Prefix without remaining 64 hex chars
        assert_eq!(
            validate_memo(&MemoType::Hash("0x".to_string())),
            Err(MemoError::InvalidHash)
        );
        assert_eq!(
            validate_memo(&MemoType::Hash(format!("0x{}", "f".repeat(63)))),
            Err(MemoError::InvalidHash)
        );
    }

    #[test]
    fn test_memo_id_valid() {
        assert!(validate_memo(&MemoType::Id(0)).is_ok());
        assert!(validate_memo(&MemoType::Id(u64::MAX)).is_ok());
        assert!(validate_memo(&MemoType::Id(123456789)).is_ok());
    }

    #[test]
    fn test_memo_return_valid() {
        // 64 hex characters
        assert!(validate_memo(&MemoType::Return("b".repeat(64))).is_ok());
        // With 0x prefix
        assert!(validate_memo(&MemoType::Return(format!("0x{}", "e".repeat(64)))).is_ok());
    }

    #[test]
    fn test_memo_return_invalid() {
        // Too short
        assert_eq!(
            validate_memo(&MemoType::Return("b".repeat(63))),
            Err(MemoError::InvalidReturn)
        );
        // Too long
        assert_eq!(
            validate_memo(&MemoType::Return("b".repeat(65))),
            Err(MemoError::InvalidReturn)
        );
        // Invalid characters
        assert_eq!(
            validate_memo(&MemoType::Return("z".repeat(64))),
            Err(MemoError::InvalidReturn)
        );
    }
}
