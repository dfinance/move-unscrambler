pub use clap::Clap;
use std::path::PathBuf;

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
	// TODO: /// Input file path of 0x-address
	/// Input bytecode-file path.
	#[clap(short = "i", long = "input", name = "input file")]
	pub path: PathBuf,

	/// Dependencies search directory path.
	/// Can be used multiple times.
	#[clap(short, long, name = "directory or file")]
	pub dependencies: Vec<PathBuf>,

	/// Resolves dependencies online by shared node.
	#[clap(long, name = "online mode")]
	pub online: bool,

	/// Sets URI to shared data-source. Used if online mode enabled.
	#[clap(long = "online-node")]
	pub ds: Option<String /* TODO: use http::Uri */>,

	/// Sets Move implementation.
	#[clap(long, possible_values = &[Dialect::DFINANCE, Dialect::LIBRA], default_value = Dialect::DEFAULT)]
	dialect: Dialect,
}


#[derive(Clap, Debug)]
enum Dialect {
	Dfinance,
	Libra,
}

impl Dialect {
	// TODO: set DEFAULT to Self::DFINANCE.
	const DEFAULT: &'static str = Self::LIBRA;
	const DFINANCE: &'static str = "dfi";
	const LIBRA: &'static str = "libra";
}

impl std::str::FromStr for Dialect {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			Self::DFINANCE => Ok(Dialect::Dfinance),
			Self::LIBRA => Ok(Dialect::Libra),
			_ => Err(format!("Unsupported output format '{}'", s)),
		}
	}
}


pub fn try_init() -> Result<Opts, String> {
	let opts: Opts = Opts::try_parse().map_err(|err| format!("{}", err))?;
	logging::try_init(opts.log.verbose).map_err(|err| format!("{}", err))?;
	Ok(opts)
}

#[allow(dead_code)]
pub fn init() -> Opts {
	let opts: Opts = Opts::parse();
	logging::init(opts.log.verbose);
	opts
}
