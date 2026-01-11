use crate::{Address, AddressRange, Size};

/// A typed result used throughout the save patching codebase.
pub type SaveResult<T> = Result<T, SaveError>;

/// Errors returned when reading, validating, or patching a save buffer.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    /// The provided save buffer is smaller than the minimum size required by the operation.
    #[error("save buffer too small: expected at least {min} bytes, got {actual}")]
    SaveTooSmall {
        /// The minimum required size in bytes.
        min: usize,
        /// The actual provided size in bytes.
        actual: usize,
    },

    /// An absolute address fell outside the save buffer.
    #[error("out of bounds: address {address:?} (len={len})")]
    AddressOutOfBounds {
        /// The address that was accessed.
        address: Address,
        /// The length of the save buffer in bytes.
        len: usize,
    },

    /// An address range fell outside the save buffer.
    #[error("out of bounds: range {range:?} (len={len})")]
    RangeOutOfBounds {
        /// The attempted address range.
        range: AddressRange,
        /// The length of the save buffer in bytes.
        len: usize,
    },

    /// A bit index was outside the valid range for a single byte.
    #[error("invalid bit index: {bit} (expected 0..=7)")]
    InvalidBitIndex {
        /// The invalid bit index.
        bit: u8,
    },

    /// A range was malformed (e.g. start >= end).
    #[error("invalid address range: {range:?}")]
    InvalidAddressRange {
        /// The invalid range.
        range: AddressRange,
    },

    /// A fixed-size read/write expected a different byte length.
    #[error("size mismatch: expected {expected} bytes, got {actual} bytes")]
    SizeMismatch {
        /// The expected size.
        expected: Size,
        /// The actual size.
        actual: Size,
    },

    /// A requested symbol name was not present in the symbol database.
    #[error("symbol not found: {name}")]
    SymbolNotFound {
        /// The missing symbol name.
        name: String,
    },

    /// The embedded or provided symbol data could not be decompressed.
    #[error("symbol file decompression failed")]
    SymbolFileDecompressionFailed,

    /// A symbol existed but its address was not in SRAM.
    #[error("symbol is not in SRAM: {name} (address=0x{address:04X})")]
    SymbolNotInSram {
        /// The symbol name.
        name: String,
        /// The raw (non-SRAM-absolute) address.
        address: u16,
    },

    /// A symbol existed but was not in the expected memory region.
    #[error("symbol is not in expected region {expected}: {name} (address=0x{address:04X})")]
    SymbolNotInExpectedRegion {
        /// The symbol name.
        name: String,
        /// The expected memory region label.
        expected: &'static str,
        /// The raw (non-SRAM-absolute) address.
        address: u16,
    },

    /// A symbol-relative address calculation went backwards (negative offset).
    #[error("symbol {symbol} is before base symbol {base}")]
    SymbolBeforeBase {
        /// The symbol that resolved to an earlier address.
        symbol: String,
        /// The base symbol used as the origin.
        base: String,
    },

    /// A migration was requested from a newer version to an older version.
    #[error("unsupported migration direction: {current_version} -> {target_version}")]
    UnsupportedMigrationDirection {
        /// The current save version.
        current_version: u16,
        /// The requested target save version.
        target_version: u16,
    },

    /// A migration plan could not be built because an intermediate step is missing.
    #[error("missing migration step from {from_version} to reach {target_version}")]
    MissingMigrationStep {
        /// The version we attempted to migrate from.
        from_version: u16,
        /// The requested target save version.
        target_version: u16,
    },

    /// A fix patch was requested with a `dev_type` that is not known.
    #[error("unknown fix patch: dev_type={dev_type}")]
    UnknownFixPatch {
        /// The requested fix patch identifier.
        dev_type: u8,
    },

    /// The requested feature exists conceptually but has not been implemented.
    #[error("not implemented: {feature}")]
    NotImplemented {
        /// A short human-readable description of the missing feature.
        feature: String,
    },

    /// A computed checksum does not match the value stored in the save.
    #[error("{which} checksum mismatch: stored=0x{stored:04X} calculated=0x{calculated:04X}")]
    ChecksumMismatch {
        /// Identifies which checksum was validated (e.g. "main" or "backup").
        which: &'static str,
        /// The checksum stored in the save.
        stored: u16,
        /// The checksum computed from the save data.
        calculated: u16,
    },

    /// The save is structurally valid but in a state that prevents safe patching.
    #[error("invalid save state: {reason}")]
    InvalidSaveState {
        /// A human-readable explanation of why patching is unsafe.
        reason: String,
    },
}
