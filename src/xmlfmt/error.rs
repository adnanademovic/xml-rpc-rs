#![allow(unknown_lints, unused_doc_comments)]
use serde::{de, ser};
use std::fmt::{self, Display};

error_chain! {
    foreign_links {
        Fmt(fmt::Error);
    }

    errors {
        Decoding(t: String) {
            description("Issue while decoding data structure")
            display("Issue while decoding data structure: {}", t)
        }
        Encoding(t: String) {
            description("Issue while encoding data structure")
            display("Issue while encoding data structure: {}", t)
        }
        UnsupportedData(t: String) {
            description("Given structure is not supported")
            display("Given structure is not supported: {}", t)
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        ErrorKind::Decoding(format!("{}", msg)).into()
    }

    fn invalid_type(unexp: de::Unexpected, exp: &de::Expected) -> Self {
        Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        ErrorKind::Encoding(format!("{}", msg)).into()
    }
}
