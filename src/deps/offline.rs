extern crate walkdir;
use walkdir::{DirEntry, WalkDir};
use std::path::{PathBuf, Path};


#[derive(Default, Debug, Clone)]
pub struct OfflineDependencySearch {
	recursive: bool,
	follow_symlinks: bool,
	search_path: Vec<PathBuf>,
	files_primary: Vec<PathBuf>,
	files_secondary: Vec<PathBuf>,
}

impl OfflineDependencySearch {
	pub fn new(recursive: bool, follow_symlinks: bool) -> Self {
		Self { recursive,
		       follow_symlinks,
		       ..Default::default() }
	}

	pub fn new_from_opts(opts: &crate::cli::InputFs) -> Self {
		let mut deps = Self::new(opts.search_recursive, opts.follow_symlinks);
		opts.dependencies.iter().for_each(|d| {
			                        match d.extension().map(|s| s.to_str()).flatten() {
				                        Some("move") => deps.add_search_file(d),
			                           _ => deps.add_search_dir(d),
			                        }
		                        });
		deps
	}

	pub fn add_search_dir<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		if !self.recursive {
			self.search_path.push(path.into());
		} else {
			self.add_search_dir_recursive(path.into());
		}
	}

	fn add_search_dir_recursive<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		debug!("adding search dir recursively {}", path.as_ref().display());

		let root = path.into();
		self.search_path.push(root.clone());

		let is_allowed = |entry: &DirEntry| {
			entry.file_type().is_dir() && !entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
		};

		let mut add_secondary = Vec::new();
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
				                  trace!("added search dir: {}", sub.display());

				                  // add found files for secondary search level
				                  if let Ok(dir) = std::fs::read_dir(entry.path()) {
					                  dir.filter_map(|e| e.ok())
					                     // filter .move files only
					                     .filter(|p| matches!(p.path().extension().map(|s| s.to_str()).flatten(), Some("move")))
					                     // prevent duplicates
					                     .filter(|p| !self.files_primary.contains(&p.path()))
					                     .filter(|p| !self.files_secondary.contains(&p.path()))
					                     .for_each(|f| add_secondary.push(f.path()));
				                  }
			                  }
		                  });

		// finally dedup & push:
		add_secondary.sort();
		add_secondary.dedup();
		self.files_secondary.extend(add_secondary);
	}

	pub fn add_search_file<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		trace!("added dep bin: {}", path.as_ref().display());
		self.files_primary.push(path.into());
	}
}

impl OfflineDependencySearch {
	pub fn has_file<T: AsRef<Path>>(&self, path: T) -> bool {
		// chained `contains` impl:
		self.files_primary
		    .iter()
		    .chain(self.files_secondary.iter())
		    .any(|p| p.as_path() == path.as_ref())
	}
}
