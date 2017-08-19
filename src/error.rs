use super::xmlfmt::error as xmlfmt;

error_chain!{
    links {
        XmlFormat(xmlfmt::Error, xmlfmt::ErrorKind);
    }
}
