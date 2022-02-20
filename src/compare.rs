////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File compare functions.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use anyhow::anyhow;

// Standard library imports.
use std::cmp::Ordering;
use std::fs::File;
use std::fs::Metadata;
use std::io::BufRead as _;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;


////////////////////////////////////////////////////////////////////////////////
// FileCmp
////////////////////////////////////////////////////////////////////////////////
/// A [`File`] wrapper which provides methods for doing file comparisons.
/// operations.
///
/// [`File`]: std::fs::File
#[derive(Debug)]
pub struct FileCmp {
    file: Option<File>,
    metadata: Option<Metadata>,
}

impl TryFrom<&Path> for FileCmp {
    type Error = std::io::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        match File::options()
            .read(true)
            .open(path)
        {
            Ok(file) => FileCmp::try_from(file),

            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(FileCmp::not_found()),
                _ => Err(e),
            },
        }
    }
}

impl TryFrom<File> for FileCmp {
    type Error = std::io::Error;
    fn try_from(file: File) -> Result<Self, Self::Error> {
        Ok(FileCmp {
            metadata: Some(file.metadata()?),
            file: Some(file),
        })
    }
}

impl FileCmp {
    /// Returns a file comparer which behaves like a non-existent file.
    pub fn not_found() -> Self {
        FileCmp {
            file: None,
            metadata: None,
        }
    }

    /// Returns `true` if the file has been found.
    pub fn is_found(&self) -> bool {
        self.file.is_some()
    }

    /// Returns the modification time of the wrapped file, if it can be
    /// determined. This is equivalent to a call to [`Metadata::modified`].
    ///
    /// [`Metadata::modified`]: std::fs::Metadata::modified
    fn modified(&self) -> Option<SystemTime> {
        self.metadata
            .as_ref()
            .map(|m| m.modified().expect("get file modified time"))
    }

    /// Returns an ordering between the given `FileCmp`s based on their
    /// modification times, if such an ordering exists.
    ///
    /// ### Parameters
    /// + `promote_newest`: If true, indicates that missing files should be
    /// considered greater than other files. Otherwise, they are considered less
    /// than other files.
    pub fn partial_cmp(&self, other: &Self, promote_newest: bool)
        -> Option<Ordering>
    {
        use Ordering::*;

        let file_cmp = match (&self.file, &other.file) {
            (None,    Some(_)) => if promote_newest { Greater } else { Less },
            (Some(_), None)    => if promote_newest { Less } else { Greater },
            _                  => return None,
        };
        let time_cmp = match (&self.modified(), &other.modified()) {
            (Some(t1), Some(t2)) => t1.cmp(t2),
            (None,    Some(_))   => if promote_newest { Greater } else { Less },
            (Some(_), None)      => if promote_newest { Less } else { Greater },
            _                    => return None,
        };

        Some(file_cmp.then(time_cmp))
    }

    /// Returns an ordering between the given `FileCmp`s based on their
    /// modification times, if such an ordering exists. This method will first
    /// determine if two files' content differs, and if not, they will be
    /// treated as equal regardless of their modification times.
    ///
    /// ### Parameters
    /// + `promote_newest`: If true, indicates that missing files should be
    /// considered greater than other files. Otherwise, they are considered less
    /// than other files.
    pub fn partial_cmp_diff(&self, other: &Self, promote_newest: bool)
        -> Option<Ordering>
    {
        if let (Some(file_a), Some(file_b))
            = (self.file.as_ref(), other.file.as_ref())
        {
            let meta_a = self.metadata.as_ref().expect("get file metadata");
            let meta_b = other.metadata.as_ref().expect("get file metadata");

            let len = meta_a.len();
            if len == meta_b.len()
                && (!meta_a.is_symlink()
                    && meta_a.file_type() == meta_b.file_type())
                && FileCmp::content_eq(file_a, file_b).ok()?
            {
                return Some(Ordering::Equal);
            }
        }

        self.partial_cmp(other, promote_newest)
    }

