use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpUrlError {
    #[error("Invalid URL format")]
    InvalidHttpUrl(#[from] url::ParseError),

    #[error("Only http/https scheme is supported")]
    SchemeNotSupported
}


#[derive(Debug, Clone)]
pub struct HttpUrl(url::Url);

impl HttpUrl {
    pub fn new(url: &str) -> Result<Self, HttpUrlError> {
        if !Self::is_http_or_https_scheme(url) {
            return Err(HttpUrlError::SchemeNotSupported);
        }

        let url = url::Url::parse(url)?;
        Ok(Self(url))
    }

    fn is_http_or_https_scheme(input: &str) -> bool {
        input.starts_with("http") || input.starts_with("https")
    }
}

impl Display for HttpUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Deref for HttpUrl {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for HttpUrl {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(HttpUrl(url::Url::parse(s)?))
    }
}
