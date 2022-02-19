////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface flags.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use clap::Parser;

// Standard library imports.
use std::path::PathBuf;
use std::str::FromStr;
use std::error::Error;


////////////////////////////////////////////////////////////////////////////////
// FcmpOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options. 
#[derive(Debug, Clone)]
#[derive(Parser)]
#[clap(name = "fcmp")]
#[clap(author, version, about, long_about = None)]
pub struct FcmpOptions {
    /// File paths to compare.
    #[clap(parse(from_os_str))]
    pub paths: Vec<PathBuf>,

    /// Reverse the compare.
    #[clap(
        short = 'r',
        long = "reverse")]
    pub reverse: bool,

    /// Return the index of the relevant file, rather than the path.
    #[clap(
        short = 'i',
        long = "index")]
    pub index: bool,

    /// Consider files with the same content as equal.
    #[clap(
        short = 'd',
        long = "diff")]
    pub diff: bool,

    /// Ignore missing files.
    #[clap(
        short = 'm',
        long = "missing",
        arg_enum)]
    pub missing: MissingBehavior,

    // TODO: Directory comparisons?
}



#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(clap::ArgEnum)]
pub enum MissingBehavior {
    Error,
    Ignore,
    Promote,
}

impl FromStr for MissingBehavior {
    type Err = MissingBehaviorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("error") {
            Ok(MissingBehavior::Error)
        } else if s.eq_ignore_ascii_case("ignore") {
            Ok(MissingBehavior::Ignore)
        } else if s.eq_ignore_ascii_case("promote") {
            Ok(MissingBehavior::Promote)
        } else {
            Err(MissingBehaviorParseError)
        }
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingBehaviorParseError;

impl Error for MissingBehaviorParseError {}

impl std::fmt::Display for MissingBehaviorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing argument to option --missing")
    }
}
