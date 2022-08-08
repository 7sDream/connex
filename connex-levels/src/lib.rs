#![warn(clippy::all)]
#![warn(missing_docs, missing_debug_implementations)]
#![deny(warnings)]
#![forbid(unsafe_code)]
#![no_std]

//! Connex Levels
//!
//! This create contains levels of connex game, in string format.
//!
//! Use [`connex::World::from_str`] to compile it to real game world.

/// Connex levels.
pub const LEVELS: &[&str] = include!(concat!(env!("OUT_DIR"), "/levels.rs"));
