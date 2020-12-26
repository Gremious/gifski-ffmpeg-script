//! ```cargo
//! [dependencies]
//! structopt = "0.3"
//! log = "0.4"
//! simple_logger = "1.11"
//! anyhow = "1"
//! lazy_static = "1"
//! ```

#![warn(
clippy::all,
clippy::pedantic,
)]

use std::{fs, path::PathBuf, process::Command, sync::RwLock};
use structopt::StructOpt;
use simple_logger::SimpleLogger;
use anyhow::{ Error, Result, bail, ensure };

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref VERBOSE: RwLock<bool> = RwLock::new(false);
}

static mut ASD: bool = false;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
	/// Verbose mode.
	#[structopt(short, long)]
	verbose: bool,

	/// File to process.
	#[structopt(name = "INPUT", parse(from_os_str))]
	input: PathBuf,

	/// Name of output file.
	#[structopt(name = "OUTPUT")]
	output: Option<String>,
}

fn main() -> Result<()> {
	SimpleLogger::new().init().unwrap();
	let opt: Opt = Opt::from_args();
	{ *VERBOSE.write().unwrap() = opt.verbose; }

	verbose!("input: {}", &opt.input.display());
	// verbose!(opt, "output: {}", &opt.output.display());

	let mut frames_dir = opt.input.clone();
	frames_dir.pop();
	frames_dir.push(PathBuf::from("frames"));
	verbose!("frames dir: {}", &frames_dir.display());
	fs::create_dir(&frames_dir);

	ffmpeg_command(&opt.input, &frames_dir);
	log::info!("ffmpeg to frame conversion complete, running gifski");
	gifski_command(&opt.input, &frames_dir);

	Ok(())
}

/// ffmpeg -i video.mp4 frame%04d.png
fn ffmpeg_command(input: &PathBuf, frames_dir: &PathBuf) -> Result<()> {
	let command = Command::new("ffmpeg")
		.arg("-i").arg(format!("{}", &input.display()))
		.arg(format!("{}/frame%04d.png", &frames_dir.display()))
		.output()?;

	verbose!("stdout: {}", String::from_utf8_lossy(&command.stdout));
	verbose!("stderr: {}", String::from_utf8_lossy(&command.stderr));
	if !command.status.success() { anyhow::bail!("Command executed with failing error code: {:#?}", command.status.code().unwrap()); }
	Ok(())
}

/// gifski -o file.gif frame*.png
fn gifski_command(input: &PathBuf, frames_dir: &PathBuf) -> Result<()> {
	let command = Command::new("./gifski.exe")
		.arg("-o").arg("output.gif")
		.arg(format!("{}/frame*.png", &frames_dir.display()))
		.output()
		.expect("gifski command not recognized.");

	verbose!("stdout: {}", String::from_utf8_lossy(&command.stdout));
	verbose!("stderr: {}", String::from_utf8_lossy(&command.stderr));
	if !command.status.success() { anyhow::bail!("Command executed with failing error code: {:#?}", command.status.code().unwrap()); }
	Ok(())
}

#[macro_export]
macro_rules! verbose {
    ($target:literal, $($arg:tt)+) => {
   		{ if *VERBOSE.read().unwrap() { log::info!($target, $($arg)+); } }
    };
}
