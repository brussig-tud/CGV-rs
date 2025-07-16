
//////
//
// Imports
//

// Standard library
use std::{fs, fmt::Display, path::Path, time::SystemTime};

// Anyhow crate
use anyhow;
use anyhow::Result;

//  fs-set-times crate
use fs_set_times::{set_mtime, SystemTimeSpec};

// Reqwest crate
use reqwest;

// Zip-extract crate
pub use zip;

// Local imports
use crate::*;



//////
//
// Errors
//

/// A simple error indicating that a web request did not result in a `200 OK` response.
#[derive(Debug)]
pub struct HttpResponseNotOkError {
	/// The URL of the request that did not respond with `200 OK`.
	pub url: String,

	/// The full response of the request that did not respond with `200 OK`.
	pub response: reqwest::blocking::Response
}
impl HttpResponseNotOkError {
	/// Create a new instance for the given `url` and `response`.o
	pub fn new (url: impl Into<String>, response: reqwest::blocking::Response) -> Self { Self {
		url: url.into(), response
	}}
}
impl Display for HttpResponseNotOkError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "HttpResponseNotOkError[`{}`<-{}]", self.response.status(), self.url)
	}
}
impl std::error::Error for HttpResponseNotOkError {}


/// An error indicating that an external command invoked via [`std::process::Command`] failed, holding the complete
/// [output](std::process::Output) that the command produced.
#[derive(Debug)]
pub struct CommandFailedError {
	/// A short descriptive name for the command that failed.
	pub command_name: String,

	/// The full output produced by the command process during its execution.
	pub output: std::process::Output
}
impl CommandFailedError
{
	pub fn format_stdstream (formatter: &mut std::fmt::Formatter<'_>, prefix: &str, stream_buf: &[u8])
	                         -> std::fmt::Result {
		for line in String::from_utf8_lossy(stream_buf).lines() {
			writeln!(formatter, "{prefix}{line}")?;
		}
		Ok(())
	}
}
impl std::fmt::Display for CommandFailedError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(formatter, "CommandFailedError[`{}` -> {}]", self.command_name, self.output.status)?;
		Self::format_stdstream(formatter, " stdout: ", &self.output.stdout)?;
		Self::format_stdstream(formatter, " stderr: ", &self.output.stderr)
	}
}
impl std::error::Error for CommandFailedError {}



//////
//
// Functions
//

///
pub fn setTimestamp (path: impl AsRef<Path>, timepoint: SystemTime) -> Result<()>
{
	if path.as_ref().is_dir() {
		Ok(set_mtime(path, SystemTimeSpec::from(timepoint))?)
	}
	else {
		let file = fs::File::options().write(true).open(path)?;
		Ok(file.set_times(fs::FileTimes::new().set_modified(timepoint))?)
	}
}

///
pub fn setTimestampWithWarning (path: impl AsRef<Path>, timepoint: SystemTime) -> bool
{
	if let Err(err) = setTimestamp(path.as_ref(), timepoint) {
		println!(
			"cargo::warning=set_timestamp_with_warning: Failed to set timestamp for '{}': {}",
			path.as_ref().display(), err
		);
		false
	}
	else { true }
}

///
pub fn setTimestampToBeforeBuildScriptTime (path: impl AsRef<Path>) -> bool {
	setTimestampWithWarning(path, crate::getScriptStartTime())
}

///
pub fn setTimestampRecursively (path: impl AsRef<Path>, timepoint: SystemTime) -> Result<bool> {
	let mut no_problem = true;
	cgv_util::fs::doRecursively(
		path, |path, _, _| Ok(no_problem = setTimestampWithWarning(path, timepoint) && no_problem)
	)?;
	Ok(no_problem)
}

/// Request from the given URL and return the full response body as a sequence of bytes.
pub fn download (url: impl reqwest::IntoUrl) -> anyhow::Result<bytes::Bytes> {
	let dlResponse = reqwest::blocking::get(url.as_str())?;
	if dlResponse.status() != reqwest::StatusCode::OK {
		return Err(HttpResponseNotOkError::new(url.as_str(), dlResponse).into())
	}
	Ok(dlResponse.bytes()?)
}

/// Request from the given URL and store the response body in the given file.
pub fn downloadToFile (url: impl reqwest::IntoUrl, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
	let responseBytes = download(url)?;
	Ok(fs::write(filepath.as_ref(), responseBytes)?)
}

/// Request an archive file from the given URL and extract its contents (without the root/parent directory if the
/// archive contains one) to the given path.
pub fn downloadAndExtract (url: impl reqwest::IntoUrl, dirpath: impl AsRef<Path>) -> anyhow::Result<()> {
	let responseBytes = download(url)?;
	Ok(zip::ZipArchive::new(std::io::Cursor::new(responseBytes))?.extract(dirpath.as_ref())?)
}

///
pub fn dependOnCopiedFile (copySource: impl AsRef<Path>, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
	fs::copy(copySource.as_ref(), filepath.as_ref())?;
	dependOnFile(copySource);
	dependOnGeneratedFile(filepath)
}

///
pub fn dependOnDownloadedFile (url: impl reqwest::IntoUrl, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
	downloadToFile(url, &filepath)?;
	dependOnGeneratedFile(filepath)
}

///
pub fn dependOnDownloadedDirectory (url: impl reqwest::IntoUrl, dirpath: impl AsRef<Path>) -> Result<()> {
	downloadAndExtract(url, dirpath.as_ref())?;
	dependOnGeneratedDirectory(dirpath)
}

///
pub fn dependOnExtractedDirectory (archivePath: impl AsRef<crate::Path>, dirpath: impl AsRef<crate::Path>)
-> Result<()> {
	zip::ZipArchive::new(fs::File::open(archivePath.as_ref())?)?.extract(dirpath.as_ref())?;
	dependOnFile(archivePath);
	dependOnGeneratedDirectory(dirpath)
}

///
pub fn installFile (filepath: impl AsRef<Path>, targetDir: impl AsRef<Path>) -> anyhow::Result<()> {
	let targetFilepath = targetDir.as_ref().join(filepath.as_ref().file_name().unwrap());
	fs::copy(filepath.as_ref(), &targetFilepath)?;
	dependOnFile(filepath);
	dependOnGeneratedFile(targetFilepath)
}

/// Check if the given [process output](std::process::Output) resulted from a successful command. On most platforms,
/// that corresponds to an exit code of `0`.
///
/// # Arguments
///
/// * `output` – Some [process::Output](std::process::Output) to check.
/// * `command_name` – A short, descriptive name of the command that spawned the process (typically just the filename of
///                    the executable or script).
///
/// # Returns
///
/// `()` if the output indicates a successful execution, otherwise a [`CommandFailedError`] containing more details.
pub fn checkProcessOutput (output: std::process::Output, command_name: impl AsRef<str>)
-> Result<(), CommandFailedError>
{
	if !output.status.success() {
		Err(CommandFailedError{ command_name: String::from(command_name.as_ref()), output })
	}
	else {
		Ok(())
	}
}
