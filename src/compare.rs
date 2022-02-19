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
use std::io::ErrorKind;
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
                ErrorKind::NotFound => Ok(FileCmp {
                    file: None,
                    metadata: None,
                }),
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
    fn modified(&self) -> Option<SystemTime> {
        self.metadata
            .as_ref()
            .map(|m| m.modified().expect("get file modified time"))
    }

    pub fn partial_cmp_diff(&self, other: &Self) -> Option<Ordering> {
        match (&self.file, &other.file) {
            (Some(fa), Some(fb)) => self.partial_cmp_diff_files(fa, fb),
            _ => self.partial_cmp(other),
        }
    }

    pub fn partial_cmp_diff_files(&self, fa: &File, fb: &File)
        -> Option<Ordering>
    {
        None
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

