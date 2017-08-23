#![allow(unknown_lints, unused_doc_comment)]
pub use super::xmlfmt::error::{Error as FmtError, ErrorKind as FmtErrorKind};

error_chain!{
    links {
        XmlFormat(FmtError, FmtErrorKind);
    }
}
