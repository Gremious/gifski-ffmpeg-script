//! ```cargo
//! [dependencies]
//! structopt = "0.3"
//! log = "0.4"
//! simple_logger = "1.11"
//! anyhow = "1"
//! lazy_static = "1"
//! regex = "1"
//! ```
#![feature(clamp)]

#![warn(
clippy::all,
clippy::pedantic,
)]

use std::{fs, path::PathBuf, process::Command, sync::RwLock};
use structopt::StructOpt;
use simple_logger::SimpleLogger;
use anyhow::{Error, Result, bail, ensure};
use regex::Regex;
use std::borrow::Cow;

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
	#[structopt(short, long)]
	fps: Option<u32>,
}

fn main() -> Result<()> {
	SimpleLogger::new().init().unwrap();
	let opt: Opt = Opt::from_args();
	{ *VERBOSE.write().unwrap() = opt.verbose; }
	let file_name = opt.input.file_stem().expect("No input file specified.");
	verbose!("input: {}", &opt.input.display());
	verbose!("output: {}", if let Some(o) = &opt.output { o.clone() } else { format!("No output specified, using {:?}", file_name) });
	let mut frames_dir = opt.input.clone();
	frames_dir.pop();
	frames_dir.push(PathBuf::from("frames"));
	verbose!("frames dir: {}", &frames_dir.display());

	fs::create_dir(&frames_dir);
	verbose!("Created the frames dir");

	println!("==============================");
	let ffmpeg_stderr = ffmpeg_command(&opt.input, &frames_dir)?;
	let fps = if let Some(f) = opt.fps { f } else { parse_fps(&ffmpeg_stderr)? };
	println!("==============================");
	gifski_command(opt.quality, fps, &frames_dir)?;
	println!("==============================");

	Ok(())
}

fn parse_fps(ffmpeg_stderr: &String) -> Result<u32> {
	let re = Regex::new(r"(\d+(\.\d+)?)\s(fps)").unwrap();
	let video_fps = re.captures(ffmpeg_stderr).unwrap()[1].parse()?;
	verbose!("Original Video FPS: {}", &video_fps);
	Ok(video_fps)
}

/// ffmpeg -i video.mp4 frame%04d.png
fn ffmpeg_command(input: &PathBuf, frames_dir: &PathBuf) -> Result<String> {
	println!("Splitting video into frames.");
	let command = Command::new("ffmpeg")
		.arg("-i").arg(format!("{}", &input.display()))
		.arg(format!("{}/frame%04d.png", &frames_dir.display()))
		.output()
		.expect("Failed to run the ffmpeg command. Make sure you have ffmpeg and it is accessible.");

	verbose!("stdout: {}", String::from_utf8_lossy(&command.stdout));
	let stderr = String::from_utf8_lossy(&command.stderr);
	verbose!("stderr: {}", &stderr);

	if !command.status.success() { anyhow::bail!("Command executed with failing error code: {:#?}", command.status.code().unwrap()); }
	log::info!("Frame conversion complete");
	Ok(stderr.to_string())
}

/// gifski -o file.gif frame*.png
fn gifski_command(mut quality: u32, mut frames: u32, frames_dir: &PathBuf) -> Result<()> {
	log::info!("Running gifski. This might take a while.");
	frames = frames.clamp(0, 50);
	quality = quality.clamp(0, 100);
	verbose!("fps: {}, quality: {}", &frames, &quality);

	let command = Command::new("./gifski.exe")
		.arg("--fps").arg(frames.to_string())
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
    ($target:literal) => {
   		{ if *VERBOSE.read().unwrap() { log::info!($target); } }
    };

     ($target:literal, $($arg:tt)+) => {
   		{ if *VERBOSE.read().unwrap() { log::info!($target, $($arg)+); } }
    };
}
