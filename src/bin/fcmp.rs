////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use fcmp::command::FcmpOptions;
use fcmp::command::MissingBehavior;
use fcmp::FileCmp;

// External library imports.
// use anyhow::Context;
use anyhow::Error;
use clap::Parser;


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() {
    if let Err(err) = main_facade() {
        // Print errors to stderr and exit with error code.
        colored::control::unset_override();
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}


////////////////////////////////////////////////////////////////////////////////
// main_facade
////////////////////////////////////////////////////////////////////////////////
/// The application facade for propagating user errors.
pub fn main_facade() -> Result<(), Error> {
    // Parse command line options.
    let opts = FcmpOptions::try_parse()?;
    println!("{:?}", opts);

    // Exit early if no paths to compare.
    if opts.paths.is_empty() { return Ok(()); }

    let mut paths_iter: Box<dyn Iterator<Item=_>> = Box::new(opts.paths.iter());
    if opts.reverse {
        paths_iter = Box::new(opts.paths.iter().rev())
    }

    let mut max_idx = 0;
    let mut prev = match FileCmp::try_from(paths_iter.next().unwrap().as_path()) {
        Ok(res) => res,
        Err(e) => match opts.missing {
            MissingBehavior::Error   => return Err(e.into()),
            MissingBehavior::Ignore  => return Err(e.into()),
        },
    };


    while let Some(path) = paths_iter.next() {

    }

    Ok(())
}

