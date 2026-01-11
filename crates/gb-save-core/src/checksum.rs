use crate::{AddressRange, SaveBinary, SaveError, SaveResult};

/// Calculates the additive checksum of a save byte range.
///
/// This matches the common "sum of bytes (wrapping u16)" scheme used by many saves.
///
/// # Errors
/// Returns an error if `range` is invalid or falls outside the save buffer.
pub fn calculate_additive_u16_checksum(save: &SaveBinary, range: AddressRange) -> SaveResult<u16> {
    if range.start.0 >= range.end.0 {
        return Err(SaveError::InvalidAddressRange { range });
    }

    let bytes = save.slice(range)?;
    Ok(bytes
        .iter()
        .fold(0u16, |acc, b| acc.wrapping_add(*b as u16)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Address, AddressRange};

    #[test]
    fn wraps_like_u16() {
        let save = SaveBinary::new(vec![0xFF; 4]);
        let checksum =
            calculate_additive_u16_checksum(&save, AddressRange::new(Address(0), Address(4)))
                .unwrap();
        assert_eq!(checksum, 0x03FC);
    }
}
