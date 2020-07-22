use std::fmt::{Display, LowerHex, UpperHex, Binary, Formatter, Result};
use libra::libra_types::account_address::AccountAddress;
use super::FnAddr;

type BlockId = u16;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BlockAddr(FnAddr, BlockId);

impl BlockAddr {
    pub fn new<F: Into<FnAddr>, S: ToString>(fn_addr: F, id: BlockId) -> Self {
        Self(fn_addr.into(), id)
    }

    pub fn id(&self) -> BlockId {
        self.1
    }
    pub fn addr(&self) -> &FnAddr {
        &self.0
    }

    pub fn split(self) -> (FnAddr, BlockId) {
        (self.0, self.1)
    }
    pub fn split_ref(&self) -> (&FnAddr, BlockId) {
        (&self.0, self.1)
    }
}

pub trait IntoBlockAddr {
    fn into_block_addr(self) -> BlockAddr;
}
impl<T: Into<BlockAddr>> IntoBlockAddr for T {
    fn into_block_addr(self) -> BlockAddr {
        self.into()
    }
}

impl<'a> From<(FnAddr, BlockId)> for BlockAddr {
    fn from(v: (FnAddr, BlockId)) -> Self {
        Self(v.0, v.1)
    }
}
impl<'a> From<&'a (FnAddr, BlockId)> for BlockAddr {
    fn from(v: &'a (FnAddr, BlockId)) -> Self {
        Self(v.0.to_owned(), v.1)
    }
}
impl<'a> From<(&'a FnAddr, BlockId)> for BlockAddr {
    fn from(v: (&'a FnAddr, BlockId)) -> Self {
        Self(v.0.to_owned(), v.1)
    }
}
impl<'a> From<&'a (&'a FnAddr, BlockId)> for BlockAddr {
    fn from(v: &'a (&'a FnAddr, BlockId)) -> Self {
        Self(v.0.to_owned(), v.1)
    }
}

impl Display for BlockAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:#{}", self.0, self.1)
    }
}

impl LowerHex for BlockAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        LowerHex::fmt(&self.0, f).and_then(|_| write!(f, ":#{}", self.1))
    }
}

// TODO: impl UpperHex for BlockAddr

impl Binary for BlockAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Binary::fmt(&self.0, f).and_then(|_| write!(f, ":#{}", self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ModAddr;

    fn addr_42() -> BlockAddr {
        let module = ModAddr::test_addr_42();
        let function = FnAddr::new(module, "foo");
        BlockAddr(function.into(), 42)
    }

    #[test]
    fn block_addr_fmt_hex() {
        let addr = format!("{:#x}", addr_42());
        assert_eq!("0042000000000000", &addr[..16]);
        assert_eq!("::Foo::foo:#42", &addr[(addr.len() - 14)..]);
    }

    #[test]
    #[ignore]
    fn block_addr_fmt_bin() {
        let addr: BlockAddr = addr_42();
        #[rustfmt::skip]
		assert_eq!("0b00000000000100001000000000000000000000000000000000000000000000::Foo::foo:#42", format!("{:#064b}", addr));
        #[rustfmt::skip]
		assert_eq!("100001000000000000000000000000000000000000000000000::Foo::foo:#42", format!("{:b}", addr));
    }
}
