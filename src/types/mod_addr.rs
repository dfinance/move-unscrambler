use std::convert::TryInto;
use std::fmt::{Display, LowerHex, UpperHex, Binary, Formatter, Result};
use libra::libra_types::account_address::AccountAddress;
use libra::move_core_types::identifier::IdentStr;
use libra::move_core_types::language_storage::ModuleId;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ModAddr(pub AccountAddress, pub String);

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

// TODO: impl UpperHex for ModAddr

// TODO: fix binary fmt impl
impl Binary for ModAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Binary::fmt(&u128::from_be_bytes(self.0.into()), f).and_then(|_| write!(f, "::{}", self.1))
        let arr: [u8; AccountAddress::LENGTH] = self.0.into();
        let arr_16: [u8; 16] = (&arr[..16]).try_into().unwrap();
        // TODO: type by len of arr_other
        let _arr_other: [u8; AccountAddress::LENGTH - 16] = (&arr[16..]).try_into().unwrap();
        Binary::fmt(&u128::from_be_bytes(arr_16.into()), f).and_then(|_| write!(f, "::{}", self.1))
    }
}

#[cfg(test)]
impl ModAddr {
    pub(crate) fn test_addr_empty() -> ModAddr {
        (AccountAddress::ZERO, "Foo".to_owned()).into()
    }

    pub(crate) fn test_addr_42() -> ModAddr {
        let mut arr = [0; AccountAddress::LENGTH];
        arr[1] = 66;
        (
            // AccountAddress::new(0x004200000000000_u128.to_be_bytes()),
            AccountAddress::new(arr),
            "Foo".to_owned(),
        )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_addr_fmt_hex() {
        let addr = format!("{:#x}", ModAddr::test_addr_42());
        assert_eq!("0042000000000000", &addr[..16]);
        assert_eq!("::Foo", &addr[(addr.len() - 5)..]);

        let addr = format!("{:#x}", ModAddr::test_addr_empty());
        assert_eq!("0000000000000000", &addr[..16]);
        assert_eq!("::Foo", &addr[(addr.len() - 5)..]);
    }

    #[test]
    #[ignore]
    fn mod_addr_fmt_bin() {
        let addr: ModAddr = ModAddr::test_addr_42();
        #[rustfmt::skip]
    		assert_eq!("0b00000000000100001000000000000000000000000000000000000000000000::Foo", format!("{:#064b}", addr));
        #[rustfmt::skip]
    		assert_eq!("100001000000000000000000000000000000000000000000000::Foo", format!("{:b}", addr));
    }
}
