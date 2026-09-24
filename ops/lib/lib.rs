#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable
)]

//! Reader and writer for the Open Productivity Standard (OPS).
//!
//! Spec: <https://github.com/aricma/open-productivity-spec>
//!
//! The standard is versioned, and so is this crate: each OPS version
//! lives in its own module, and a document is read with the module for
//! the version it follows. [`v0`] is the only one today.
//!
//! ```no_run
//! use ops_lib::v0::{self, Format};
//!
//! let tasks = v0::read(Format::Json, "{\"title\": \"t\", \"status\": \"open\"}")?;
//! let json = v0::write(Format::Json, &tasks)?;
//! # Ok::<(), ops_lib::v0::Error>(())
//! ```

pub mod v0;

mod shared;
