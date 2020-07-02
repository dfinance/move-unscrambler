use net::NetCfg;
use libra::libra_types::account_address::AccountAddress;
use crate::cli::InputNet;
use super::map::{DependencySource, DependencyMapKey};
use super::DependencySearch;

pub struct OnlineDependencySearch {
	config: NetCfg<String>,
}

impl DependencySearch for OnlineDependencySearch {
	fn search(&self, addr: &AccountAddress, name: &str) -> anyhow::Result<(DependencySource, Vec<u8>)> {
		net::get(addr, name, &self.config).map(|bytes| (DependencySource::Net, bytes))
	}
}

impl OnlineDependencySearch {
	pub fn new(uri: &str) -> Self {
		let config = NetCfg::new(uri.to_owned());
		Self { config }
	}
	pub fn with_opts(opts: &InputNet) -> Self {
		Self::new(opts.ds
		              .as_ref()
		              .map(|s| s.as_str())
		              .expect("Online REST-data-source URI is missed"))
	}
}

impl Into<Box<dyn DependencySearch>> for OnlineDependencySearch {
	fn into(self) -> Box<dyn DependencySearch> { Box::new(self) }
}
