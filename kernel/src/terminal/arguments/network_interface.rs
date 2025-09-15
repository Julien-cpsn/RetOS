use crate::devices::network::manager::NETWORK_MANAGER;
use alloc::format;
use embedded_cli::arguments::FromArgumentError;

pub struct NetworkInterfaceArg<'a>(pub &'a str);

impl<'a> embedded_cli::arguments::FromArgument<'a> for NetworkInterfaceArg<'a> {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> where Self: Sized {
        let network_manager = NETWORK_MANAGER.lock();
        
        if arg == "lo" {
            return Ok(NetworkInterfaceArg(arg));
        }
        
        for (index, _) in network_manager.interfaces.iter().enumerate() {
            if arg == format!("eth{index}") {
                return Ok(NetworkInterfaceArg(arg));
            }
        }

        Err(FromArgumentError {
            value: arg,
            expected: "Network interface not found",
        })
    }
}