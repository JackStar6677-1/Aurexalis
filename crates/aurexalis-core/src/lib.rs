//! Core shared types for Aurexalis services.
//!
//! This crate intentionally stays dependency-light. It defines request metadata
//! and errors used by higher-level modules before they are wired into Gecko.

#![forbid(unsafe_code)]

use std::fmt;

#[derive(Debug)]
pub enum AurexalisError {
    InvalidUrl(String),
    Unsupported(&'static str),
    Io(std::io::Error),
}

impl fmt::Display for AurexalisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AurexalisError::InvalidUrl(value) => write!(formatter, "invalid url: {value}"),
            AurexalisError::Unsupported(value) => write!(formatter, "unsupported operation: {value}"),
            AurexalisError::Io(error) => write!(formatter, "io error: {error}"),
        }
    }
}

impl std::error::Error for AurexalisError {}

impl From<std::io::Error> for AurexalisError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRequest {
    pub url: String,
    pub host: String,
    pub source_url: Option<String>,
    pub source_host: Option<String>,
    pub kind: ResourceKind,
}

impl NetworkRequest {
    pub fn parse(url: &str, source_url: Option<&str>, kind: ResourceKind) -> Result<Self> {
        let host = extract_host(url).ok_or_else(|| AurexalisError::InvalidUrl(url.to_owned()))?;
        let parsed_source = match source_url {
            Some(value) => Some(value.to_owned()),
            None => None,
        };
        let source_host = match source_url {
            Some(value) => {
                Some(extract_host(value).ok_or_else(|| AurexalisError::InvalidUrl(value.to_owned()))?)
            }
            None => None,
        };

        Ok(Self {
            url: url.to_owned(),
            host,
            source_url: parsed_source,
            source_host,
            kind,
        })
    }

    pub fn is_third_party(&self) -> bool {
        let Some(source_host) = &self.source_host else {
            return false;
        };

        self.host != *source_host
    }

    pub fn host(&self) -> Option<&str> {
        Some(&self.host)
    }
}

fn extract_host(value: &str) -> Option<String> {
    let (_, rest) = value.split_once("://")?;
    let authority = rest.split('/').next()?.split('?').next()?.split('#').next()?;
    let host_port = authority.rsplit('@').next()?;
    let host = host_port.split(':').next()?.trim().to_ascii_lowercase();

    if host.is_empty() || host.contains(' ') {
        return None;
    }

    Some(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_network_request_with_source() {
        let request = NetworkRequest::parse(
            "https://cdn.example.net/app.js",
            Some("https://example.com"),
            ResourceKind::Script,
        )
        .expect("request should parse");

        assert_eq!(request.host(), Some("cdn.example.net"));
        assert_eq!(request.kind, ResourceKind::Script);
        assert!(request.is_third_party());
    }

    #[test]
    fn rejects_invalid_url() {
        let error = NetworkRequest::parse("not a url", None, ResourceKind::Other)
            .expect_err("invalid URL should fail");

        assert!(matches!(error, AurexalisError::InvalidUrl(_)));
    }

    #[test]
    fn displays_resource_kind() {
        assert_eq!(ResourceKind::Stylesheet.to_string(), "stylesheet");
        assert_eq!(ResourceKind::Xhr.to_string(), "xhr");
    }

    #[test]
    fn extracts_lowercase_host() {
        let request = NetworkRequest::parse(
            "https://User:Pass@CDN.Example.NET:443/app.js",
            None,
            ResourceKind::Script,
        )
        .expect("request should parse");

        assert_eq!(request.host(), Some("cdn.example.net"));
    }
}
