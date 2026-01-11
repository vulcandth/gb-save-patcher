use std::fmt;
use std::ops::Range;

/// Returns the number of bytes required to store `bits` bits.
///
/// This is equivalent to $\lceil bits/8 \rceil$.
///
/// # Example
/// ```
/// use gb_save_core::bits_to_bytes;
/// assert_eq!(bits_to_bytes(0), 0);
/// assert_eq!(bits_to_bytes(1), 1);
/// assert_eq!(bits_to_bytes(8), 1);
/// assert_eq!(bits_to_bytes(9), 2);
/// ```
#[must_use]
pub fn bits_to_bytes(bits: usize) -> usize {
    bits.div_ceil(8)
}

/// Absolute address into a save buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(
    /// Absolute offset (in bytes) into the save buffer.
    pub u32,
);

impl Address {
    /// Converts the address to a `usize` index for use with slices.
    #[must_use]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

/// Size in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size(
    /// Number of bytes.
    pub u32,
);

impl Size {
    /// Converts the size to `usize`.
    #[must_use]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Half-open byte range `[start, end)` in absolute address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddressRange {
    /// Inclusive start address.
    pub start: Address,
    /// Exclusive end address.
    pub end: Address,
}

impl AddressRange {
    /// Creates a new half-open range `[start, end)`.
    #[must_use]
    pub fn new(start: Address, end: Address) -> Self {
        Self { start, end }
    }

    /// Returns the range length in bytes.
    #[must_use]
    pub fn len(self) -> Size {
        Size(self.end.0.saturating_sub(self.start.0))
    }

    /// Converts the range to a `Range<usize>` suitable for slice indexing.
    #[must_use]
    pub fn to_usize_range(self) -> Range<usize> {
        self.start.as_usize()..self.end.as_usize()
    }
}

impl fmt::Display for AddressRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}
