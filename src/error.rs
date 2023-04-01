use thiserror::Error;

#[derive(Error, Debug)]
pub enum XmlRpcError {
    #[error("Issue while encoding data structure: {0}")]
    Encoding(String),
    #[error("Issue while decoding data structure: {0}")]
    Decoding(String),
    #[error("Given structure is not supported: {0}")]
    UnsupportedData(String),
    #[error("Invalid type: {0}")]
    InvalidType(String),
    #[error("Failed to bind XML-RPC server to port: {0}")]
    BindFail(String),
    #[error("Failed to run the HTTP request within ureq.")]
    Ureq(Box<ureq::Error>),
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse XML data")]
    Xml(#[from] serde_xml_rs::Error),
}

impl From<ureq::Error> for XmlRpcError {
    fn from(value: ureq::Error) -> Self {
        XmlRpcError::Ureq(Box::new(value))
    }
}

pub type XmlRpcResult<T> = std::result::Result<T, XmlRpcError>;

impl serde::de::Error for XmlRpcError {
    fn custom<T: std::fmt::Display>(msg: T) -> XmlRpcError {
        XmlRpcError::Decoding(format!("{}", msg))
    }

    fn invalid_type(unexp: serde::de::Unexpected, exp: &dyn serde::de::Expected) -> Self {
        XmlRpcError::InvalidType(format!("invalid type: {}, expected {}", unexp, exp))
    }
}

impl serde::ser::Error for XmlRpcError {
    fn custom<T: std::fmt::Display>(msg: T) -> XmlRpcError {
        XmlRpcError::Encoding(format!("{}", msg))
    }
}
