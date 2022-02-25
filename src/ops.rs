////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File compare operations.
////////////////////////////////////////////////////////////////////////////////


// External library imports.
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use serde::Deserialize;

// Standard library imports.
use std::path::Path;
use std::io::BufRead as _;
use std::io::BufReader;
use std::io::ErrorKind;
use std::process::Command;
use std::ops::Not;
use std::fs::File;


/// A diff operation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum DiffOp {
	/// No diff will be performed.
	None,
	
	/// An internal diff will be used.
	Internal,

	/// A diff command will be run as a subprocess.
	Subprocess {
		/// The command to execute.
		command: &'static str,
		/// The arguments to pass to it.
		args: Vec<&'static str>,
	},
}


impl DiffOp {
	/// Returns a `DiffOp` that will execute a POSIX diff subprocess.
	pub fn posix_diff() -> DiffOp {
		DiffOp::Subprocess {
			command: "diff",
			args: vec![],
		}
	}

	/// Returns a `DiffOp` that will execute a POSIX cmp subprocess.
	pub fn posix_cmp() -> DiffOp {
		DiffOp::Subprocess {
			command: "cmp",
			args: vec!["-s"],
		}
	}
	

	/// Returns true if the files at the given paths are different.
	pub fn diff(&self, a: &Path, b: &Path) -> Result<bool, std::io::Error> {
		match self {
			DiffOp::None => Ok(a != b),

			DiffOp::Internal => {
				let file_a = match File::options().read(true).open(a) {
					Ok(f)  => Some(f),
					Err(e) if matches!(e.kind(), ErrorKind::NotFound) => None,
					Err(e) => return Err(e),
				};

				let file_b = match File::options().read(true).open(b) {
					Ok(f)  => Some(f),
					Err(e) if matches!(e.kind(), ErrorKind::NotFound) => None,
					Err(e) => return Err(e),
				};

				match (file_a, file_b) {
					(Some(a), Some(b)) => {
						let meta_a = a.metadata().expect("get file metadata");
						let meta_b = b.metadata().expect("get file metadata");

						if meta_a.len() != meta_b.len()
							|| meta_a.is_symlink()
							|| meta_b.is_symlink()
							|| meta_a.file_type() != meta_b.file_type()
						{
							 Ok(false)
						} else {
							DiffOp::internal_eq(&a, &b)
								.map(bool::not)
						}
					},

					(None, None) => Ok(true),
					_            => Ok(false),
				}
			},

			DiffOp::Subprocess { command, args } => {
				let status = Command::new(command)
					.args(args)
					.arg(a)
					.arg(b)
					.status()?;

				match status.code() {
					Some(0) => Ok(false),
					Some(1) => Ok(true),
					Some(_) => Err(std::io::Error::from(ErrorKind::Other)),
					None => Err(std::io::Error::from(ErrorKind::Interrupted)),
				}
			},
			
		}
	}

	/// Returns `true` if the given files have the same content.
	///
	/// ### Errors
	///
	/// Returns a [`std::io::Error`] if the file's contents fail to read
	/// correctly.
	///
	/// [`std::io::Error`]: std::io::Error
	fn internal_eq(a: &File, b: &File) -> Result<bool, std::io::Error> {
		let mut buf_reader_a = BufReader::new(a);
		let mut buf_reader_b = BufReader::new(b);

		loop {
			let buf_a = buf_reader_a.fill_buf()?;
			let buf_b = buf_reader_b.fill_buf()?;

			if buf_a.is_empty() && buf_b.is_empty() {
				return Ok(true);
			}

			let read_len = if buf_a.len() <= buf_b.len() {
				buf_a.len()
			} else {
				buf_b.len()
			};

			if buf_a[0..read_len] != buf_b[0..read_len] {
				return Ok(false);
			}

			buf_reader_a.consume(read_len);
			buf_reader_b.consume(read_len);
		}
	}
}
