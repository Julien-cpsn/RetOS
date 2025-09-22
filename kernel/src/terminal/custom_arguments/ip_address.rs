use alloc::format;
use core::str::FromStr;
use no_std_clap_core::arg::from_arg::FromArg;
use no_std_clap_core::error::ParseError;
use smoltcp::wire::{IpAddress, IpCidr};

pub struct IpAddressArg(pub IpAddress);

impl FromArg for IpAddressArg {
    fn from_arg(arg: &str) -> Result<Self, ParseError> where Self: Sized {
        match IpAddress::from_str(arg) {
            Ok(ip_address) => Ok(IpAddressArg(ip_address)),
            Err(_) => Err(ParseError::InvalidValue(format!("\"{arg}\", need an IPv4 or IPv6 address")))
        }
    }
}

pub struct IpCidrArg(pub IpCidr);

impl FromArg for IpCidrArg {
    fn from_arg(arg: &str) -> Result<Self, ParseError> where Self: Sized {
        match IpCidr::from_str(arg) {
            Ok(ip_address) => Ok(IpCidrArg(ip_address)),
            Err(_) => Err(ParseError::InvalidValue(format!("\"{arg}\", need an IPv4 or IPv6 address with subnet mask (Cidr)")))
        }
    }
}