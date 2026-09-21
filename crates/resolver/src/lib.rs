//! Input resolution boundaries for Nexum.

use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveKind {
    Http,
    Https,
    Magnet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveRequest {
    source: String,
}

impl ResolveRequest {
    pub fn new(source: impl Into<String>) -> Result<Self, ResolverError> {
        let source = source.into();
        if source.trim().is_empty() {
            return Err(ResolverError::EmptySource);
        }
        Ok(Self { source })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn kind(&self) -> Result<ResolveKind, ResolverError> {
        if self.source.starts_with("http://") {
            Ok(ResolveKind::Http)
        } else if self.source.starts_with("https://") {
            Ok(ResolveKind::Https)
        } else if self.source.starts_with("magnet:?") {
            Ok(ResolveKind::Magnet)
        } else {
            Err(ResolverError::UnsupportedScheme(self.scheme()))
        }
    }

    fn scheme(&self) -> String {
        self.source
            .split_once(':')
            .map(|(scheme, _)| scheme.to_ascii_lowercase())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveResult {
    pub kind: ResolveKind,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolverError {
    EmptySource,
    UnsupportedScheme(String),
    InvalidSource(String),
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySource => f.write_str("source is empty"),
            Self::UnsupportedScheme(scheme) => write!(f, "unsupported source scheme: {scheme}"),
            Self::InvalidSource(message) => write!(f, "invalid source: {message}"),
        }
    }
}

impl std::error::Error for ResolverError {}

pub trait Resolver {
    fn resolve(&self, request: &ResolveRequest) -> Result<ResolveResult, ResolverError>;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultResolver;

impl Resolver for DefaultResolver {
    fn resolve(&self, request: &ResolveRequest) -> Result<ResolveResult, ResolverError> {
        let kind = request.kind()?;
        match kind {
            ResolveKind::Http | ResolveKind::Https => {
                if request.source().len() <= request.source().find("://").unwrap_or(0) + 3 {
                    return Err(ResolverError::InvalidSource(
                        "HTTP source is missing a host".into(),
                    ));
                }
            }
            ResolveKind::Magnet => {
                if request.source().len() <= "magnet:?".len() {
                    return Err(ResolverError::InvalidSource(
                        "magnet source is missing parameters".into(),
                    ));
                }
            }
        }
        Ok(ResolveResult { kind, source: request.source().to_owned() })
    }
}

impl FromStr for ResolveRequest {
    type Err = ResolverError;
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::new(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_http_https_and_magnet() {
        let resolver = DefaultResolver;
        for (source, kind) in [
            ("http://example.com/file", ResolveKind::Http),
            ("https://example.com/file", ResolveKind::Https),
            ("magnet:?xt=urn:btih:abc", ResolveKind::Magnet),
        ] {
            let request = ResolveRequest::new(source).unwrap();
            assert_eq!(resolver.resolve(&request).unwrap().kind, kind);
        }
    }

    #[test]
    fn rejects_empty_and_unsupported_sources() {
        assert_eq!(ResolveRequest::new(""), Err(ResolverError::EmptySource));
        let request = ResolveRequest::new("ftp://example.com/file").unwrap();
        assert_eq!(
            request.kind(),
            Err(ResolverError::UnsupportedScheme("ftp".into()))
        );
    }

    #[test]
    fn rejects_invalid_http_and_magnet_sources() {
        let resolver = DefaultResolver;
        let http = ResolveRequest::new("https://").unwrap();
        assert!(matches!(
            resolver.resolve(&http),
            Err(ResolverError::InvalidSource(_))
        ));
        let magnet = ResolveRequest::new("magnet:?").unwrap();
        assert!(matches!(
            resolver.resolve(&magnet),
            Err(ResolverError::InvalidSource(_))
        ));
    }
}
