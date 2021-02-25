//! Background resolver.
//!
//! This module contains utilities to aid with implementing resolvers that run
//! in the background in a [`tokio`] thread pool.
//!
//! You can provide a custom [`Resolve`] trait implementation that uses either
//! [`tokio::spawn`], [`tokio::task::spawn_blocking`] or anything else that
//! returns [`tokio::task::JoinHandle`].

use std::{
    fmt,
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    task::{self, Poll},
};

use hyper::client::connect::dns::Name;
use tokio::task::JoinHandle;
use tower_service::Service;

/// Resolve the name in the background.
pub trait Resolve {
    /// An iterator type used to enumerate the resolved addresses.
    type Iter: Iterator<Item = SocketAddr>;

    /// Perform the name resolution.
    fn resolve(&mut self, name: Name) -> JoinHandle<io::Result<Self::Iter>>;
}

/// A [`hyper`]-compatible resolver implementation.
/// Delegates the actual resolution logic to the generic parameter `T`.
#[derive(Clone)]
pub struct Resolver<T> {
    inner: T,
}

impl<T> Resolver<T> {
    /// Create a new [`Resolver`] backed by `inner` resolution implementation.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Consume [`Resolver`] and return the wrapped `T`.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> AsRef<T> for Resolver<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> Service<Name> for Resolver<T>
where
    T: Resolve + Send + Sync + 'static,
{
    type Response = <T as Resolve>::Iter;
    type Error = io::Error;
    type Future = ResolverFuture<<T as Resolve>::Iter>;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let handle = self.inner.resolve(name);
        ResolverFuture { inner: handle }
    }
}

impl<T> fmt::Debug for Resolver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Resolver")
    }
}

/// The opaque resolver future.
///
/// Ready when the underlying background resolution is ready.
/// Propagates panics from the underlying task.
pub struct ResolverFuture<T> {
    inner: JoinHandle<io::Result<T>>,
}

impl<T> Future for ResolverFuture<T> {
    type Output = io::Result<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|res| match res {
            Ok(Ok(addrs)) => Ok(addrs),
            Ok(Err(err)) => Err(err),
            Err(join_err) => {
                if join_err.is_cancelled() {
                    Err(io::Error::new(io::ErrorKind::Interrupted, join_err))
                } else {
                    panic!("resolver Resolver task failed: {:?}", join_err)
                }
            }
        })
    }
}

impl<T> fmt::Debug for ResolverFuture<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("ResolverFuture")
    }
}
