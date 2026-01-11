use std::collections::HashMap;
use std::io::Read;

use crate::{Address, SaveError, SaveResult};

/// A single symbol entry parsed from a `.sym` file.
///
/// `bank` is the memory bank, and `address` is the in-bank address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// Memory bank for the symbol.
    pub bank: u8,
    /// In-bank address for the symbol.
    pub address: u16,
}

/// A lookup table for `.sym` symbols used to translate symbolic addresses into save offsets.
///
/// # Example
/// ```
/// use gb_save_core::SymbolDatabase;
///
/// let db = SymbolDatabase::from_sym_text("00:ABE2 sSaveVersion\n");
/// assert!(db.contains("sSaveVersion"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct SymbolDatabase {
    symbols: HashMap<String, Symbol>,
}

impl SymbolDatabase {
    /// Creates an empty symbol database.
    #[must_use]
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Parses a `.sym` text blob.
    ///
    /// Lines that do not match the expected format are ignored.
    pub fn from_sym_text(text: &str) -> Self {
        let mut db = Self::new();
        for line in text.lines() {
            if let Some((name, symbol)) = parse_sym_line(line) {
                db.symbols.insert(name, symbol);
            }
        }
        db
    }

    /// Parses a gzip-compressed `.sym.gz` payload.
    ///
    /// # Errors
    /// Returns an error if decompression fails.
    pub fn from_gzip_bytes(gz_bytes: &[u8]) -> SaveResult<Self> {
        let mut decoder = flate2::read::GzDecoder::new(gz_bytes);
        let mut text = String::new();
        decoder
            .read_to_string(&mut text)
            .map_err(|_| SaveError::SymbolFileDecompressionFailed)?;
        Ok(Self::from_sym_text(&text))
    }

    /// Looks up a symbol by name.
    ///
    /// # Errors
    /// Returns an error if the symbol is missing.
    pub fn get_symbol(&self, name: &str) -> SaveResult<Symbol> {
        self.symbols
            .get(name)
            .copied()
            .ok_or_else(|| SaveError::SymbolNotFound {
                name: name.to_string(),
            })
    }

    /// Returns true if a symbol exists.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Iterates all symbols.
    ///
    /// The returned iterator yields `(name, symbol)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Symbol)> {
        self.symbols.iter().map(|(name, sym)| (name.as_str(), *sym))
    }

    /// Resolves a symbol expected to be in SRAM into an absolute save-buffer address.
    ///
    /// # Errors
    /// Returns an error if the symbol is missing or not in SRAM.
    pub fn sram_absolute_address(&self, name: &str) -> SaveResult<Address> {
        let symbol = self.get_symbol(name)?;
        if !(0xA000..0xC000).contains(&symbol.address) {
            return Err(SaveError::SymbolNotInSram {
                name: name.to_string(),
                address: symbol.address,
            });
        }

        let bank_offset = (symbol.bank as u32) * 0x2000;
        let address_offset = (symbol.address as u32) - 0xA000;
        Ok(Address(bank_offset + address_offset))
    }

    /// Returns true if `address` is in WRAM.
    #[must_use]
    pub fn is_wram_address(address: u16) -> bool {
        (0xC000..0xE000).contains(&address)
    }

    /// Returns true if `address` is in SRAM.
    #[must_use]
    pub fn is_sram_address(address: u16) -> bool {
        (0xA000..0xC000).contains(&address)
    }

    /// Resolves an address by taking a WRAM-relative offset and applying it to an SRAM base.
    ///
    /// This is convenient when the save data layout mirrors WRAM structs.
    ///
    /// # Errors
    /// Returns an error if any symbol is missing, in the wrong region, or would require a
    /// negative offset.
    pub fn wram_relative_to_sram_absolute_address(
        &self,
        base_wram_symbol: &str,
        base_sram_symbol: &str,
        wram_symbol: &str,
    ) -> SaveResult<Address> {
        let base_wram = self.get_symbol(base_wram_symbol)?;
        if !Self::is_wram_address(base_wram.address) {
            return Err(SaveError::SymbolNotInExpectedRegion {
                name: base_wram_symbol.to_string(),
                expected: "WRAM",
                address: base_wram.address,
            });
        }

        let wram = self.get_symbol(wram_symbol)?;
        if !Self::is_wram_address(wram.address) {
            return Err(SaveError::SymbolNotInExpectedRegion {
                name: wram_symbol.to_string(),
                expected: "WRAM",
                address: wram.address,
            });
        }

        let distance = i32::from(wram.address) - i32::from(base_wram.address);
        if distance < 0 {
            return Err(SaveError::SymbolBeforeBase {
                symbol: wram_symbol.to_string(),
                base: base_wram_symbol.to_string(),
            });
        }

        let base_sram = self.get_symbol(base_sram_symbol)?;
        if !Self::is_sram_address(base_sram.address) {
            return Err(SaveError::SymbolNotInExpectedRegion {
                name: base_sram_symbol.to_string(),
                expected: "SRAM",
                address: base_sram.address,
            });
        }

        let base = self.sram_absolute_address(base_sram_symbol)?;
        Ok(Address(base.0 + (distance as u32)))
    }
}

fn parse_sym_line(line: &str) -> Option<(String, Symbol)> {
    let line = line.trim_end_matches(['\r', '\n']);
    let mut parts = line.split_whitespace();
    let bank_and_addr = parts.next()?;
    let name = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    let (bank_str, addr_str) = bank_and_addr.split_once(':')?;
    if bank_str.len() != 2 || addr_str.len() != 4 {
        return None;
    }

    let bank = u8::from_str_radix(bank_str, 16).ok()?;
    let address = u16::from_str_radix(addr_str, 16).ok()?;

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return None;
    }

    Some((name.to_string(), Symbol { bank, address }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_symbol_lines_and_ignores_invalid() {
        let text = "00:ABE2 sSaveVersion\ninvalid\n01:AD0D sChecksum\n";
        let db = SymbolDatabase::from_sym_text(text);
        assert!(db.contains("sSaveVersion"));
        assert!(db.contains("sChecksum"));
        assert!(!db.contains("invalid"));
    }

    #[test]
    fn last_symbol_wins_on_duplicates() {
        let text = "00:0001 dup\n00:0002 dup\n";
        let db = SymbolDatabase::from_sym_text(text);
        assert_eq!(db.get_symbol("dup").unwrap().address, 0x0002);
    }

    #[test]
    fn missing_symbol_returns_typed_error() {
        let db = SymbolDatabase::new();
        let err = db.get_symbol("nope").unwrap_err();
        match err {
            SaveError::SymbolNotFound { name } => assert_eq!(name, "nope"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
