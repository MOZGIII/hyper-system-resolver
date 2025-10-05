//! System DNS resolver for [`hyper`].
//!
//! Resolves the name via `getaddrinfo`, but more flexible than [`hyper`]
//! standard resolver.
//!
//! ## Usage
//!
//!    ```
//!    # #[cfg(feature = "addr-info-hints")] {
//!    use hyper_system_resolver::{addr_info_hints, AddrInfoHints};
//!    use hyper_util::client::legacy::{Client, connect::HttpConnector};
//!
//!    let addr_info_hints = AddrInfoHints {
//!         address_family: addr_info_hints::AddressFamily::Inet6,
//!    };
//!    let system_resolve = hyper_system_resolver::system::System {
//!        addr_info_hints: Some(addr_info_hints.into()),
//!        service: None,
//!    };
//!    let executor = hyper_util::rt::TokioExecutor::new();
//!    let http_connector = HttpConnector::new_with_resolver(system_resolve.resolver());
//!    let client = Client::builder(executor).build::<_, hyper::body::Incoming>(http_connector);
//!    # }
//!    ```

#![warn(missing_docs, clippy::all)]

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate derive_builder;

#[cfg(feature = "addr-info-hints")]
pub mod addr_info_hints;
pub mod background;
pub mod system;

#[cfg(feature = "addr-info-hints")]
pub use addr_info_hints::AddrInfoHints;
