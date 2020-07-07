use std::fmt::{Display, LowerHex, UpperHex, Binary, Formatter, Result};
use libra::libra_types::account_address::AccountAddress;
use super::ModAddr;

pub type ResAddr = StructAddr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructAddr(ModAddr, String);

impl StructAddr {
    pub fn new<M: Into<ModAddr>, S: ToString>(module: M, name: S) -> Self {
        Self(module.into(), name.to_string())
    }

    pub fn name(&self) -> &str {
        &self.1
    }
    pub fn addr(&self) -> &ModAddr {
        &self.0
    }

    pub fn split(self) -> (ModAddr, String) {
        (self.0, self.1)
    }
    pub fn split_ref(&self) -> (&ModAddr, &str) {
        (&self.0, &self.1)
    }
}

pub trait IntoResAddr {
    fn into_fn_addr(self) -> StructAddr;
}
impl<T: Into<StructAddr>> IntoResAddr for T {
    fn into_fn_addr(self) -> StructAddr {
        self.into()
    }
}

impl<'a, S: ToString> From<(ModAddr, S)> for StructAddr {
    fn from(v: (ModAddr, S)) -> Self {
        Self(v.0, v.1.to_string())
    }
}
impl<'a, S: ToString> From<&'a (ModAddr, S)> for StructAddr {
    fn from(v: &'a (ModAddr, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}
impl<'a, S: ToString> From<(&'a ModAddr, S)> for StructAddr {
    fn from(v: (&'a ModAddr, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}
impl<'a, S: ToString> From<&'a (&'a ModAddr, S)> for StructAddr {
    fn from(v: &'a (&'a ModAddr, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}

impl Display for StructAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}::{}", self.0, self.1)
    }
}

impl LowerHex for StructAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        LowerHex::fmt(&self.0, f).and_then(|_| write!(f, "::{}", self.1))
    }
}

impl UpperHex for StructAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        UpperHex::fmt(&self.0, f).and_then(|_| write!(f, "::{}", self.1))
    }
}

impl Binary for StructAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Binary::fmt(&self.0, f).and_then(|_| write!(f, "::{}", self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr_42() -> StructAddr {
        let module = (
            AccountAddress::new(0x004200000000000_u128.to_be_bytes()),
            "Foo".to_owned(),
        );
        StructAddr(module.into(), "FOO".to_owned())
    }

    #[test]
    fn fn_addr_fmt_hex() {
        let addr: StructAddr = addr_42();
        #[rustfmt::skip]
		assert_eq!("00000000000000000004200000000000::Foo::FOO", format!("{:x}", addr));
        assert_eq!("0x4200000000000::Foo::FOO", format!("{:#X}", addr));
    }

    #[test]
    fn fn_addr_fmt_bin() {
        let addr: StructAddr = addr_42();
        #[rustfmt::skip]
		assert_eq!("0b00000000000100001000000000000000000000000000000000000000000000::Foo::FOO", format!("{:#064b}", addr));
        #[rustfmt::skip]
		assert_eq!("100001000000000000000000000000000000000000000000000::Foo::FOO", format!("{:b}", addr));
    }
}
