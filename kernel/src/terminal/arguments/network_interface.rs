use crate::devices::network::interface::NETWORK_INTERFACES;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use embedded_cli::arguments::FromArgumentError;

pub struct NetworkInterfaceArg<'a>(pub &'a str);

impl<'a> embedded_cli::arguments::FromArgument<'a> for NetworkInterfaceArg<'a> {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> where Self: Sized {
        let network_interfaces = NETWORK_INTERFACES.write();
        match network_interfaces.contains_key(arg) {
            true => Ok(NetworkInterfaceArg(arg)),
            false => {
                let known_interfaces = network_interfaces.keys().cloned().collect::<Vec<String>>().join(", ");

                Err(FromArgumentError {
                    value: arg,
                    expected: format!("Network interface not found. Known interfaces are: {known_interfaces}").leak(),
                })
            }
        }
    }
}