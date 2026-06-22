#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GboxError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    InvalidBase64(#[from] base64::DecodeError),

    #[cfg(feature = "schema")]
    #[error("Schema validation error: {0:?}")]
    Validation(String),

    #[cfg(feature = "openssl")]
    #[error(transparent)]
    OpenSsl(#[from] openssl::error::ErrorStack),

    #[error(transparent)]
    InvalidUrl(url::ParseError),

    #[error("Decryption failed: {0:?}")]
    DecryptionFailed(rncryptor::v3::errors::Error),

    #[error("Encryption failed: {0:?}")]
    EncryptionFailed(rncryptor::v3::errors::Error),

    #[error("Provided datetime ({got}) is invalid. Expected to get {expected}")]
    InvalidDateTimeFormat { got: String, expected: String },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InstallableAppError {
    #[error("Input path is invalid.")]
    InvalidInputString,

    #[error("Missing field: {0}")]
    MissingField(&'static str),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AltstoreError {
    #[error("{0} is not a valid hex color.")]
    InvalidHexColor(String),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConversionError {
    #[error("This item in unsupported in selected conversion.")]
    UnsupportedItemType,

    #[error("URL is missed or invalid for this conversion.")]
    InvalidOrMissingUrl,

    #[error("Version asset is missed.")]
    MissingVersionAsset,
}
