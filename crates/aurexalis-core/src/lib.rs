use std::fmt;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum AurexalisError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AurexalisError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Document,
    Script,
    Stylesheet,
    Image,
    Media,
    Font,
    Xhr,
    Other,
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ResourceKind::Document => "document",
            ResourceKind::Script => "script",
            ResourceKind::Stylesheet => "stylesheet",
            ResourceKind::Image => "image",
            ResourceKind::Media => "media",
            ResourceKind::Font => "font",
            ResourceKind::Xhr => "xhr",
            ResourceKind::Other => "other",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: Url,
    pub source_url: Option<Url>,
    pub kind: ResourceKind,
}

impl NetworkRequest {
    pub fn parse(url: &str, source_url: Option<&str>, kind: ResourceKind) -> Result<Self> {
        let parsed_url = Url::parse(url).map_err(|_| AurexalisError::InvalidUrl(url.to_owned()))?;
        let parsed_source = match source_url {
            Some(value) => Some(
                Url::parse(value).map_err(|_| AurexalisError::InvalidUrl(value.to_owned()))?,
            ),
            None => None,
        };

        Ok(Self {
            url: parsed_url,
            source_url: parsed_source,
            kind,
        })
    }
}

