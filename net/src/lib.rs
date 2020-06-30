#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use libra::libra_types::account_address::AccountAddress;
use libra::move_core_types::language_storage::ModuleId;
use libra::move_core_types::identifier::Identifier;
use libra::libra_types::access_path::AccessPath;

pub fn get(addr: &AccountAddress, name: impl Into<Box<str>>, cfg: &NetCfg) -> Result<Vec<u8>, Error> {
	let path = AccessPath::code_access_path(&ModuleId::new(*addr, Identifier::new(name)?));
	let url = format!(
	                  "{base_url}vm/data/{address}/{path}",
	                  base_url = cfg.node_base_url(),
	                  address = hex::encode(&path.address),
	                  path = hex::encode(path.path)
	);

	let resp = reqwest::blocking::get(&url)?;
	if resp.status().is_success() {
		let res: LoaderResponse = resp.json()?;
		Ok(hex::decode(&res.result.value)?)
	} else {
		let res: LoaderErrorResponse = resp.json()?;
		Err(anyhow!("Failed to load dependencies :'{}' [{}]", url, res.error))
	}
}

pub struct NetCfg {
	node_base_url: String,
}

impl NetCfg {
	pub fn new(node_base_url: String) -> Self { NetCfg { node_base_url } }

	pub fn node_base_url(&self) -> &str { &self.node_base_url }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct LoaderResponse {
	result: Response,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Response {
	value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct LoaderErrorResponse {
	error: String,
}
