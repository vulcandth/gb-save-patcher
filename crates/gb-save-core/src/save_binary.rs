use crate::{Address, AddressRange, SaveError, SaveResult, Size};

/// Mutable byte buffer with safe, bounds-checked helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBinary {
    bytes: Vec<u8>,
}

impl SaveBinary {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn require_min_size(&self, min: usize) -> SaveResult<()> {
        if self.bytes.len() < min {
            return Err(SaveError::SaveTooSmall {
                min,
                actual: self.bytes.len(),
            });
        }

        Ok(())
    }

    fn check_address(&self, address: Address) -> SaveResult<usize> {
        let index = address.as_usize();
        if index >= self.bytes.len() {
            return Err(SaveError::AddressOutOfBounds {
                address,
                len: self.bytes.len(),
            });
        }

        Ok(index)
    }

    fn check_range(&self, range: AddressRange) -> SaveResult<std::ops::Range<usize>> {
        let r = range.to_usize_range();
        if r.start > r.end || r.end > self.bytes.len() {
            return Err(SaveError::RangeOutOfBounds {
                range,
                len: self.bytes.len(),
            });
        }

        Ok(r)
    }

    pub fn read_u8(&self, address: Address) -> SaveResult<u8> {
        let index = self.check_address(address)?;
        Ok(self.bytes[index])
    }

    pub fn write_u8(&mut self, address: Address, value: u8) -> SaveResult<()> {
        let index = self.check_address(address)?;
        self.bytes[index] = value;
        Ok(())
    }

    pub fn read_u16_le(&self, address: Address) -> SaveResult<u16> {
        let lo = self.read_u8(address)?;
        let hi = self.read_u8(Address(address.0 + 1))?;
        Ok(u16::from_le_bytes([lo, hi]))
    }

    pub fn read_u16_be(&self, address: Address) -> SaveResult<u16> {
        let hi = self.read_u8(address)?;
        let lo = self.read_u8(Address(address.0 + 1))?;
        Ok(u16::from_be_bytes([hi, lo]))
    }

    pub fn write_u16_le(&mut self, address: Address, value: u16) -> SaveResult<()> {
        let [lo, hi] = value.to_le_bytes();
        self.write_u8(address, lo)?;
        self.write_u8(Address(address.0 + 1), hi)?;
        Ok(())
    }

    pub fn write_u16_be(&mut self, address: Address, value: u16) -> SaveResult<()> {
        let [hi, lo] = value.to_be_bytes();
        self.write_u8(address, hi)?;
        self.write_u8(Address(address.0 + 1), lo)?;
        Ok(())
    }

    pub fn read_bytes(&self, range: AddressRange) -> SaveResult<Vec<u8>> {
        let r = self.check_range(range)?;
        Ok(self.bytes[r].to_vec())
    }

    pub fn slice(&self, range: AddressRange) -> SaveResult<&[u8]> {
        let r = self.check_range(range)?;
        Ok(&self.bytes[r])
    }

    pub fn slice_mut(&mut self, range: AddressRange) -> SaveResult<&mut [u8]> {
        let r = self.check_range(range)?;
        Ok(&mut self.bytes[r])
    }

    pub fn write_bytes(&mut self, start: Address, data: &[u8]) -> SaveResult<()> {
        let end = Address(start.0 + data.len() as u32);
        let r = self.check_range(AddressRange::new(start, end))?;
        self.bytes[r].copy_from_slice(data);
        Ok(())
    }

    pub fn fill(&mut self, range: AddressRange, value: u8) -> SaveResult<()> {
        let r = self.check_range(range)?;
        self.bytes[r].fill(value);
        Ok(())
    }

    pub fn fill_len(&mut self, start: Address, len: Size, value: u8) -> SaveResult<()> {
        if len.0 == 0 {
            return Ok(());
        }

        self.fill(AddressRange::new(start, Address(start.0 + len.0)), value)
    }

    pub fn clear_len(&mut self, start: Address, len: Size) -> SaveResult<()> {
        self.fill_len(start, len, 0)
    }

    /// Copies bytes from `src` into this buffer.
    ///
    /// This is a safe, bounds-checked equivalent of `dst[dst..dst+len] = src[src..src+len]`.
    pub fn copy_from_other(
        &mut self,
        src: &SaveBinary,
        src_start: Address,
        dst_start: Address,
        len: Size,
    ) -> SaveResult<()> {
        if len.0 == 0 {
            return Ok(());
        }

        let src_end = Address(src_start.0 + len.0);
        let dst_end = Address(dst_start.0 + len.0);

        let src_range = src.check_range(AddressRange::new(src_start, src_end))?;
        let dst_range = self.check_range(AddressRange::new(dst_start, dst_end))?;
        self.bytes[dst_range].copy_from_slice(&src.bytes[src_range]);
        Ok(())
    }

    /// Copies `len` bytes from `src` to `dst` with memmove-like overlap behavior.
    pub fn copy_within(&mut self, src: Address, dst: Address, len: Size) -> SaveResult<()> {
        if len.0 == 0 {
            return Ok(());
        }

        let src_end = Address(src.0 + len.0);
        let dst_end = Address(dst.0 + len.0);
        self.check_range(AddressRange::new(src, src_end))?;
        self.check_range(AddressRange::new(dst, dst_end))?;

        let src_range = src.as_usize()..src_end.as_usize();
        self.bytes.copy_within(src_range, dst.as_usize());
        Ok(())
    }

    pub fn read_bit(&self, address: Address, bit: u8) -> SaveResult<bool> {
        if bit > 7 {
            return Err(SaveError::InvalidBitIndex { bit });
        }

        let value = self.read_u8(address)?;
        Ok((value & (1u8 << bit)) != 0)
    }

    pub fn write_bit(&mut self, address: Address, bit: u8, set: bool) -> SaveResult<()> {
        if bit > 7 {
            return Err(SaveError::InvalidBitIndex { bit });
        }

        let mut value = self.read_u8(address)?;
        let mask = 1u8 << bit;
        if set {
            value |= mask;
        } else {
            value &= !mask;
        }

        self.write_u8(address, value)
    }

    pub fn read_indexed_bit(&self, base: Address, bit_index: usize) -> SaveResult<bool> {
        let byte_offset = (bit_index / 8) as u32;
        let bit = (bit_index % 8) as u8;
        self.read_bit(Address(base.0 + byte_offset), bit)
    }

    pub fn write_indexed_bit(
        &mut self,
        base: Address,
        bit_index: usize,
        set: bool,
    ) -> SaveResult<()> {
        let byte_offset = (bit_index / 8) as u32;
        let bit = (bit_index % 8) as u8;
        self.write_bit(Address(base.0 + byte_offset), bit, set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_u16_be_round_trip() {
        let mut save = SaveBinary::new(vec![0; 8]);
        save.write_u16_be(Address(2), 0x1234).unwrap();
        assert_eq!(save.read_u16_be(Address(2)).unwrap(), 0x1234);
        assert_eq!(save.as_bytes()[2], 0x12);
        assert_eq!(save.as_bytes()[3], 0x34);
    }

    #[test]
    fn copy_within_handles_overlap() {
        let mut save = SaveBinary::new((0u8..=9).collect());
        save.copy_within(Address(0), Address(2), Size(8)).unwrap();
        assert_eq!(save.as_bytes(), &[0, 1, 0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn out_of_bounds_errors() {
        let save = SaveBinary::new(vec![0; 4]);
        let err = save.read_u8(Address(4)).unwrap_err();
        match err {
            SaveError::AddressOutOfBounds { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
