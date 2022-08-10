#![warn(clippy::all)]
#![warn(missing_docs, missing_debug_implementations)]
#![deny(warnings)]
#![forbid(unsafe_code)]
#![no_std]

//! # Connex
//!
//! Base library for connex gameplay logic.

mod block;
mod world;

extern crate alloc;

pub use block::{Block, Direction};
pub use world::World;
