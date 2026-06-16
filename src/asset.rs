use std::fmt;

/// Represents the types of Stellar assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    /// The native asset (XLM).
    Native,
    /// An alphanumeric asset code of 1 to 4 characters.
    Alphanumeric4,
    /// An alphanumeric asset code of 5 to 12 characters.
    Alphanumeric12,
}

/// Errors that can occur during Stellar asset code validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetError {
    /// The asset code has an invalid length (must be between 1 and 12 characters).
    InvalidLength,
    /// The asset code contains invalid characters (must be alphanumeric).
    InvalidCharacters,
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::InvalidLength => {
                write!(f, "Asset code must be between 1 and 12 characters long")
            }
            AssetError::InvalidCharacters => {
                write!(f, "Asset code must consist only of alphanumeric characters")
            }
        }
    }
}

impl std::error::Error for AssetError {}

/// Validates a Stellar asset code and returns its type.
///
/// # Validation Rules
///
/// - The code must be between 1 and 12 characters long.
/// - The code must consist only of alphanumeric ASCII characters (`a-z`, `A-Z`, `0-9`).
/// - Codes matching `"XLM"` or `"native"` (case-insensitive) are classified as `AssetType::Native`.
/// - Other valid codes of 1 to 4 characters are classified as `AssetType::Alphanumeric4`.
/// - Other valid codes of 5 to 12 characters are classified as `AssetType::Alphanumeric12`.
///
/// # Examples
///
/// ```
/// use soroban_toolkit::asset::{validate_asset_code, AssetType, AssetError};
///
/// // Valid Native Asset Code
/// assert_eq!(validate_asset_code("XLM"), Ok(AssetType::Native));
/// assert_eq!(validate_asset_code("native"), Ok(AssetType::Native));
///
/// // Valid Alphanumeric 4 Asset Code
/// assert_eq!(validate_asset_code("USD"), Ok(AssetType::Alphanumeric4));
///
/// // Valid Alphanumeric 12 Asset Code
/// assert_eq!(validate_asset_code("USDCENTERP"), Ok(AssetType::Alphanumeric12));
///
/// // Invalid Length (Empty)
/// assert_eq!(validate_asset_code(""), Err(AssetError::InvalidLength));
///
/// // Invalid Characters
/// assert_eq!(validate_asset_code("USD$"), Err(AssetError::InvalidCharacters));
/// ```
pub fn validate_asset_code(code: &str) -> Result<AssetType, AssetError> {
    if code.is_empty() || code.len() > 12 {
        return Err(AssetError::InvalidLength);
    }

    if !code.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AssetError::InvalidCharacters);
    }

    let upper = code.to_uppercase();
    if upper == "XLM" || upper == "NATIVE" {
        Ok(AssetType::Native)
    } else if code.len() <= 4 {
        Ok(AssetType::Alphanumeric4)
    } else {
        Ok(AssetType::Alphanumeric12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_native() {
        assert_eq!(validate_asset_code("XLM"), Ok(AssetType::Native));
        assert_eq!(validate_asset_code("xlm"), Ok(AssetType::Native));
        assert_eq!(validate_asset_code("native"), Ok(AssetType::Native));
        assert_eq!(validate_asset_code("NATIVE"), Ok(AssetType::Native));
    }

    #[test]
    fn test_validate_alphanumeric4() {
        assert_eq!(validate_asset_code("A"), Ok(AssetType::Alphanumeric4));
        assert_eq!(validate_asset_code("USD"), Ok(AssetType::Alphanumeric4));
        assert_eq!(validate_asset_code("BTC"), Ok(AssetType::Alphanumeric4));
        assert_eq!(validate_asset_code("USDT"), Ok(AssetType::Alphanumeric4));
        assert_eq!(validate_asset_code("1234"), Ok(AssetType::Alphanumeric4));
    }

    #[test]
    fn test_validate_alphanumeric12() {
        assert_eq!(validate_asset_code("USDCE"), Ok(AssetType::Alphanumeric12));
        assert_eq!(
            validate_asset_code("USDCENTERPR"),
            Ok(AssetType::Alphanumeric12)
        );
        assert_eq!(
            validate_asset_code("USDCENTERPR1"),
            Ok(AssetType::Alphanumeric12)
        );
    }

    #[test]
    fn test_validate_invalid_length() {
        // Empty string
        assert_eq!(validate_asset_code(""), Err(AssetError::InvalidLength));
        // Too long (13 characters)
        assert_eq!(
            validate_asset_code("USDCENTERPR12"),
            Err(AssetError::InvalidLength)
        );
    }

    #[test]
    fn test_validate_invalid_characters() {
        assert_eq!(
            validate_asset_code("USD$"),
            Err(AssetError::InvalidCharacters)
        );
        assert_eq!(
            validate_asset_code("EUR-"),
            Err(AssetError::InvalidCharacters)
        );
        assert_eq!(
            validate_asset_code("US D"),
            Err(AssetError::InvalidCharacters)
        );
    }
}
