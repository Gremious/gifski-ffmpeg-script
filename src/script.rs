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

	/// Quality for gifski.
	#[structopt(short, long, default_value = "100")]
	quality: u32,

	/// fps for gifski.
	#[structopt(short, long, default_value = "50")]
	fps : u32,
}

fn main() -> Result<()> {
	SimpleLogger::new().init().unwrap();
	let opt: Opt = Opt::from_args();
	{ *VERBOSE.write().unwrap() = opt.verbose; }
	let file_name =  opt.input.file_stem()?;
	verbose!("input: {}", &opt.input.display());
	verbose!("output: {}", if let Some(o) = &opt.output { o.display() } else { format!("No output specified, using {}", file_name) });
	let mut frames_dir = opt.input.clone();
	frames_dir.pop();
	frames_dir.push(PathBuf::from("frames"));
	verbose!("frames dir: {}", &frames_dir.display());

	fs::create_dir(&frames_dir);
	verbose!("Created the frames dir");

	println!("==============================");
	ffmpeg_command(&opt.input, &frames_dir);
	println!("==============================");
	gifski_command(opt.quality, &frames_dir);
	println!("==============================");

	Ok(())
}

/// ffmpeg -i video.mp4 frame%04d.png
fn ffmpeg_command(input: &PathBuf, frames_dir: &PathBuf) -> Result<()> {
	println!("Splitting video into frames.");
	let command = Command::new("ffmpeg")
		.arg("-i").arg(format!("{}", &input.display()))
		.arg(format!("{}/frame%04d.png", &frames_dir.display()))
		.output()
		.expect("Failed to run the ffmpeg command. Make sure you have ffmpeg and it is accessible.");

	verbose!("stdout: {}", String::from_utf8_lossy(&command.stdout));
	verbose!("stderr: {}", String::from_utf8_lossy(&command.stderr));
	if !command.status.success() { anyhow::bail!("Command executed with failing error code: {:#?}", command.status.code().unwrap()); }
	log::info!("Frame conversion complete");
	Ok(())
}

/// gifski -o file.gif frame*.png
fn gifski_command(quality: u32, frames_dir: &PathBuf) -> Result<()> {
	log::info!("Running gifski. This might take a while.");
	let command = Command::new("./gifski.exe")
		.arg("--fps").arg("33")
		.arg("--quality").arg(quality.to_string())
		.arg("-o").arg("./output.gif")
		.arg(format!("{}/frame*.png", &frames_dir.display()))
		.output()
		.expect("Failed to run the gifski command. Make sure you have gifski and it is accessible.");

	verbose!("stdout: {}", String::from_utf8_lossy(&command.stdout));
	verbose!("stderr: {}", String::from_utf8_lossy(&command.stderr));
	if !command.status.success() { anyhow::bail!("Command executed with failing error code: {:#?}", command.status.code().unwrap()); }
	log::info!("gifski complete");
	Ok(())
}

#[macro_export]
macro_rules! verbose {
    ($target:literal, $($arg:tt)+) => {
   		{ if *VERBOSE.read().unwrap() { log::info!($target, $($arg)+); } }
    };
}
