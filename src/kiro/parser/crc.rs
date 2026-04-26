//! CRC32 checksum implementation
//!
//! AWS Event Stream uses CRC32 (ISO-HDLC/Ethernet/ZIP standard)

use crc::{CRC_32_ISO_HDLC, Crc};

/// CRC32 calculator instance (ISO-HDLC standard, polynomial 0xEDB88320)
const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// Compute the CRC32 checksum (ISO-HDLC standard)
///
/// # Arguments
/// * `data` - Data to compute the checksum for
///
/// # Returns
/// CRC32 checksum value
pub fn crc32(data: &[u8]) -> u32 {
    CRC32.checksum(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        // CRC32 of empty data should be 0
        assert_eq!(crc32(&[]), 0);
    }

    #[test]
    fn test_crc32_known_value() {
        // CRC32 (ISO-HDLC) of "123456789" is 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32(data), 0xCBF43926);
    }
}
