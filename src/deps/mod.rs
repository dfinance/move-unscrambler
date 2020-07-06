use libra::libra_types::account_address::AccountAddress;
use std::path::PathBuf;

pub mod map;
pub mod offline;
pub mod online;
pub mod resolver;


pub trait DependencySearch<Q> {
	fn search(&self, query: Q) -> anyhow::Result<(DependencySource, Vec<u8>)>;
}

impl<Q, T: DependencySearch<Q>> DependencySearch<Q> for Box<T> {
	fn search(&self, query: Q) -> anyhow::Result<(DependencySource, Vec<u8>)> {
		let inner: &T = self.as_ref();
		inner.search(query)
	}
}

impl<Q> DependencySearch<Q> for Box<dyn DependencySearch<Q>> {
	fn search(&self, query: Q) -> anyhow::Result<(DependencySource, Vec<u8>)> {
		let inner: &dyn DependencySearch<_> = self.as_ref();
		inner.search(query)
	}
}


#[derive(Debug, Clone)]
pub enum DependencySource {
	Fs(PathBuf),
	Net,
	None,
}

impl<T: Into<PathBuf>> From<T> for DependencySource {
	fn from(p: T) -> Self { DependencySource::Fs(p.into()) }
}
