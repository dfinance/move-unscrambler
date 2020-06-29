#[macro_use]
extern crate log;


use libra::libra_types::account_address::AccountAddress;


pub fn get<S: AsRef<str>>(addr: &AccountAddress, name: S /* ,cfg:NetCfg */) -> Result<Vec<u8>, ()> {
	error!("TODO: impl req {} {}", addr.to_string(), name.as_ref());
	todo!("impl me");
}
