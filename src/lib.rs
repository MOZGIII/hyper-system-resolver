//! System DNS resolver for [`hyper`].
//!
//! Resolves the name via `getaddrinfo`, but more flexible than [`hyper`]
//! standard resolver.

#![warn(missing_docs, clippy::all)]

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate derivative;

pub mod addr_info_hints;
pub mod background;
pub mod system;

pub use addr_info_hints::AddrInfoHints;
