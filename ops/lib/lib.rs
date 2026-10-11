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
//! Every format is a cargo feature, so this example walks the ones the
//! build claims: write a forest, read it back, and get the same tasks.
//!
//! ```
//! use ops_lib::v0::{self, Format, Status, Task};
//!
//! let tasks = vec![Task::new("buy milk".to_string(), Status::Open)];
//! for &format in Format::ALL {
//!     let document = v0::write(format, &tasks)?;
//!     assert_eq!(v0::read(format, &document)?, tasks);
//! }
//! # Ok::<(), ops_lib::v0::Error>(())
//! ```

pub mod v0;

mod shared;
