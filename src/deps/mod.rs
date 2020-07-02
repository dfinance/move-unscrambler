use libra::libra_types::account_address::AccountAddress;
use map::DependencySource;

pub mod map;
pub mod offline;
pub mod online;
pub mod resolver;


pub trait DependencySearch {
	fn search(&self, addr: &AccountAddress, name: &str) -> anyhow::Result<(DependencySource, Vec<u8>)>;
}

impl<T: DependencySearch> DependencySearch for Box<T> {
	fn search(&self, addr: &AccountAddress, name: &str) -> anyhow::Result<(DependencySource, Vec<u8>)> {
		let inner: &T = self.as_ref();
		inner.search(addr, name)
	}
}

impl DependencySearch for Box<dyn DependencySearch> {
	fn search(&self, addr: &AccountAddress, name: &str) -> anyhow::Result<(DependencySource, Vec<u8>)> {
		let inner: &dyn DependencySearch = self.as_ref();
		inner.search(addr, name)
	}
}
