use std::{collections::HashMap, cell::RefCell, hash::Hash};
use libra::libra_types::account_address::AccountAddress;
use super::map::{ModMap, AsMap, ModInfo, DependencyMap};
use super::DependencySource;
use super::DependencySearch;
use crate::types::ModAddr;


pub type UnresolvedMap<K> = HashMap<K, anyhow::Error>;

pub struct DependencyResolverMap<Q, Storage> {
	map: Storage,
	unresolved: UnresolvedMap<Q>,
	searchers: Vec<Box<dyn DependencySearch<Q>>>,
}

impl<Q, Storage> DependencyResolverMap<Q, Storage> {
	pub fn new(map: Storage) -> Self {
		Self { map,
		       searchers: Default::default(),
		       unresolved: Default::default() }
	}

	pub fn split(self) -> (Storage, UnresolvedMap<Q>) { (self.map, self.unresolved) }

	pub fn add_searcher<F>(&mut self, resolver: F)
		where F: Into<Box<dyn DependencySearch<Q>>> {
		self.searchers.push(resolver.into());
	}
}

impl<Q, Storage> DependencyResolverMap<Q, Storage>
	where Q: ToOwned<Owned = Q>,
	      Q: Clone + Eq + PartialEq + Hash
{
	pub fn search(&mut self, query: &Q) -> Option<(DependencySource, Vec<u8>)> {
		let mut error = None;
		let result = self.searchers
		                 .iter()
		                 .filter_map(|s| {
			                 s.search(query.to_owned())
			                  .map_err(|err| {
				                  error = Some(err);
			                  })
			                  .ok()
		                 })
		                 .next();

		if let (None, Some(error)) = (&result, error) {
			self.unresolved.insert(query.to_owned(), error);
		}

		result
	}
}


impl DependencyResolverMap<ModAddr, ModMap> {
	pub fn prefetch_deps(&mut self, keys: &[ModAddr]) -> Vec<ModAddr> {
		let keys: Vec<_> = keys.iter()
		                       .filter(|key| !self.map.as_map().contains_key(key))
		                       .collect();

		let mut added = Vec::new();
		for key in keys {
			if let Some(found) = self.search(&key) {
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
