use std::path::Path;

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
	use std::env::current_dir;
	let p: &Path = path.as_ref();
	if p.is_absolute() {
		current_dir().and_then(|pb| p.strip_prefix(pb).or_else(|_| Ok(p)))
		             .unwrap()
		             .display()
		             .to_string()
	} else {
		p.display().to_string()
	}
}
