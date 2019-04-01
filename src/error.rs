#![allow(unknown_lints, unused_doc_comments)]
pub use super::xmlfmt::error::{Error as FmtError, ErrorKind as FmtErrorKind};

error_chain! {
    links {
        XmlFormat(FmtError, FmtErrorKind);
    }
    errors {
        BindFail(details: String) {
            description("Failed to bind XML-RPC server to port")
            display("Failed to bind XML-RPC server to port: {}", details)
        }
    }
}
