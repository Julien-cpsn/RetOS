use core::str::FromStr;
use embedded_cli::arguments::FromArgumentError;
use smoltcp::wire::{IpAddress, IpCidr};

pub struct IpAddressArg(pub IpAddress);

impl<'a> embedded_cli::arguments::FromArgument<'a> for IpAddressArg {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> where Self: Sized {
        match IpAddress::from_str(arg) {
            Ok(ip_address) => Ok(IpAddressArg(ip_address)),
            Err(_) => Err(FromArgumentError {
                value: arg,
                expected: "IPv4 or IPv6 address",
            })
        }
    }
}

pub struct IpCidrArg(pub IpCidr);

impl<'a> embedded_cli::arguments::FromArgument<'a> for IpCidrArg {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> where Self: Sized {
        match IpCidr::from_str(arg) {
            Ok(ip_address) => Ok(IpCidrArg(ip_address)),
            Err(_) => Err(FromArgumentError {
                value: arg,
                expected: "IPv4 or IPv6 address with subnet mask (Cidr)",
            })
        }
    }
}