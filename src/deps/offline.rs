extern crate walkdir;
use walkdir::{DirEntry, WalkDir};
use std::path::{PathBuf, Path};
use std::io::{Result, Error};
use std::fs;
use crate::output::utils::path_to_string;


const MOVE_BIN_EXT: &str = "mv";

#[derive(Default, Debug, Clone)]
pub struct OfflineDependencySearch {
	recursive: bool,
	follow_symlinks: bool,
	search_path: Vec<PathBuf>,
	files_primary: Vec<PathBuf>,
	files_secondary: Vec<PathBuf>,
	files_exclude: Vec<PathBuf>,
}

impl OfflineDependencySearch {
	pub fn new(recursive: bool, follow_symlinks: bool) -> Self {
		Self { recursive,
		       follow_symlinks,
		       ..Default::default() }
	}

	pub fn new_from_opts(opts: &crate::cli::InputFs) -> Self {
		let mut deps = Self::new(opts.search_recursive, opts.follow_symlinks);
		deps.files_exclude = vec![opts.path.to_owned()];
		opts.dependencies.iter().for_each(|d| {
			                        match d.extension().map(|s| s.to_str()).flatten() {
				                        Some(MOVE_BIN_EXT) => deps.add_search_file(d),
			                           _ => deps.add_search_dir(d),
			                        }
		                        });
		deps
	}

	pub fn add_search_dir<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		if !self.recursive {
			self.add_files_from_dir(&path);
			self.search_path.push(path.into());
		} else {
			self.add_search_dir_recursive(path.into());
		}
	}

	fn add_files_from_dir<T: AsRef<Path>>(&mut self, path: T) {
		use std::fs::canonicalize;
		let mut add_secondary = Vec::new();

		if let Ok(dir) = fs::read_dir(path) {
			dir.filter_map(|e| e.ok())
				// filter .move files only
				.filter(|p| matches!(p.path().extension().map(|s| s.to_str()).flatten(), Some(MOVE_BIN_EXT)))
				// prevent duplicates
				.filter(|p| !self.files_primary.contains(&p.path()))
				.filter(|p| !self.files_secondary.contains(&p.path()))
				.filter(|p| !self.files_exclude.contains(&p.path()))
				.filter_map(|p| canonicalize(p.path()).ok())
				.for_each(|f| {
					add_secondary.push(f)
				});
		}

		// finally dedup & push:
		add_secondary.sort();
		add_secondary.dedup();
		self.files_secondary.extend(add_secondary);
	}

	fn add_search_dir_recursive<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		debug!("adding search dir recursively {}", path_to_string(&path));

		let root = path.into();
		self.add_files_from_dir(root.as_path());
		self.search_path.push(root.clone());

		let is_allowed = |entry: &DirEntry| {
			entry.file_type().is_dir() &&
			!entry.file_name()
			      .to_str()
			      .map(|s| s.starts_with("."))
			      .unwrap_or(false)
		};

		WalkDir::new(root).follow_links(self.follow_symlinks)
		                  .into_iter()
		                  .filter_entry(is_allowed)
		                  .filter_map(|e| e.ok())
		                  .for_each(|entry| {
			                  let sub = entry.path();
			                  let sub_buf = entry.path().to_owned();
			                  // prevent cycles on symlinks:
			                  if !self.search_path.contains(&sub_buf) {
				                  self.search_path.push(sub_buf);
				                  trace!("added search dir: {}", path_to_string(&sub));
				                  // add found files for secondary search level
				                  self.add_files_from_dir(sub);
			                  }
		                  });

		{
			// INSTEAD OF - just call self.add_files_from_dir(path) for each search-path
			// self.add_files_from_dir(path)
		}
	}

	pub fn add_search_file<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		let pathbuf = path.into();
		if !self.files_exclude.contains(&pathbuf) {
			trace!("add dep bin: {}", path_to_string(&pathbuf));
			self.files_primary.push(pathbuf);
		} else {
			trace!("skip because already added: {}", path_to_string(&pathbuf));
		}
	}
}

impl OfflineDependencySearch {
	pub fn all_files(&self) -> impl Iterator<Item = &PathBuf> {
		self.files_primary.iter().chain(self.files_secondary.iter())
	}
	pub fn into_all_files(self) -> impl Iterator<Item = PathBuf> {
		self.files_primary
		    .into_iter()
		    .chain(self.files_secondary.into_iter())
	}

	pub fn has_file<T: AsRef<Path>>(&self, path: T) -> bool {
		// chained `contains` impl:
		self.all_files().any(|p| p.as_path() == path.as_ref())
	}
}


impl OfflineDependencySearch {
	pub fn into_load_all(self) -> impl Iterator<Item = (PathBuf, Result<Vec<u8>>)> {
		self.into_all_files().map(|p| (p.clone(), fs::read(p)))
	}
}
