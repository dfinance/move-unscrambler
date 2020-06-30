extern crate walkdir;
use walkdir::{DirEntry, WalkDir};
use std::path::{PathBuf, Path};


#[derive(Default, Debug, Clone)]
pub struct OfflineDependencyResolver {
	recursive: bool,
	search_path: Vec<PathBuf>,
	files_primary: Vec<PathBuf>,
	files_secondary: Vec<PathBuf>,
}

impl OfflineDependencyResolver {
	pub fn new(recursive: bool) -> Self {
		Self { recursive,
		       ..Default::default() }
	}

	pub fn new_from_opts(opts: &crate::cli::Input) {
		let mut deps = Self::new(opts.search_recursive);
		opts.dependencies.iter().for_each(|d| {
			                        match d.extension().map(|s| s.to_str()).flatten() {
				                        Some("move") => deps.add_search_file(d),
			                           _ => deps.add_search_dir(d),
			                        }
		                        });
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
		// let mut sub = std::fs::read_dir(path.as_ref());

		let is_allowed = |entry: &DirEntry| {
			entry.file_type().is_dir() && !entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
		};

		WalkDir::new(root).follow_links(true)
		                  .into_iter()
		                  .filter_entry(is_allowed)
		                  .filter_map(|e| e.ok())
		                  .for_each(|entry| {
			                  let sub = entry.path();
			                  self.search_path.push(sub.into());
			                  trace!("added search dir: {}", sub.display());

			                  // add found files for secondary search level
			                  if let Ok(dir) = std::fs::read_dir(entry.path()) {
				                  dir.filter_map(|e| e.ok())
				                     .filter(|f| matches!(f.path().extension().map(|s| s.to_str()).flatten(), Some("move")))
				                     .for_each(|f| self.files_secondary.push(f.path()));
			                  }
			                  // trace!("added bins:\n{:#?}", self.files_secondary);
		                  });
	}

	pub fn add_search_file<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		trace!("added dep bin: {}", path.as_ref().display());
		self.files_primary.push(path.into());
	}
}


// let move_binary = |entry: &DirEntry| {
// 	// self.files_secondary.push(entry)
// 	// entry.file_type().is_dir() && !entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
// 	let ext = entry.path().extension();
// 	let is = ext.is_some() && ext.unwrap() == "move";
// 	// if is {
// 	// 	self.files_secondary.push(entry.into_path());
// 	// }
// 	is
// };
