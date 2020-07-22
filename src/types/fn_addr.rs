use std::fmt::{Display, LowerHex, UpperHex, Binary, Formatter, Result};
use libra::libra_types::account_address::AccountAddress;
use serde::Serialize;
use super::ModAddr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct FnAddr(pub ModAddr, pub String);

impl FnAddr {
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

pub trait IntoFnAddr {
    fn into_fn_addr(self) -> FnAddr;
}
impl<T: Into<FnAddr>> IntoFnAddr for T {
    fn into_fn_addr(self) -> FnAddr {
        self.into()
    }
}

impl<'a, A: Into<ModAddr>, S: ToString> From<(A, S)> for FnAddr {
    fn from(v: (A, S)) -> Self {
        FnAddr(v.0.into(), v.1.to_string())
    }
}
impl<'a, S: ToString> From<&'a (ModAddr, S)> for FnAddr {
    fn from(v: &'a (ModAddr, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}
impl<'a, S: ToString> From<(&'a ModAddr, S)> for FnAddr {
    fn from(v: (&'a ModAddr, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}
impl<'a, S: ToString> From<&'a (&'a ModAddr, S)> for FnAddr {
    fn from(v: &'a (&'a ModAddr, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}

impl Display for FnAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}::{}", self.0, self.1)
    }
}

impl LowerHex for FnAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        LowerHex::fmt(&self.0, f).and_then(|_| write!(f, "::{}", self.1))
    }
}

// TODO: impl UpperHex for FnAddr

impl Binary for FnAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Binary::fmt(&self.0, f).and_then(|_| write!(f, "::{}", self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr_42() -> FnAddr {
        let module = ModAddr::test_addr_42();
        FnAddr(module.into(), "foo".to_owned())
    }

    #[test]
    fn fn_addr_fmt_hex() {
        let addr = format!("{:#x}", addr_42());
        assert_eq!("0042000000000000", &addr[..16]);
        assert_eq!("::Foo::foo", &addr[(addr.len() - 10)..]);
    }

    #[test]
    #[ignore]
    fn fn_addr_fmt_bin() {
        let addr: FnAddr = addr_42();
        #[rustfmt::skip]
		assert_eq!("0b00000000000100001000000000000000000000000000000000000000000000::Foo::foo", format!("{:#064b}", addr));
        #[rustfmt::skip]
		assert_eq!("100001000000000000000000000000000000000000000000000::Foo::foo", format!("{:b}", addr));
    }
}
