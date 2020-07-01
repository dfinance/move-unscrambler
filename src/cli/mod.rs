use std::path::PathBuf;
use anyhow::{format_err, Result};
pub use clap::Clap;

mod logging;


#[derive(Clap, Debug)]
#[clap(name = "Move Describer")]
pub struct Opts {
	// /// Sets a custom config file. Could have been an Option<T> with no default too
	// #[clap(short, long, default_value = "~/.somedir/default.toml")]
	// config: String,
	//
	//
	#[clap(flatten)]
	pub input: Input,

	#[clap(flatten)]
	pub output: Output,

	#[clap(flatten)]
	log: logging::Opts,
}


#[derive(Clap, Debug)]
pub struct Output {
	/// Output target directory.
	/// Prints to stdout by default.
	#[clap(short = "o", long = "output", name = "output directory")]
	// TODO: add default_value = "--" and support output to stdout.
	pub dir: PathBuf,

	/// Forces override files in the output directory.
	#[clap(short, long)]
	force: bool,

	/// Sets format of output document.
	#[clap(long = "fmt", possible_values = &[OutputFmt::MARKDOWN, OutputFmt::HTML], default_value = OutputFmt::DEFAULT)]
	format: OutputFmt,

	/// Inlines assets into the output document.
	/// Default value is false, but forced to true if output setted up to stdout (--).
	#[clap(long, name = "inline assets")]
	inline: bool,
}


#[derive(Clap, Debug)]
enum OutputFmt {
	Markdown,
	Html,
}

impl OutputFmt {
	const DEFAULT: &'static str = Self::MARKDOWN;
	const MARKDOWN: &'static str = "md";
	const HTML: &'static str = "html";
}

impl std::str::FromStr for OutputFmt {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			Self::MARKDOWN => Ok(OutputFmt::Markdown),
			Self::HTML => Ok(OutputFmt::Html),
			_ => Err(format!("Unsupported output format '{}'", s)),
		}
	}
}


#[derive(Clap, Debug)]
pub struct Input {
	#[clap(flatten)]
	pub online: InputNet,

	#[clap(flatten)]
	pub offline: InputFs,

	/// Sets Move implementation.
	#[clap(long, possible_values = &Dialect::ALL_OPTIONS, default_value = Dialect::DEFAULT)]
	pub dialect: Dialect,
}

#[derive(Clap, Debug)]
pub struct InputFs {
	// TODO: /// Input file path of 0x-address
	/// Input bytecode-file path.
	#[clap(short = "i", long = "input", name = "input file")]
	pub path: PathBuf,

	/// Sets type for main input bytecode-file to script or module.
	#[clap(long, possible_values = &MoveKind::ALL_OPTIONS, default_value = MoveKind::DEFAULT)]
	pub kind: MoveKind,

	/// Dependencies search directory path.
	/// Can be used multiple times.
	#[clap(short, long, name = "directory or file")]
	pub dependencies: Vec<PathBuf>,

	/// Enables recursive search dependencies.
	#[clap(short = "r", long = "recursive")]
	pub search_recursive: bool,

	/// Follow symbolic links when search dependencies.
	#[clap(long = "follow-symlinks")]
	pub follow_symlinks: bool,
}

#[derive(Clap, Debug)]
pub struct InputNet {
	/// Disables any online requests such as
	/// resolving dependencies by shared node.
	#[clap(long, name = "offline mode")]
	pub offline: bool,

	/// Sets URI to shared data-source.
	/// Used to resolve dependencies online by shared node.
	/// Can be disabled by pass --offline flag.
	#[clap(long = "data-source", name = "URI")]
	pub ds: Option<String /* TODO: use http::Uri */>,
}


#[derive(Clap, Debug)]
pub enum MoveKind {
	Script,
	Module,
	Auto,
}

impl MoveKind {
	const DEFAULT: &'static str = Self::AUTO;
	const SCRIPT: &'static str = "script";
	const MODULE: &'static str = "module";
	const AUTO: &'static str = "auto";
	const ALL_OPTIONS: [&'static str; 3] = [Self::AUTO, Self::SCRIPT, Self::MODULE];
}

impl std::str::FromStr for MoveKind {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			Self::AUTO => Ok(Self::Auto),
			Self::SCRIPT => Ok(Self::Script),
			Self::MODULE => Ok(Self::Module),
			_ => Err(format!("Unsupported input type '{}'", s)),
		}
	}
}


#[derive(Clap, Copy, Clone, Debug)]
pub enum Dialect {
	Dfinance,
	Libra,
}

impl Dialect {
	// TODO: set DEFAULT to Self::DFINANCE.
	const DEFAULT: &'static str = Self::LIBRA;
	const DFINANCE: &'static str = "dfi";
	const LIBRA: &'static str = "libra";
	const ALL_OPTIONS: [&'static str; 2] = [Self::DFINANCE, Self::LIBRA];
}

impl std::str::FromStr for Dialect {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			Self::DFINANCE => Ok(Self::Dfinance),
			Self::LIBRA => Ok(Self::Libra),
			_ => Err(format!("Unsupported output format '{}'", s)),
		}
	}
}


#[allow(dead_code)]
pub fn try_init() -> Result<Opts> {
	let opts: Opts = Opts::try_parse().map_err(|err| anyhow!("{}", err))?;
	logging::try_init(opts.log.verbose).map_err(|err| anyhow!("{}", err))?;
	Ok(opts)
}

#[allow(dead_code)]
pub fn init() -> Opts {
	let opts: Opts = Opts::parse();
	logging::init(opts.log.verbose);
	opts
}
