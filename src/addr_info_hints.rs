use dns_lookup::AddrFamily;

#[cfg(unix)]
use libc::{AF_INET, AF_INET6, AF_UNIX, AF_UNSPEC};

#[cfg(windows)]
use winapi::shared::ws2def::{AF_INET, AF_INET6, AF_UNIX, AF_UNSPEC};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Derivative)]
#[derivative(Default)]
pub enum AddressFamily {
    #[derivative(Default)]
    Unspec,
    Unix,
    Inet,
    Inet6,
    Custom(i32),
}

impl From<AddrFamily> for AddressFamily {
    fn from(af: AddrFamily) -> Self {
        match af {
            AddrFamily::Inet => Self::Inet,
            AddrFamily::Inet6 => Self::Inet6,
            AddrFamily::Unix => Self::Unix,
        }
    }
}

/// Portable [`AddrInfoHints`] builder.
#[derive(Debug, Builder, Default)]
pub struct AddrInfoHints {
    #[builder(default)]
    /// Address family to request.
    pub address_family: AddressFamily,
}

impl AddrInfoHints {
    /// Create a new [`AddrInfoHints`] builder.
    pub fn builder() -> AddrInfoHintsBuilder {
        AddrInfoHintsBuilder::default()
    }
}

impl From<&AddrInfoHints> for dns_lookup::AddrInfoHints {
    fn from(opts: &AddrInfoHints) -> Self {
        let address = match opts.address_family {
            AddressFamily::Unspec => AF_UNSPEC,
            AddressFamily::Inet => AF_INET,
            AddressFamily::Inet6 => AF_INET6,
            AddressFamily::Unix => AF_UNIX,
            AddressFamily::Custom(val) => val,
        };
        Self {
            address,
            ..Default::default()
        }
    }
}

impl From<AddrInfoHints> for dns_lookup::AddrInfoHints {
    fn from(opts: AddrInfoHints) -> Self {
        From::from(&opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_family() {
        let cases = vec![
            (AddrInfoHints::default(), AF_UNSPEC),
            (
                AddrInfoHints {
                    address_family: AddressFamily::Unspec,
                },
                AF_UNSPEC,
            ),
            (
                AddrInfoHints {
                    address_family: AddressFamily::Inet,
                },
                AF_INET,
            ),
            (
                AddrInfoHints {
                    address_family: AddressFamily::Inet6,
                },
                AF_INET6,
            ),
            (
                AddrInfoHints {
                    address_family: AddressFamily::Unix,
                },
                AF_UNIX,
            ),
            (
                AddrInfoHints {
                    address_family: AddressFamily::Custom(123),
                },
                123,
            ),
        ];

        for (addr_info_hints, expected) in cases {
            let dns_lookup::AddrInfoHints { address, .. } = addr_info_hints.into();
            assert_eq!(address, expected);
        }
    }
}
