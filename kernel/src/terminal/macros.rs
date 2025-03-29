#[macro_export]
/// Generates Cli compatible enum argument
macro_rules! arg_from_enum {
    ($arg:ty) => {
        use embedded_cli::arguments::{FromArgumentError,FromArgument};
        use core::str::FromStr;
        use alloc::string::String;
        use paste::paste;
        use spin::Lazy;

        paste! {
            pub struct [<$arg Arg>]($arg);

            static [<$arg:snake:upper _ARG_ERROR>]: Lazy<String> = Lazy::new(|| $arg::VARIANTS.join("|"));

            impl<'a> FromArgument<'a> for [<$arg Arg>] {
                fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> where Self: Sized {
                    match $arg::from_str(arg) {
                        Ok(data) => Ok([<$arg Arg>](data)),
                        Err(_) => Err(FromArgumentError {
                            value: arg,
                            expected: &[<$arg:snake:upper _ARG_ERROR>],
                        })
                    }
                }
            }
        }
    };
}

#[macro_export]
/// Add a verbosity argument to every enum variant
macro_rules! add_verbosity {
    (
        $(#[$enum_meta:meta])*
         $vis:vis enum $name:ident$(<$lifetime:lifetime>)? {
            $(
                $(#[$meta:meta])*
                $variant:ident $( {
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident : $field_type:ty
                    ),* $(,)?
                } )?
            ),* $(,)?
        }
    ) => {
        use $crate::terminal::arguments::verbosity::Verbosity;

        $(#[$enum_meta])*
        $vis enum $name$(<$lifetime>)? {
            $(
                $(#[$meta])*
                $variant {
                    #[arg(short = "v", value_name = "level", default_value_t = None)]
                    /// Change verbosity
                    verbosity: Option<Verbosity>,
                    $(
                        $(
                            $(#[$field_meta])*
                            $field : $field_type
                        ),*
                    )?
                },
            )*
        }

        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            /// Fixme: recursive verbosity
            pub fn get_verbosity(&self) -> &Option<Verbosity> {
                match self {
                    $(Self::$variant { verbosity, .. } => verbosity),*
                }
            }
        }
    };
}

#[macro_export]
/// Add a verbosity argument to every enum subcommand
macro_rules! add_group_verbosity {
    (
        $(#[$enum_meta:meta])*
         $vis:vis enum $name:ident$(<$lifetime:lifetime>)? {
            $(
                $(#[$meta:meta])*
                $variant:ident($field_type:ty)
            ),* $(,)?
        }
    ) => {
        use $crate::terminal::arguments::verbosity::Verbosity;

        $(#[$enum_meta])*
        $vis enum $name$(<$lifetime>)? {
            $(
                $(#[$meta])*
                $variant(Option<$field_type>),
            )*
        }

        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            pub fn get_verbosity(&self) -> &Option<Verbosity> {
                match self {
                    $(Self::$variant(subcommand) => match subcommand {
                        None => &None,
                        Some(variant) => variant.get_verbosity(),
                    }),*
                }
            }
        }
    };
}