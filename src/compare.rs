////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File compare functions.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::ops::DiffOp;

// External library imports.
use anyhow::anyhow;

// Standard library imports.
use std::cmp::Ordering;
use std::fs::File;
use std::fs::Metadata;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
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
    path: PathBuf,
    file: Option<File>,
    metadata: Option<Metadata>,
}

impl TryFrom<PathBuf> for FileCmp {
    type Error = std::io::Error;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        match File::options()
            .read(true)
            .open(&path)
        {
            Ok(file) => Ok(Self {
                path,
                metadata: Some(file.metadata()?),
                file: Some(file),
            }),

            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(Self::not_found(path)),
                _ => Err(e),
            },
        }
    }
}


impl FileCmp {
    /// Returns a file comparer which behaves like a non-existent file.
    #[must_use]
    pub fn not_found(path: PathBuf) -> Self {
        Self {
            path,
            file: None,
            metadata: None,
        }
    }

    /// Returns `true` if the file has been found.
    #[must_use]
    pub fn is_found(&self) -> bool {
        self.file.is_some()
    }

    /// Returns the modification time of the wrapped file, if it can be
    /// determined. This is equivalent to a call to [`Metadata::modified`].
    ///
    /// [`Metadata::modified`]: std::fs::Metadata::modified
    #[must_use]
    fn modified(&self) -> Option<SystemTime> {
        self.metadata
            .as_ref()
            .map(|m| m.modified().expect("get file modified time"))
    }

    /// Returns an ordering between the given `FileCmp`s based on their
    /// modification times, if such an ordering exists.
    ///
    /// ### Parameters
    /// + `other`: The other `FileCmp` to compare to.
    /// + `diff_op`: The `DiffOp` to compare file differences. If the files do
    /// not differ, they will compare equal regardless of their modification
    /// times. 
    /// + `promote_newest`: If true, indicates that missing files should be
    /// considered greater than other files. Otherwise, they are considered less
    /// than other files.
    #[must_use]
    pub fn partial_cmp(
        &self,
        other: &Self,
        diff_op: &DiffOp,
        promote_newest: bool)
        -> Option<Ordering>
    {
        use Ordering::*;

        if let Ok(false) = diff_op
            .diff(self.path.as_path(), other.path.as_path())
        {
            return Some(Equal);
        }

        let file_cmp = match (&self.file, &other.file) {
            (Some(_), Some(_)) => Equal,
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
            Ok(Self::Oldest)
        } else if s.eq_ignore_ascii_case("newest") {
            Ok(Self::Newest)
        } else if s.eq_ignore_ascii_case("ignore") {
            Ok(Self::Ignore)
        } else if s.eq_ignore_ascii_case("error") {
            Ok(Self::Error)
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
// compare
////////////////////////////////////////////////////////////////////////////////
/// Returns the ordering of two files based on their modification times. The
/// order is a partial order, and as such, None will be returned if the file
/// modification times cannot be determined.
///
///
/// ### Parameters
/// 
/// + `diff`: Whether to consider files with equivalent content to be equal.
/// + `missing`: The [`MissingFileBehavior`] indicating how to handle missing
/// files.
/// 
/// ### Errors
///
/// Returns an error if `MissingFileBehavior::Error` is used and a provided
/// file is missing, or if reading the file results in an unexpected IO error.
///
/// [`Path`]: std::path::Path
/// [`MissingFileBehavior`]: MissingFileBehavior
pub fn partial_cmp_paths(
    a: &Path,
    b: &Path,
    diff_op: &DiffOp,
    missing: MissingFileBehavior)
    -> Result<Option<Ordering>, anyhow::Error>
{
    let promote_newest = matches!(missing, MissingFileBehavior::Newest);

    // Check if they're the same paths.
    if a == b { return Ok(Some(Ordering::Equal)); }

    let a = match FileCmp::try_from(a.to_path_buf()) {
        Ok(file_cmp) if !file_cmp.is_found() => match missing {
            MissingFileBehavior::Error => return Err(
                anyhow!("file '{}' not found", a.display())
            ),

            MissingFileBehavior::Ignore => None,
            _ => Some(file_cmp),
        },
        Ok(file_cmp) => Some(file_cmp),
        Err(e) => return Err(e.into()),
    };

    let b = match FileCmp::try_from(b.to_path_buf()) {
        Ok(file_cmp) if !file_cmp.is_found() => match missing {
            MissingFileBehavior::Error => return Err(
                anyhow!("file '{}' not found", b.display())
            ),

            MissingFileBehavior::Ignore => None,
            _ => Some(file_cmp),
        },
        Ok(file_cmp) => Some(file_cmp),
        Err(e) => return Err(e.into()),
    };

    let ordering = match (a, b) {
        (Some(a), Some(b)) => a.partial_cmp(&b, diff_op, promote_newest),
        (None, None) => Some(Ordering::Equal),
        (None,    _) => Some(Ordering::Greater),
        (_,    None) => Some(Ordering::Less),
    };

    Ok(ordering)
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
/// 
/// + `reverse`: Whether to reverse to comparison order and return the least
/// recently modified file.
/// + `diff`: Whether to consider files with equivalent content to be equal.
/// + `missing`: The [`MissingFileBehavior`] indicating how to handle missing
/// files.
/// 
/// ### Errors
///
/// Returns an error if `MissingFileBehavior::Error` is used and a provided
/// file is missing, or if reading the file results in an unexpected IO error.
///
/// [`Path`]: std::path::Path
/// [`MissingFileBehavior`]: MissingFileBehavior
pub fn compare_all<'p, P>(
    paths: P,
    reverse: bool,
    diff_op: &DiffOp,
    missing: MissingFileBehavior)
    -> Result<usize, anyhow::Error>
    where P: IntoIterator<Item=&'p Path>
{
    let promote_newest = matches!(missing, MissingFileBehavior::Newest);

    let mut max_idx = 0;
    let mut prev_file_cmp: Option<FileCmp> = None;

    for (idx, p) in paths.into_iter().enumerate() {
        let curr = match FileCmp::try_from(p.to_path_buf()) {
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
                let cmp = prev.partial_cmp(&curr, diff_op, promote_newest)
                    .map(|o| if reverse { o } else { o.reverse() });
                if cmp == Some(Ordering::Greater) {
                    prev_file_cmp = Some(curr);
                    max_idx = idx;
                }
            },
            (None, curr) => {
                prev_file_cmp = curr;
                max_idx = idx;
            },
            _ => (),
        }
    }

    Ok(max_idx)
}
