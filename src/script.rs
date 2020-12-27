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

use std::{
	fs,
	ffi::{OsStr, OsString},
	path::PathBuf,
	process::Command,
	sync::RwLock,
};
use structopt::StructOpt;
use simple_logger::SimpleLogger;
use anyhow::Result;
use regex::Regex;

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
	#[structopt(name = "OUTPUT", parse(from_os_str))]
	output: Option<OsString>,

	/// Quality for gifski.
	#[structopt(short, long, default_value = "100")]
	quality: u32,

	/// fps for gifski.
	#[structopt(short, long)]
	fps: Option<f32>,
}

fn main() -> Result<()> {
	SimpleLogger::new().init().unwrap();
	let opt: Opt = Opt::from_args();
	{ *VERBOSE.write().unwrap() = opt.verbose; }

	let file_name = opt.input.file_stem().expect("No input file specified.");
	verbose!("input: {}", &opt.input.display());
	verbose!("output: {}", if let Some(o) = &opt.output { format!("{:?}", &o) } else { format!("No output specified, using {:?}", file_name) });

	let mut frames_dir = std::env::temp_dir();
	frames_dir.push(PathBuf::from("frames"));
	verbose!("Frames directory: {}", &frames_dir.display());
	fs::remove_dir_all(&frames_dir)?;
	fs::create_dir(&frames_dir)?;
	verbose!("Created frames directory.");

	let output = parse_output(opt.input.clone(), &opt.output, &file_name)?;
	verbose!("Output: {}", &output.display());

	println!("============[ffmpeg]============");
	let ffmpeg_stderr = ffmpeg_command(&opt.input, &frames_dir)?;
	let fps = if let Some(f) = opt.fps { f } else { parse_fps(&ffmpeg_stderr)? };
	println!("============[gifski]============");
	gifski_command(opt.quality, fps, &frames_dir, output)?;
	println!("============[Cleaning Up]============");
	fs::remove_dir_all(&frames_dir)?;
	verbose!("Deleted frames directory: {}.", if frames_dir.exists() { "failed" } else { "success" });
	println!("============[Complete!]============");

	Ok(())
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
	println!("Frame conversion complete");
	Ok(stderr.to_string())
}

/// gifski -o file.gif frame*.png
fn gifski_command(mut quality: u32, mut frames: f32, frames_dir: &PathBuf, output: PathBuf) -> Result<()> {
	println!("Running gifski. This might take a while.");
	frames = frames.clamp(0.0, 50.0);
	quality = quality.clamp(0, 100);
	println!("fps: {}, quality: {}", &frames, &quality);

	let command = Command::new("gifski")
		.arg("--fps").arg(frames.to_string())
		.arg("--quality").arg(quality.to_string())
		.arg("-o").arg(output.into_os_string())
		.arg(format!("{}/frame*.png", &frames_dir.display()))
		.output()
		.expect("Failed to run the gifski command. Make sure you have gifski and it is accessible.");

	verbose!("stdout: {}", String::from_utf8_lossy(&command.stdout));
	verbose!("stderr: {}", String::from_utf8_lossy(&command.stderr));
	if !command.status.success() { anyhow::bail!("Command executed with failing error code: {:#?}", command.status.code().unwrap()); }
	println!("gifski complete");
	Ok(())
}

fn parse_fps(ffmpeg_stderr: &String) -> Result<f32> {
	let re = Regex::new(r"(\d+(\.\d+)?)\s(fps)").unwrap();
	let video_fps = re.captures(ffmpeg_stderr).unwrap()[1].parse()?;
	verbose!("Original Video FPS: {}", &video_fps);
	Ok(video_fps)
}

fn parse_output(input: PathBuf, output: &Option<OsString>, file_name: &OsStr) -> Result<PathBuf> {
	let mut curr = input.parent().unwrap_or(&input).to_owned();
	return if let Some(s) = output {
		if s.clone().to_string_lossy().contains('/') {
			// ./some/path.gif
			Ok(PathBuf::from(s))
		} else {
			if PathBuf::from(&s).extension().is_some() {
				// output.gif
				curr.push(s);
				Ok(curr)
			} else {
				// output
				curr.push(s);
				curr.set_extension("gif");
				Ok(curr)
			}
		}
	} else {
		// none
		let mut name = file_name.to_os_string();
		name.push("-gifski");
		curr.push(name);
		curr.set_extension("gif");
		Ok(curr)
	};
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