    /// Returns `true` if the given files have the same content.
    ///
    /// ### Errors
    ///
    /// Returns a [`std::io::Error`] if the file's contents fail to read
    /// correctly.
    ///
    /// [`std::io::Error`]: std::io::Error
    fn content_eq(a: &File, b: &File) -> Result<bool, std::io::Error> {
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

            if &buf_a[0..read_len] != &buf_b[0..read_len] {
                return Ok(false);
            }

            buf_reader_a.consume(read_len);
            buf_reader_b.consume(read_len);
        }
    }
}





////////////////////////////////////////////////////////////////////////////////
// MissingFileBehavior
////////////////////////////////////////////////////////////////////////////////
/// Options for handling missing files.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(clap::ArgEnum)]
pub enum MissingFileBehavior {
    /// Treat missing files as older than all others.
    Oldest,
    /// Treat missing files as newer than all others.
    Newest,
    /// Ignore the file if it is missing.
    Ignore,
    /// Return an error if the file is missing.
    Error,
}

impl FromStr for MissingFileBehavior {
    type Err = MissingFileBehaviorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("oldest") {
            Ok(MissingFileBehavior::Oldest)
        } else if s.eq_ignore_ascii_case("newest") {
            Ok(MissingFileBehavior::Newest)
        } else if s.eq_ignore_ascii_case("ignore") {
            Ok(MissingFileBehavior::Ignore)
        } else if s.eq_ignore_ascii_case("error") {
            Ok(MissingFileBehavior::Error)
        } else {
            Err(MissingFileBehaviorParseError)
        }
    }
}

/// An error indicating a failure to parse a [`MissingFileBehavior`].
///
/// [`MissingFileBehavior`]: MissingFileBehavior 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingFileBehaviorParseError;

impl std::error::Error for MissingFileBehaviorParseError {}

impl std::fmt::Display for MissingFileBehaviorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failure to parse MissingFileBehavior")
    }
}



////////////////////////////////////////////////////////////////////////////////
// compare_all
////////////////////////////////////////////////////////////////////////////////
/// Takes an iterator of [`Path`]s and returns the index of the most recently
/// modified file.
/// 
/// If the result would be ambiguous, the first occurring ambiguous item in the
/// list will be returned.
///
///
/// ### Parameters
/// + `reverse`: Whether to reverse to comparison order and return the least
/// recently modified file.
/// + `diff`: Whether to consider files with equivalent content to be equal.
/// + `missing`: The [`MissingFileBehavior`] indicating how to handle missing
/// files.
///    
/// [`Path`]: std::path::Path
/// [`MissingFileBehavior`]: MissingFileBehavior
pub fn compare_all<'p, P>(
    paths: P,
    reverse: bool,
    diff: bool,
    missing: MissingFileBehavior)
    -> Result<usize, anyhow::Error>
    where P: IntoIterator<Item=&'p Path>
{
    let promote_newest = matches!(missing, MissingFileBehavior::Newest);

    let mut paths_iter = paths.into_iter().enumerate();

    let mut max_idx = 0;
    let mut prev_file_cmp: Option<FileCmp> = None;

    while let Some((idx, p)) = paths_iter.next() {
        let curr = match FileCmp::try_from(p) {
            Ok(file_cmp) if !file_cmp.is_found() => match missing {
                MissingFileBehavior::Error => return Err(
                    anyhow!("file '{}' not found", p.display())
                ),

                MissingFileBehavior::Ignore => continue,
                _ => Some(file_cmp),
            },
            Ok(file_cmp) => Some(file_cmp),
            Err(e) => return Err(e.into()),
        };

        match (prev_file_cmp.as_ref(), curr) {
            (Some(prev), Some(curr)) => {
                let cmp = if diff {
                    prev.partial_cmp_diff(&curr, promote_newest)
                } else {
                    prev.partial_cmp(&curr, promote_newest)
                };
                if let Some(Ordering::Greater) = cmp
                    .map(|o| if reverse { o } else { o.reverse() })
                {
                    prev_file_cmp = Some(curr);
                    max_idx = idx;
                }
            }
            (None, curr) => {
                prev_file_cmp = curr;
                max_idx = idx;
            },
            _ => (),
        }
    }

    Ok(max_idx)
}
