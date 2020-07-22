const NATIVE_ADDRESS_LENGTH: usize = crate::NATIVE_ADDR_LEN;

pub fn expand_addr(src: impl Iterator<Item = u8>, to_length: usize) -> impl Iterator<Item = u8> {
    src.into_iter()
        .chain((0..(NATIVE_ADDRESS_LENGTH - to_length) as u8).map(|_| 0))
}
