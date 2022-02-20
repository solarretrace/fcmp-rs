////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use fcmp::command::FcmpOptions;

// External library imports.
use clap::Parser;
use anyhow::Error;


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
    // println!("{:?}", opts);

    // Exit early if no paths to compare.
    if opts.paths.is_empty() { return Ok(()); }

    let idx = fcmp::compare_all(
        opts.paths.iter().map(|p| p.as_path()),
        opts.reverse,
        opts.diff,
        opts.missing)?;

    // Print the result and exit.
    if opts.index {
        println!("{}", idx);
    } else {
        println!("{}", opts.paths[idx].display());
    }
    Ok(())
}



