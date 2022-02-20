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
use anyhow::anyhow;
use clap::Parser;
use either::Either;

use std::cmp::Ordering;


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

    let mut paths_iter = if opts.reverse {
        Either::Right(opts.paths.iter().enumerate().rev())
    } else {
        Either::Left(opts.paths.iter().enumerate())
    };

    let mut max_idx = 0;
    let mut prev_file_cmp: Option<FileCmp> = None;

    while let Some((idx, p)) = paths_iter.next() {
        let curr = match FileCmp::try_from(p.as_path()) {
            Ok(file_cmp) if !file_cmp.is_found() => match opts.missing {
                MissingBehavior::Error => return Err(
                    anyhow!("file '{}' not found", p.display())
                ),

                MissingBehavior::Ignore => Some(file_cmp),
            },
            Ok(file_cmp) => Some(file_cmp),
            Err(e) => return Err(e.into()),
        };

        match (prev_file_cmp.as_ref(), curr) {
            (Some(prev), Some(curr)) => {
                let cmp = if opts.diff {
                    prev.partial_cmp_diff(&curr)
                } else {
                    prev.partial_cmp(&curr)
                };
                if let Some(Ordering::Greater) = cmp {
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

    // Print the result and exit.
    if opts.index {
        println!("{}", max_idx);
    } else {
        println!("{}", opts.paths[max_idx].display());
    }
    Ok(())
}



