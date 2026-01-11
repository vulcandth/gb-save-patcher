use gb_save_core::{Address, SaveBinary, SaveResult};

/// Absolute offset (within the save buffer) of the save version field.
///
/// Real games typically store this somewhere in SRAM; this example keeps it at the start of the
/// buffer for simplicity.
pub const SAVE_VERSION_ABS_ADDRESS: u32 = 0;

/// Minimum byte length required for a valid save buffer in this example.
pub const MIN_SAVE_SIZE: usize = 2;

/// Reads the current save version.
///
/// # Errors
/// Returns an error if the save is too small.
pub fn get_save_version(save: &SaveBinary) -> SaveResult<u16> {
    save.require_min_size(MIN_SAVE_SIZE)?;
    save.read_u16_le(Address(SAVE_VERSION_ABS_ADDRESS))
}
