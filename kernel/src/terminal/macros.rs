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
         $vis:vis enum $name:ident<'a> {
            $(
                $(#[$meta:meta])* // Capture des attributs des variants (comme /// commentaires)
                $variant:ident $( {
                    $(#[$field_meta:meta])*
                    $($field:ident : $type:ty),* $(,)?
                } )?
            ),* $(,)?
        }
    ) => {
        use $crate::terminal::verbosity::Verbosity;

        $(#[$enum_meta])*
        $vis enum $name<'a> {
            $(
                $(#[$meta])*
                $variant {
                    #[arg(short = "v", value_name = "level", default_value_t = None)]
                    /// Change verbosity
                    verbosity: Option<Verbosity>,
                    $($($field : $type),*)?
                },
            )*
        }

        impl<'a> $name<'a> {
            #[must_use]
            pub fn get_verbosity(&self) -> &Option<Verbosity> {
                match self {
                    $(Self::$variant { verbosity, .. } => verbosity),*
                }
            }
        }
    };
}
