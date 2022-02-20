////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface flags.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::MissingBehavior;

// External library imports.
use clap::Parser;

// Standard library imports.
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// FcmpOptions
////////////////////////////////////////////////////////////////////////////////
/// Takes a list of file names and returns the most recently modified file.
/// 
/// If the result would be ambiguous, the first occurring ambiguous item in the
/// file list will be returned.
#[derive(Debug, Clone)]
#[derive(Parser)]
#[clap(name = "fcmp")]
#[clap(author, version)]
pub struct FcmpOptions {
    /// File paths to compare.
    #[clap(parse(from_os_str))]
    pub paths: Vec<PathBuf>,

    /// Return the oldest file instead of the newest.
    #[clap(
        short = 'r',
        long = "reverse")]
    pub reverse: bool,

    /// Return the (0-based) index of the file instead of the path.
    #[clap(
        short = 'i',
        long = "index")]
    pub index: bool,

    /// Ignore files that have the same content.
    #[clap(
        short = 'd',
        long = "diff")]
    pub diff: bool,

    /// Determines how to handle missing files.
    /// 
    /// By default, missing files will be treated as older than all other files.
    #[clap(
        short = 'm',
        long = "missing",
        default_value = "oldest",
        arg_enum)]
    pub missing: MissingBehavior,
}


