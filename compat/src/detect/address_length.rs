use anyhow::Result;

const NATIVE_ADDRESS_LENGTH: usize = crate::NATIVE_ADDR_LEN;

/// Dummy implementation of determining length (`X`) of known total length.
/// Dummy because works for `X` values less than `NATIVE_ADDRESS_LENGTH`.
pub fn address_length(addr_tables_lengths: &[u32]) -> Result<u32> {
    if addr_tables_lengths
        .iter()
        .cloned()
        .map(|v| v as usize % NATIVE_ADDRESS_LENGTH)
        .all(|v| v == 0)
    {
        return Ok(NATIVE_ADDRESS_LENGTH as u32);
    }

    let minimal = addr_tables_lengths.iter().cloned().min();

    // search minimal len, less then natively supported address length:
    if let Some(min) = minimal.filter(|v| *v as usize <= NATIVE_ADDRESS_LENGTH) {
        return Ok(min);
    }

    if let Some(min) = minimal {
        if min % 2 != 0 {
            return Ok(min);
        } else {
            let mut min = min;
            while min as usize > NATIVE_ADDRESS_LENGTH {
                min /= 2;
            }
            return Ok(min);
        }
    }

    Err(anyhow!("Unable to detect address length"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE: u32 = NATIVE_ADDRESS_LENGTH as u32;

    fn detect_unwrap(addr_tables_lengths: &[u32]) -> u32 {
        address_length(addr_tables_lengths).unwrap()
    }

    #[test]
    fn detect_address_length_one_lower_eq() {
        assert_eq!(16, detect_unwrap(&[16]));
        assert_eq!(15, detect_unwrap(&[15]));
        assert_eq!(14, detect_unwrap(&[14]));
    }

    #[test]
    fn detect_address_length_mul_lower_eq() {
        assert_eq!(16, detect_unwrap(&[16, 32]));
        assert_eq!(15, detect_unwrap(&[15, 30]));
        assert_eq!(14, detect_unwrap(&[14, 28]));
    }

    #[test]
    fn detect_address_length_one_lower() {
        assert_eq!(16, detect_unwrap(&[32]));
        assert_eq!(15, detect_unwrap(&[30]));
        assert_eq!(14, detect_unwrap(&[28]));
    }

    #[test]
    fn detect_address_length_mul_lower() {
        assert_eq!(16, detect_unwrap(&[16 * 2, 16 * 3]));
        assert_eq!(15, detect_unwrap(&[15 * 2, 15 * 3]));
        assert_eq!(14, detect_unwrap(&[14 * 2, 14 * 3]));
    }

    #[test]
    fn detect_address_length_one_higher_eq() {
        assert_eq!(NATIVE, detect_unwrap(&[NATIVE * 2]));
    }

    #[test]
    fn detect_address_length_mul_higher_eq() {
        assert_eq!(NATIVE, detect_unwrap(&[NATIVE * 5]));
    }

    #[test]
    /// Test unsupported case when length is more than natively supported
    /// because we cannot cut tables, but expand only.
    fn detect_address_length_one_higher() {
        assert_ne!(22, detect_unwrap(&[22 * 2]));
        assert_ne!(24, detect_unwrap(&[24 * 2]));
    }

    #[test]
    /// Test unsupported case when length is more than natively supported
    /// because we cannot cut tables, but expand only.
    fn detect_address_length_mul_higher() {
        assert_ne!(22, detect_unwrap(&[22 * 2, 22 * 3]));
        assert_ne!(24, detect_unwrap(&[24 * 2, 24 * 3]));
    }
}
