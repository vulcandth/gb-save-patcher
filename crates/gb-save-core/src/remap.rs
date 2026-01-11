use crate::{Address, SaveBinary, SaveResult};

#[allow(clippy::too_many_arguments)]
/// Copies set bits from one bitset to another using an index mapping.
///
/// This is useful for remapping flag arrays where the meaning of individual bits changes between
/// versions.
///
/// - `map_index` maps a source bit index to a destination bit index.
/// - `on_unmapped` is called for any source bit that cannot be mapped or would fall out of range.
///
/// # Errors
/// Returns an error if the source/destination bit addresses are out of bounds.
pub fn map_bitset(
    source: &SaveBinary,
    src_base: Address,
    src_bits: usize,
    dest: &mut SaveBinary,
    dst_base: Address,
    dst_bits: usize,
    mut map_index: impl FnMut(usize) -> Option<usize>,
    mut on_unmapped: impl FnMut(usize),
) -> SaveResult<()> {
    for src_index in 0..src_bits {
        if !source.read_indexed_bit(src_base, src_index)? {
            continue;
        }

        let Some(dst_index) = map_index(src_index) else {
            on_unmapped(src_index);
            continue;
        };

        if dst_index >= dst_bits {
            on_unmapped(src_index);
            continue;
        }

        dest.write_indexed_bit(dst_base, dst_index, true)?;
    }

    Ok(())
}

/// Remaps a zero-terminated list of `u8` values in-place.
///
/// Iteration stops at the first `0` byte (or after `max_len` bytes). Values that cannot be mapped
/// are left unchanged and reported via `on_invalid`.
///
/// # Errors
/// Returns an error if any accessed bytes are out of bounds.
pub fn remap_zero_terminated_u8(
    save: &mut SaveBinary,
    base: Address,
    max_len: usize,
    mut map_value: impl FnMut(u8) -> Option<u8>,
    mut on_invalid: impl FnMut(usize, u8),
) -> SaveResult<()> {
    for index in 0..max_len {
        let addr = Address(base.0 + index as u32);
        let value = save.read_u8(addr)?;
        if value == 0 {
            break;
        }

        let Some(mapped) = map_value(value) else {
            on_invalid(index, value);
            continue;
        };

        if mapped != value {
            save.write_u8(addr, mapped)?;
        }
    }

    Ok(())
}

/// Remaps a fixed-length list of `u8` values in-place, skipping zeros.
///
/// - If a value is `0`, it is left as-is.
/// - If `map_value` returns `None`, `on_invalid` decides a replacement value.
///
/// # Errors
/// Returns an error if any accessed bytes are out of bounds.
pub fn remap_fixed_len_u8_skip_zero(
    save: &mut SaveBinary,
    base: Address,
    len: usize,
    mut map_value: impl FnMut(u8) -> Option<u8>,
    mut on_invalid: impl FnMut(usize, u8) -> u8,
) -> SaveResult<()> {
    for index in 0..len {
        let addr = Address(base.0 + index as u32);
        let value = save.read_u8(addr)?;
        if value == 0 {
            continue;
        }

        let Some(mapped) = map_value(value) else {
            let replacement = on_invalid(index, value);
            if replacement != value {
                save.write_u8(addr, replacement)?;
            }
            continue;
        };

        if mapped != value {
            save.write_u8(addr, mapped)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Size;

    #[test]
    fn remap_zero_terminated_stops_on_zero_and_writes() {
        let mut save = SaveBinary::new(vec![1, 2, 0, 3, 0]);
        let mut invalid: Vec<(usize, u8)> = Vec::new();

        remap_zero_terminated_u8(
            &mut save,
            Address(0),
            5,
            |v| Some(v + 10),
            |i, v| invalid.push((i, v)),
        )
        .unwrap();

        assert_eq!(save.as_bytes(), &[11, 12, 0, 3, 0]);
        assert!(invalid.is_empty());
    }

    #[test]
    fn map_bitset_maps_set_bits_only() {
        let mut src = SaveBinary::new(vec![0u8; 2]);
        let mut dst = SaveBinary::new(vec![0u8; 2]);
        dst.clear_len(Address(0), Size(2)).unwrap();

        src.write_indexed_bit(Address(0), 0, true).unwrap();
        src.write_indexed_bit(Address(0), 9, true).unwrap();

        let mut unmapped: Vec<usize> = Vec::new();
        map_bitset(
            &src,
            Address(0),
            16,
            &mut dst,
            Address(0),
            16,
            |i| Some(i + 1),
            |i| unmapped.push(i),
        )
        .unwrap();

        assert!(unmapped.is_empty());
        assert!(dst.read_indexed_bit(Address(0), 1).unwrap());
        assert!(dst.read_indexed_bit(Address(0), 10).unwrap());
        assert!(!dst.read_indexed_bit(Address(0), 0).unwrap());
    }

    #[test]
    fn remap_fixed_len_skip_zero_keeps_zeros_and_replaces_invalid() {
        let mut save = SaveBinary::new(vec![0, 1, 2, 3]);
        let mut invalid: Vec<(usize, u8)> = Vec::new();

        remap_fixed_len_u8_skip_zero(
            &mut save,
            Address(0),
            4,
            |v| (v != 2).then_some(v + 10),
            |i, v| {
                invalid.push((i, v));
                0
            },
        )
        .unwrap();

        assert_eq!(save.as_bytes(), &[0, 11, 0, 13]);
        assert_eq!(invalid, vec![(2, 2)]);
    }
}
