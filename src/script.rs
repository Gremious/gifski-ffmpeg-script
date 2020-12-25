//! ```cargo
//! [dependencies]
//! structopt = "0.3"
//! log = "0.4"
//! simple_logger = "1.11"
//! ```

#![warn(
clippy::all,
clippy::pedantic,
)]

use std::{fs, path::PathBuf};
use structopt::StructOpt;
use simple_logger::SimpleLogger;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
	/// Verbose mode.
	#[structopt(short, long)]
	verbose: bool,

	/// File to process.
	#[structopt(name = "INPUT", parse(from_os_str))]
	input: PathBuf,

	/// Output file.
	#[structopt(name = "OUTPUT", parse(from_os_str))]
	output: PathBuf,
}

fn main() {
	SimpleLogger::new().init().unwrap();
	let opt: Opt = Opt::from_args();
	let input = fs::read_to_string(&opt.input).expect("Failed reading the input file.");
	verbose!(opt, "output: {:#?}", &input);

	let output = fs::read_to_string(&opt.input).expect("Failed reading the output file.");
	verbose!(opt, "output_dir {:#?}", &output);

	fs::write(output, "Hello, world!").expect("Unable to write file");
	println!("Hello, world!");
}

#[macro_export]
macro_rules! verbose {
    ($opt:ident, $target:literal, $($arg:tt)+) => {
   		if $opt.verbose { log::info!($target, $($arg)+); }
    };
}
