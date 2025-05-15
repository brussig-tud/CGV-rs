
//////
//
// Imports
//

// Standard library
use std::{fs, fmt::Display};

// Anyhow crate
use anyhow;

// Reqwest crate
use reqwest;

// Zip-extract crate
pub use zip_extract as zip;



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



//////
//
// Functions
//

/// Request from the given URL and return the full response body as a sequence of bytes.
pub fn download (url: impl reqwest::IntoUrl) -> anyhow::Result<bytes::Bytes> {
	let dlResponse = reqwest::blocking::get(url.as_str())?;
	if dlResponse.status() != reqwest::StatusCode::OK {
		return Err(HttpResponseNotOkError::new(url.as_str(), dlResponse).into())
	}
	Ok(dlResponse.bytes()?)
}

/// Request from the given URL and store the response body in the given file.
pub fn downloadToFile (url: impl reqwest::IntoUrl, filepath: impl AsRef<crate::Path>) -> anyhow::Result<()> {
	let responseBytes = download(url)?;
	Ok(fs::write(filepath.as_ref(), responseBytes)?)
}

/// Request an archive file from the given URL and extract its contents (without the root/parent directory if the
/// archive contains one) to the given path.
pub fn downloadAndExtract (url: impl reqwest::IntoUrl, dirpath: impl AsRef<crate::Path>) -> anyhow::Result<()> {
	let responseBytes = download(url)?;
	Ok(zip::extract(std::io::Cursor::new(responseBytes), dirpath.as_ref(), true)?)
}

///
pub fn dependOnDownloadedFile (url: impl reqwest::IntoUrl, filepath: impl AsRef<crate::Path>) -> anyhow::Result<()> {
	downloadToFile(url, filepath.as_ref())?;
	println!("cargo:rerun-if-changed={}", filepath.as_ref().display());
	Ok(())
}

///
pub fn dependOnDownloadedDirectory (url: impl reqwest::IntoUrl, dirpath: impl AsRef<crate::Path>) -> anyhow::Result<()> {
	downloadAndExtract(url, dirpath.as_ref())?;
	println!("cargo:rerun-if-changed={}", dirpath.as_ref().display());
	Ok(())
}
