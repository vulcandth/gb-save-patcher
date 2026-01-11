use gb_save_core::{SaveError, SaveResult, SymbolDatabase};

/// The set of save versions this example game supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedSaveVersion {
    V1,
    V2,
    V3,
}

impl SupportedSaveVersion {
    #[must_use]
    pub fn as_u16(self) -> u16 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
            Self::V3 => 3,
        }
    }
}

/// Converts a raw `u16` into a supported version.
///
/// # Errors
/// Returns an error if `version` is not supported.
pub fn supported_version_from_u16(version: u16) -> SaveResult<SupportedSaveVersion> {
    match version {
        1 => Ok(SupportedSaveVersion::V1),
        2 => Ok(SupportedSaveVersion::V2),
        3 => Ok(SupportedSaveVersion::V3),
        _ => Err(SaveError::InvalidSaveState {
            reason: format!("unsupported save version {version}"),
        }),
    }
}

/// Returns the symbol database for `version`.
///
/// Real games generally load per-version `.sym` files (often embedded). This example returns an
/// empty database.
#[must_use]
pub fn symbols_for_version(_version: SupportedSaveVersion) -> SaveResult<SymbolDatabase> {
    Ok(SymbolDatabase::new())
}
