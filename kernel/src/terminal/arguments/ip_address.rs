use embedded_cli::arguments::FromArgumentError;
use goolog::log::LevelFilter;

pub struct Verbosity {
    pub level: LevelFilter
}

impl<'a> embedded_cli::arguments::FromArgument<'a> for Verbosity {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> where Self: Sized {
        let level = match arg {
            "" => LevelFilter::Warn,
            "v" => LevelFilter::Info,
            "vv" => LevelFilter::Debug,
            "vvv" => LevelFilter::Trace,
            "q" => LevelFilter::Off,
            _ => return Err(FromArgumentError {
                value: arg,
                expected: "v|vv|vvv|vvvv|q",
            })
        };

        Ok(Self {
            level,
        })
    }
}