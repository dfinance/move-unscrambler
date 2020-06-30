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
		// self.search_path.push(root.clone());
		// let mut sub = std::fs::read_dir(path.as_ref());

		let is_allowed = |entry: &DirEntry| {
			entry.file_type().is_dir() && !entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
		};

		let mut towalk: Vec<PathBuf> = vec![root];
		while let Some(current) = towalk.pop() {
			for entry in WalkDir::new(current.clone()).follow_links(true)
			                                          .into_iter()
			                                          .filter_entry(is_allowed)
			                                          .filter_map(|e| e.ok())
			{
				let sub = entry.path();
				if current != sub {
					trace!("added search dir: {}", sub.display());
					let buf: PathBuf = sub.into();
					self.search_path.push(buf.clone());
					towalk.push(buf);
				}
			}
		}
	}

	pub fn add_search_file<T>(&mut self, path: T)
		where T: Into<PathBuf> + AsRef<Path> {
		trace!("added dep bin: {}", path.as_ref().display());
		self.files_primary.push(path.into());
	}
}
