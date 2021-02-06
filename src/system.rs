//! The system resolver implementation.

use std::{io, net::SocketAddr, vec};

use dns_lookup::AddrInfoHints;
use hyper::client::connect::dns::Name;
use tokio::task::JoinHandle;

use crate::background;

/// [`System`] encapsulates logic to perform the name resolution in
/// the background using system resolution mechanisms.
///
/// Uses [`dns_lookup::getaddrinfo`] in a [`tokio::task::spawn_blocking`] to
/// perform the resolution.
#[derive(Debug, Builder, Default, Clone)]
pub struct System {
    /// The hints to give the the system resolver when performing the
    /// resolution.
    ///
    /// Passing [`None`] is not equivalent to passing [`Some`] value filled with
    /// zeroes, as underlying systems typically have some non-trivial defaults
    /// when hint is omitted.
    pub addr_info_hints: Option<AddrInfoHints>,

    /// The name of the service to resolve.
    /// If set to [`None`], the network address of the node is resolved.
    /// If set to [`Some`], the the requested service address is resolved.
    /// This can be either a descriptive name or a numeric representation
    /// suitable for use with the address family or families.
    /// If the specified address family is AF_INET,  AF_INET6, or AF_UNSPEC,
    /// the service can be specified as a string specifying a decimal port
    /// number.
    pub service: Option<String>,
}

impl background::Resolve for System {
    type Iter = vec::IntoIter<SocketAddr>;

    fn resolve(&mut self, name: Name) -> JoinHandle<io::Result<Self::Iter>> {
        let addr_info_hints = self.addr_info_hints;
        tokio::task::spawn_blocking(move || {
            debug!("resolving host={:?}", name);

            let iter = dns_lookup::getaddrinfo(Some(name.as_str()), None, addr_info_hints)?;
            let list = iter
                .map(|result| result.map(|addr_info| addr_info.sockaddr))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(list.into_iter())
        })
    }
}

/// System resolver compatible with [`hyper`].
pub type Resolver = background::Resolver<System>;

impl System {
    /// Use this [`System`] to create a new [`hyper`]-compatible [`Resolver`].
    pub fn resolver(self) -> Resolver {
        Resolver::new(self)
    }
}

impl From<System> for Resolver {
    fn from(system: System) -> Self {
        system.resolver()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
        str::FromStr,
    };

    use tower_service::Service;

    use super::*;

    #[tokio::test]
    async fn test_resolve_ipv4() {
        let mut resolver = background::Resolver::new(System {
            addr_info_hints: Some(
                AddrInfoHints {
                    address: dns_lookup::AddrFamily::Inet.into(),
                    ..Default::default()
                }
                .into(),
            ),
            service: None,
        });

        let addrs: Vec<_> = resolver
            .call(Name::from_str("localhost").unwrap())
            .await
            .unwrap()
            .collect();

        let localhost = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0));

        assert!(addrs.len() > 0);
        for addr in addrs {
            assert_eq!(addr, localhost);
        }
    }

    #[tokio::test]
    async fn test_resolve_ipv6() {
        let mut resolver = background::Resolver::new(System {
            addr_info_hints: Some(AddrInfoHints {
                address: dns_lookup::AddrFamily::Inet6.into(),
                ..Default::default()
            }),
            service: None,
        });

        let addrs: Vec<_> = resolver
            .call(Name::from_str("localhost").unwrap())
            .await
            .unwrap()
            .collect();

        let localhost = SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
            0,
            0,
            0,
        ));

        assert!(addrs.len() > 0);
        for addr in addrs {
            assert_eq!(addr, localhost);
        }
    }
}
