use crate::devices::network::manager::NETWORK_MANAGER;
use alloc::format;
use alloc::string::{String, ToString};
use no_std_clap_core::arg::from_arg::FromArg;
use no_std_clap_core::error::ParseError;

pub struct NetworkInterfaceArg(pub String);

impl FromArg for NetworkInterfaceArg {
    fn from_arg(arg: &str) -> Result<Self, ParseError> where Self: Sized {
        let network_manager = NETWORK_MANAGER.lock();
        
        if arg == "lo" {
            return Ok(NetworkInterfaceArg(arg.to_string()));
        }
        
        for (index, _) in network_manager.interfaces.iter().enumerate() {
            if arg == format!("eth{index}") {
                return Ok(NetworkInterfaceArg(arg.to_string()));
            }
        }

        Err(ParseError::InvalidValue(format!("Network interface \"{arg}\" not found")))
    }
}