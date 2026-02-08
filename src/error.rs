use std::fmt;

macro_rules! define_errors {
    (
        $(wrap $variant:ident($ty:ty) => $label:expr),*;
        $(cfg_wrap [$cfg_attr:meta] $cfg_variant:ident($cfg_ty:ty) => $cfg_label:expr),*;
        $(msg $msg_variant:ident => $msg_label:expr),*
        $(;)?
    ) => {
        #[derive(Debug)]
        pub enum MgdlError {
            $($variant($ty),)*
            $(
                #[$cfg_attr]
                $cfg_variant($cfg_ty),
            )*
            $($msg_variant(String),)*
        }

        impl fmt::Display for MgdlError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(MgdlError::$variant(err) => write!(f, "{}: {}", $label, err),)*
                    $(
                        #[$cfg_attr]
                        MgdlError::$cfg_variant(err) => write!(f, "{}: {}", $cfg_label, err),
                    )*
                    $(MgdlError::$msg_variant(msg) => write!(f, "{}: {}", $msg_label, msg),)*
                }
            }
        }

        $(impl From<$ty> for MgdlError {
            fn from(err: $ty) -> Self {
                MgdlError::$variant(err)
            }
        })*

        $(
            #[$cfg_attr]
            impl From<$cfg_ty> for MgdlError {
                fn from(err: $cfg_ty) -> Self {
                    MgdlError::$cfg_variant(err)
                }
            }
        )*
    };
}

define_errors! {
    wrap Io(std::io::Error)             => "Io error",
    wrap Toml(toml::de::Error)          => "Toml error",
    wrap Reqwest(reqwest::Error)        => "Reqwest error",
    wrap Rusqlite(rusqlite::Error)      => "Rusqlite error",
    wrap Parse(std::num::ParseIntError) => "Parse error",
    wrap Join(tokio::task::JoinError)   => "Join error";
    cfg_wrap [cfg(feature = "bench")] Csv(csv::Error) => "CSV error";
    msg Config     => "Config error",
    msg Db         => "DB error",
    msg Scrape     => "Scrape error",
    msg Logger     => "Logger error"
}

impl std::error::Error for MgdlError {}

pub type MgdlResult<T> = std::result::Result<T, MgdlError>;
