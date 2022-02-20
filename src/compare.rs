////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File compare functions.
////////////////////////////////////////////////////////////////////////////////

// Standard library imports.
use std::cmp::Ordering;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::BufReader;
use std::io::BufRead as _;
use std::path::Path;
use std::time::SystemTime;
use std::fs::Metadata;



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

    fn modified(&self) -> Option<SystemTime> {
        self.metadata
            .as_ref()
            .map(|m| m.modified().expect("get file modified time"))
    }

    pub fn partial_cmp_diff(&self, other: &Self) -> Option<Ordering> {
        if let (Some(file_a), Some(file_b))
            = (self.file.as_ref(), other.file.as_ref())
        {
            let meta_a = self.metadata.as_ref().expect("get file metadata");
            let meta_b = other.metadata.as_ref().expect("get file metadata");

            let len = meta_a.len();
            if len == meta_b.len()
                && (!meta_a.is_symlink()
                    && meta_a.file_type() == meta_b.file_type())
                && FileCmp::content_eq(file_a, file_b, len).ok()?
            {
                return Some(Ordering::Equal);
            }
        }

        self.partial_cmp(other)
    }

    fn content_eq(a: &File, b: &File, len: u64) -> Result<bool, Error> {
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

impl PartialEq for FileCmp {
    fn eq(&self, other: &Self) -> bool {
        self.file.is_some() == other.file.is_some() &&
            self.modified() == other.modified()
    }
}

impl PartialOrd for FileCmp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let file_cmp = match (&self.file, &other.file) {
            (None,    Some(_)) => Ordering::Less,
            (Some(_), None)    => Ordering::Greater,
            _                  => return None,
        };
        let time_cmp = match (&self.modified(), &other.modified()) {
            (Some(t1), Some(t2)) => t1.cmp(t2),
            (Some(_),  None)     => Ordering::Greater,
            (None,     Some(_))  => Ordering::Less,
            _                    => return None,
        };

        Some(file_cmp.then(self.modified().cmp(&other.modified())))
    }
}

