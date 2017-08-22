#![allow(unknown_lints, unused_doc_comment)]
use super::xmlfmt::error as xmlfmt;

error_chain!{
    links {
        XmlFormat(xmlfmt::Error, xmlfmt::ErrorKind);
    }
}
