use std::path::PathBuf;
use std::collections::HashMap;
use libra::libra_types::account_address::AccountAddress;


// TODO: make/fill DependencyIndex from offline::OfflineDependencySearch


pub struct DependencyIndex {
	// key: (AccountAddress, name:String)
	// val: (opt<fs-path>, bc, meta: { script/mod,  })
	map: HashMap<(AccountAddress, /* name: */ String), DependencyInfo>,
}

pub struct DependencyInfo {
	source: DependencySource,
	bytes: Vec<u8>,
	address: AccountAddress,
	name: String,
}

pub enum DependencySource {
	Fs(PathBuf),
	Net,
	None,
}

impl DependencyIndex {}
