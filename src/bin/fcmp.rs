////////////////////////////////////////////////////////////////////////////////
// Fcmp file compare utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////
#![warn(anonymous_parameters)]
#![warn(bad_style)]
#![warn(bare_trait_objects)]
#![warn(const_err)]
#![warn(dead_code)]
#![warn(elided_lifetimes_in_paths)]
#![warn(improper_ctypes)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_doc_code_examples)]
#![warn(missing_docs)]
#![warn(no_mangle_generic_items)]
#![warn(non_shorthand_field_patterns)]
#![warn(nonstandard_style)]
#![warn(overflowing_literals)]
#![warn(path_statements)]
#![warn(patterns_in_fns_without_body)]
#![warn(private_in_public)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unconditional_recursion)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(unused_allocation)]
#![warn(unused_comparisons)]
#![warn(unused_parens)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
#![warn(while_true)]

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



