use std::{collections::HashMap, cell::RefCell};
use libra::libra_types::account_address::AccountAddress;
use super::map::{DependencyMapKey, DependencyMap, MapAccess, DependencyInfo, DependencySource};
use super::DependencySearch;


pub struct DependencyResolverMap {
	map: DependencyMap,
	searchers: Vec<Box<dyn DependencySearch>>,
}

impl DependencyResolverMap {
	pub fn new(map: DependencyMap) -> Self {
		Self { map,
		       searchers: Vec::new() }
	}

	pub fn into_map(self) -> DependencyMap { self.map }
	pub fn as_map_mut(&mut self) -> &mut HashMap<DependencyMapKey, DependencyInfo> { self.map.as_map_mut() }

	pub fn add_searcher<F>(&mut self, resolver: F)
		where F: Into<Box<dyn DependencySearch>> {
		self.searchers.push(resolver.into());
	}

	pub fn search(&mut self, addr: &AccountAddress, name: &str) -> Option<(DependencySource, Vec<u8>)> {
		self.searchers
		    .iter()
		    .filter_map(|s| {
			    s.search(addr, name)
			     .map_err(|err| {
				     warn!("Module 0x{}:{} not found, Err: '{}'", addr, name, err);
			     })
			     .ok()
		    })
		    .next()
	}

	pub fn prefetch_deps(&mut self, keys: &[DependencyMapKey]) -> Vec<(AccountAddress, String)> {
		let keys: Vec<_> = keys.iter()
		                       .filter(|key| !self.map.as_map().contains_key(key))
		                       .collect();

		let mut added = Vec::new();
		for key in keys {
			if let Some(found) = self.search(&key.0, &key.1) {
				self.map.insert_mod_bin(found.0, found.1);
				added.push(key.to_owned());
			}
		}
		added
	}

	pub fn prefetch_deps_recursively(&mut self) {
		let mut keys: Vec<_> = self.map
		                           .as_map()
		                           .iter()
		                           .flat_map(|(_, v)| (v.dependencies()))
		                           .cloned()
		                           .collect();

		while let Some(key) = keys.pop() {
			let added = self.prefetch_deps(&[key]);
			let deps_of_added = added.iter()
			                         .filter_map(|key| self.map.as_map().get(key).map(|info| info.dependencies()))
			                         .flatten()
			                         .cloned();
			keys.extend(deps_of_added);
		}
	}
}
