mod async_fetch;
#[cfg(feature = "fetch-blocking")]
pub mod blocking;

pub use async_fetch::*;

use std::fmt;

pub const CELESTRAK_SW_LAST5YEARS_URL: &str = "https://celestrak.org/SpaceData/SW-Last5Years.csv";
pub const CELESTRAK_SW_ALL_URL: &str = "https://celestrak.org/SpaceData/SW-All.csv";
pub const SET_SOLFSMY_URL: &str = "https://sol.spacenvironment.net/JB2008/indices/SOLFSMY.TXT";
pub const SET_DTCFILE_URL: &str = "https://sol.spacenvironment.net/JB2008/indices/DTCFILE.TXT";

#[derive(Clone, Debug, Default)]
pub struct CacheHeaders {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug)]
pub enum FetchResult {
    Data { bytes: Vec<u8>, cache: CacheHeaders },
    NotModified,
}

#[derive(Debug)]
pub enum FetchError {
    Http(reqwest::Error),
    Status(reqwest::StatusCode),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Status(code) => write!(f, "unexpected status: {code}"),
        }
    }
}

impl std::error::Error for FetchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Status(_) => None,
        }
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

fn extract_cache_headers(headers: &reqwest::header::HeaderMap) -> CacheHeaders {
    CacheHeaders {
        etag: headers
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        last_modified: headers
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    }
}
