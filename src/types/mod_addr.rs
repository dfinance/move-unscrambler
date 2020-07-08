use std::convert::TryInto;
use std::fmt::{Display, LowerHex, UpperHex, Binary, Formatter, Result};
use libra::libra_types::account_address::AccountAddress;
use libra::move_core_types::identifier::IdentStr;
use libra::move_core_types::language_storage::ModuleId;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ModAddr(AccountAddress, String);

impl ModAddr {
    pub fn new<A: Into<AccountAddress>, S: ToString>(addr: A, name: S) -> Self {
        Self(addr.into(), name.to_string())
    }

    pub fn name(&self) -> &str {
        &self.1
    }
    pub fn addr(&self) -> &AccountAddress {
        &self.0
    }

    pub fn split(self) -> (AccountAddress, String) {
        (self.0, self.1)
    }
    pub fn split_ref(&self) -> (&AccountAddress, &str) {
        (&self.0, &self.1)
    }
}

pub trait IntoModAddr {
    fn into_mod_addr(self) -> ModAddr;
}
impl<T: Into<ModAddr>> IntoModAddr for T {
    fn into_mod_addr(self) -> ModAddr {
        self.into()
    }
}

impl<'a, S: ToString> From<(AccountAddress, S)> for ModAddr {
    fn from(v: (AccountAddress, S)) -> Self {
        Self(v.0, v.1.to_string())
    }
}
impl<'a, S: ToString> From<&'a (AccountAddress, S)> for ModAddr {
    fn from(v: &'a (AccountAddress, S)) -> Self {
        Self(v.0, v.1.to_string())
    }
}
impl<'a, S: ToString> From<(&'a AccountAddress, S)> for ModAddr {
    fn from(v: (&'a AccountAddress, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}
impl<'a, S: ToString> From<&'a (&'a AccountAddress, S)> for ModAddr {
    fn from(v: &'a (&'a AccountAddress, S)) -> Self {
        Self(v.0.to_owned(), v.1.to_string())
    }
}
impl From<ModuleId> for ModAddr {
    fn from(mod_id: ModuleId) -> Self {
        (mod_id.address(), mod_id.name()).into()
    }
}

impl Display for ModAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}::{}", self.0, self.1)
    }
}

impl LowerHex for ModAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        LowerHex::fmt(&self.0, f).and_then(|_| write!(f, "::{}", self.1))
    }
}

impl UpperHex for ModAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        UpperHex::fmt(&u128::from_be_bytes(self.0.into()), f)
            .and_then(|_| write!(f, "::{}", self.1))
    }
}

impl Binary for ModAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Binary::fmt(&u128::from_be_bytes(self.0.into()), f).and_then(|_| write!(f, "::{}", self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr_empty() -> ModAddr {
        (
            AccountAddress::new([0; AccountAddress::LENGTH]),
            "Foo".to_owned(),
        )
            .into()
    }
    fn addr_42() -> ModAddr {
        (
            AccountAddress::new(0x004200000000000_u128.to_be_bytes()),
            "Foo".to_owned(),
        )
            .into()
    }

    #[test]
    fn mod_addr_fmt_hex() {
        let addr: ModAddr = addr_42();
        assert_eq!(
            "00000000000000000004200000000000::Foo",
            format!("{:x}", addr)
        );
        assert_eq!("0x4200000000000::Foo", format!("{:#X}", addr));

        let addr: ModAddr = addr_empty();
        assert_eq!(
            "00000000000000000000000000000000::Foo",
            format!("{:x}", addr)
        );
        assert_eq!("0x0::Foo", format!("{:#X}", addr));
    }

    #[test]
    fn mod_addr_fmt_bin() {
        let addr: ModAddr = addr_42();
        #[rustfmt::skip]
		assert_eq!("0b00000000000100001000000000000000000000000000000000000000000000::Foo", format!("{:#064b}", addr));
        #[rustfmt::skip]
		assert_eq!("100001000000000000000000000000000000000000000000000::Foo", format!("{:b}", addr));
    }
}
