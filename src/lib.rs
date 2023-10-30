//! A library for working with recurring Gitlab Issues.

#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_docs,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::nursery)]

mod issue;
mod template;

pub use issue::Issue;
